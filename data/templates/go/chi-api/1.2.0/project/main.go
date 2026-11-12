package main

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"github.com/go-chi/chi/v5"
	"io"
	"log"
	"net/http"
	"os"
	"os/signal"
	"strconv"
	"syscall"
	"time"
)

func jsonResponse(w http.ResponseWriter, status int, value any) {
	w.Header().Set("Content-Type", "application/json; charset=utf-8")
	w.WriteHeader(status)
	if err := json.NewEncoder(w).Encode(value); err != nil {
		log.Print(err)
	}
}
func problem(w http.ResponseWriter, status int, message string) {
	jsonResponse(w, status, map[string]any{"error": map[string]any{"status": status, "message": message}})
}
func result(w http.ResponseWriter, status int, i Issue, err error) {
	if err != nil {
		code := 422
		if errors.Is(err, ErrMissing) {
			code = 404
		} else if errors.Is(err, ErrConflict) || errors.Is(err, ErrTransition) {
			code = 409
		} else if errors.Unwrap(err) != nil {
			log.Print(err)
			problem(w, 500, "Unable to save the issue. No changes were committed.")
			return
		}
		problem(w, code, err.Error())
		return
	}
	w.Header().Set("ETag", fmt.Sprintf(`"%d"`, i.Version))
	jsonResponse(w, status, i)
}
func decode(w http.ResponseWriter, r *http.Request, dst any) bool {
	dec := json.NewDecoder(http.MaxBytesReader(w, r.Body, 32<<10))
	dec.DisallowUnknownFields()
	if err := dec.Decode(dst); err != nil {
		problem(w, 400, "Send one valid JSON object with documented fields (maximum 32 KiB).")
		return false
	}
	if err := dec.Decode(new(any)); err != io.EOF {
		problem(w, 400, "Send exactly one JSON object.")
		return false
	}
	return true
}
func issueID(w http.ResponseWriter, r *http.Request) (int, bool) {
	id, err := strconv.Atoi(chi.URLParam(r, "id"))
	if err != nil || id < 1 {
		problem(w, 400, "Use a positive issue ID.")
		return 0, false
	}
	return id, true
}
func routes(store *Store) http.Handler {
	r := chi.NewRouter()
	r.Get("/", func(w http.ResponseWriter, _ *http.Request) {
		jsonResponse(w, 200, map[string]any{"app": "Issue Desk", "issues": "/issues", "workflow": "open → doing → done → open", "guide": "See README.md for create, update, comment and search examples."})
	})
	r.Get("/healthz", func(w http.ResponseWriter, _ *http.Request) { jsonResponse(w, 200, map[string]string{"status": "ok"}) })
	r.Get("/issues", func(w http.ResponseWriter, r *http.Request) {
		status := r.URL.Query().Get("status")
		if status != "" && status != "open" && status != "doing" && status != "done" {
			problem(w, 400, "Unknown status filter.")
			return
		}
		limit, offset := 50, 0
		var err error
		if v := r.URL.Query().Get("limit"); v != "" {
			limit, err = strconv.Atoi(v)
			if err != nil || limit < 1 || limit > 100 {
				problem(w, 400, "limit must be 1–100")
				return
			}
		}
		if v := r.URL.Query().Get("offset"); v != "" {
			offset, err = strconv.Atoi(v)
			if err != nil || offset < 0 {
				problem(w, 400, "offset must be nonnegative")
				return
			}
		}
		rows := store.List(status, r.URL.Query().Get("q"))
		total := len(rows)
		offset = min(offset, total)
		end := min(offset+limit, total)
		jsonResponse(w, 200, map[string]any{"items": rows[offset:end], "total": total, "offset": offset, "limit": limit})
	})
	r.Post("/issues", func(w http.ResponseWriter, r *http.Request) {
		var in Input
		if !decode(w, r, &in) {
			return
		}
		i, err := store.Create(in)
		if err == nil {
			w.Header().Set("Location", fmt.Sprintf("/issues/%d", i.ID))
		}
		result(w, 201, i, err)
	})
	r.Get("/issues/{id}", func(w http.ResponseWriter, r *http.Request) {
		id, ok := issueID(w, r)
		if !ok {
			return
		}
		i, err := store.Get(id)
		result(w, 200, i, err)
	})
	r.Put("/issues/{id}", func(w http.ResponseWriter, r *http.Request) {
		id, ok := issueID(w, r)
		if !ok {
			return
		}
		var in Input
		if !decode(w, r, &in) {
			return
		}
		i, err := store.Update(id, in)
		result(w, 200, i, err)
	})
	r.Post("/issues/{id}/comments", func(w http.ResponseWriter, r *http.Request) {
		id, ok := issueID(w, r)
		if !ok {
			return
		}
		var in struct {
			Version int    `json:"version"`
			Body    string `json:"body"`
		}
		if !decode(w, r, &in) {
			return
		}
		i, err := store.Comment(id, in.Version, in.Body)
		result(w, 200, i, err)
	})
	r.NotFound(func(w http.ResponseWriter, _ *http.Request) { problem(w, 404, "Route not found.") })
	return r
}
func main() {
	path := os.Getenv("ISSUE_DATA")
	if path == "" {
		path = ".data/issues.json"
	}
	store, err := OpenStore(path)
	if err != nil {
		log.Fatal(err)
	}
	defer store.Close()
	addr := os.Getenv("LISTEN_ADDR")
	if addr == "" {
		addr = "127.0.0.1:8080"
	}
	server := &http.Server{Addr: addr, Handler: routes(store), ReadHeaderTimeout: 5 * time.Second, ReadTimeout: 10 * time.Second, WriteTimeout: 10 * time.Second, IdleTimeout: 60 * time.Second}
	ctx, stop := signal.NotifyContext(context.Background(), os.Interrupt, syscall.SIGTERM)
	defer stop()
	go func() {
		<-ctx.Done()
		shutdown, cancel := context.WithTimeout(context.Background(), 5*time.Second)
		defer cancel()
		if err := server.Shutdown(shutdown); err != nil {
			log.Print(err)
		}
	}()
	log.Printf("Issue Desk on http://%s (data: %s)", addr, path)
	if err = server.ListenAndServe(); err != nil && !errors.Is(err, http.ErrServerClosed) {
		log.Print(err)
		os.Exit(1)
	}
}
