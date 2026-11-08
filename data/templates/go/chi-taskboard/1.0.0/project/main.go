package main

import (
	"context"
	"embed"
	"errors"
	"html/template"
	"log"
	"net"
	"net/http"
	"os"
	"os/signal"
	"strconv"
	"strings"
	"syscall"
	"time"
	"unicode/utf8"

	"github.com/go-chi/chi/v5"
)

//go:embed web/*
var assets embed.FS
var boardPage = template.Must(template.ParseFS(assets, "web/board.html"))
var errInvalidTitle = errors.New("enter a task title between 1 and 120 characters")

type boardPageData struct {
	Tasks     []task
	Completed int
	Source    string
}

func handler(store taskStore) http.Handler {
	router := chi.NewRouter()
	router.Use(localRequests, http.NewCrossOriginProtection().Handler)
	router.Get("/healthz", func(w http.ResponseWriter, _ *http.Request) {
		w.WriteHeader(http.StatusOK)
	})
	router.Get("/", func(w http.ResponseWriter, r *http.Request) {
		ctx, cancel := context.WithTimeout(r.Context(), 4*time.Second)
		defer cancel()
		tasks, source, err := store.List(ctx)
		if err != nil {
			log.Printf("could not read board: %T", err)
			http.Error(w, "The board could not reach PostgreSQL. Check your local services.", http.StatusServiceUnavailable)
			return
		}
		w.Header().Set("Content-Type", "text/html; charset=utf-8")
		w.Header().Set("Cache-Control", "no-store")
		w.Header().Set("X-Taskboard-Read-From", source)
		data := boardPageData{Tasks: tasks, Source: source}
		for _, task := range tasks {
			if task.Done {
				data.Completed++
			}
		}
		if err := boardPage.Execute(w, data); err != nil {
			log.Print("could not finish sending the board")
		}
	})
	router.Get("/style.css", func(w http.ResponseWriter, _ *http.Request) {
		w.Header().Set("Content-Type", "text/css; charset=utf-8")
		data, _ := assets.ReadFile("web/style.css")
		_, _ = w.Write(data)
	})
	router.Post("/tasks", taskAction(false, func(ctx context.Context, w http.ResponseWriter, r *http.Request, _ int64) error {
		title, err := readTitle(w, r)
		if err != nil {
			return err
		}
		return store.Add(ctx, title)
	}))
	router.Post("/tasks/{id}/rename", taskAction(true, func(ctx context.Context, w http.ResponseWriter, r *http.Request, id int64) error {
		title, err := readTitle(w, r)
		if err != nil {
			return err
		}
		return store.Rename(ctx, id, title)
	}))
	router.Post("/tasks/{id}/toggle", taskAction(true, func(ctx context.Context, _ http.ResponseWriter, _ *http.Request, id int64) error {
		return store.Toggle(ctx, id)
	}))
	router.Post("/tasks/{id}/delete", taskAction(true, func(ctx context.Context, _ http.ResponseWriter, _ *http.Request, id int64) error {
		return store.Delete(ctx, id)
	}))
	return router
}

func localRequests(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		host := r.Host
		if parsed, _, err := net.SplitHostPort(host); err == nil {
			host = parsed
		}
		if !loopback(host) {
			http.Error(w, "Open the board through localhost.", http.StatusForbidden)
			return
		}
		w.Header().Set("X-Content-Type-Options", "nosniff")
		w.Header().Set("Content-Security-Policy", "default-src 'none'; style-src 'self'; form-action 'self'; frame-ancestors 'none'; base-uri 'none'")
		next.ServeHTTP(w, r)
	})
}

func readTitle(w http.ResponseWriter, r *http.Request) (string, error) {
	r.Body = http.MaxBytesReader(w, r.Body, 2048)
	if err := r.ParseForm(); err != nil {
		return "", errInvalidTitle
	}
	title := strings.TrimSpace(r.PostForm.Get("title"))
	if !utf8.ValidString(title) || strings.ContainsRune(title, 0) || utf8.RuneCountInString(title) < 1 || utf8.RuneCountInString(title) > 120 {
		return "", errInvalidTitle
	}
	return title, nil
}

type action func(context.Context, http.ResponseWriter, *http.Request, int64) error

func taskAction(hasID bool, perform action) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		var id int64
		if hasID {
			var err error
			id, err = strconv.ParseInt(chi.URLParam(r, "id"), 10, 64)
			if err != nil || id < 1 {
				http.NotFound(w, r)
				return
			}
		}
		ctx, cancel := context.WithTimeout(r.Context(), 4*time.Second)
		defer cancel()
		switch err := perform(ctx, w, r, id); {
		case err == nil:
			http.Redirect(w, r, "/", http.StatusSeeOther)
		case errors.Is(err, errInvalidTitle):
			http.Error(w, errInvalidTitle.Error(), http.StatusBadRequest)
		case errors.Is(err, errNotFound):
			http.NotFound(w, r)
		case err != nil:
			log.Printf("could not save task: %T", err)
			http.Error(w, "The board could not save your change. Check PostgreSQL and try again.", http.StatusServiceUnavailable)
		}
	}
}

func main() {
	if err := run(); err != nil {
		log.Fatal(err)
	}
}

func run() error {
	config, err := loadConfig()
	if err != nil {
		return err
	}
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	store, err := openBoard(ctx, config)
	if err != nil {
		return err
	}
	defer store.close()
	if err := store.migrate(ctx); err != nil {
		return errors.New("could not initialize PostgreSQL. Start the project with devline infra run")
	}
	if err := store.redis.Ping(ctx).Err(); err != nil {
		log.Print("Redis is unavailable. Reads will use PostgreSQL until Redis returns.")
	}
	listener, err := net.Listen("tcp", config.listen)
	if err != nil {
		return err
	}
	defer listener.Close()
	server := &http.Server{Handler: handler(store), ReadHeaderTimeout: 3 * time.Second, ReadTimeout: 6 * time.Second, WriteTimeout: 8 * time.Second, IdleTimeout: 30 * time.Second}
	stopping, stop := signal.NotifyContext(context.Background(), os.Interrupt, syscall.SIGTERM)
	defer stop()
	go func() {
		<-stopping.Done()
		ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
		defer cancel()
		_ = server.Shutdown(ctx)
	}()
	log.Printf("Taskboard is ready at http://%s", listener.Addr())
	if err := server.Serve(listener); !errors.Is(err, http.ErrServerClosed) {
		return err
	}
	return nil
}
