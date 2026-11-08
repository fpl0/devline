package main

import (
	"context"
	"database/sql"
	"encoding/json"
	"errors"
	"log/slog"
	"net"
	"net/http"
	"os"
	"os/signal"
	"strings"
	"syscall"
	"time"

	"github.com/go-chi/chi/v5"
)

func handler(db *sql.DB, lab *releaseLab) http.Handler {
	router := chi.NewRouter()
	router.Use(localRequests, http.NewCrossOriginProtection().Handler)
	links := map[string]string{"app": "Release Rehearsal", "health": "/healthz", "local_notes": "/notes"}
	if lab != nil {
		lab.routes(router)
		links["rehearse"] = "POST /releases/{id}"
		links["receipt"] = "/releases/{id}"
		links["artifact"] = "/releases/{id}/artifact"
		links["search"] = "/search?q=term"
		links["mail_inbox"] = lab.mailURL
	}
	router.Get("/", func(w http.ResponseWriter, _ *http.Request) { reply(w, http.StatusOK, links) })
	router.Get("/healthz", func(w http.ResponseWriter, r *http.Request) {
		if err := db.PingContext(r.Context()); err != nil {
			http.Error(w, "SQLite unavailable", http.StatusServiceUnavailable)
			return
		}
		w.Header().Set("Content-Type", "application/json")
		_ = json.NewEncoder(w).Encode(map[string]string{"status": "ok", "database": "sqlite"})
	})
	router.Post("/notes", func(w http.ResponseWriter, r *http.Request) {
		var input struct {
			Text string `json:"text"`
		}
		decoder := json.NewDecoder(http.MaxBytesReader(w, r.Body, 4096))
		if err := decoder.Decode(&input); err != nil || strings.TrimSpace(input.Text) == "" || len(input.Text) > 1024 {
			http.Error(w, "Provide JSON text between 1 and 1024 bytes", http.StatusBadRequest)
			return
		}
		if _, err := db.ExecContext(r.Context(), "INSERT INTO notes (text) VALUES (?)", input.Text); err != nil {
			http.Error(w, "Could not save note", http.StatusServiceUnavailable)
			return
		}
		w.WriteHeader(http.StatusCreated)
	})
	router.Get("/notes", func(w http.ResponseWriter, r *http.Request) {
		rows, err := db.QueryContext(r.Context(), "SELECT text FROM notes ORDER BY id DESC LIMIT 100")
		if err != nil {
			http.Error(w, "Could not read notes", http.StatusServiceUnavailable)
			return
		}
		defer rows.Close()
		notes := make([]string, 0)
		for rows.Next() {
			var text string
			if err := rows.Scan(&text); err != nil {
				http.Error(w, "Could not read notes", http.StatusServiceUnavailable)
				return
			}
			notes = append(notes, text)
		}
		if err := rows.Err(); err != nil {
			http.Error(w, "Could not read notes", http.StatusServiceUnavailable)
			return
		}
		w.Header().Set("Content-Type", "application/json")
		_ = json.NewEncoder(w).Encode(notes)
	})
	return router
}

func prepareNotes(ctx context.Context, db *sql.DB) error {
	_, err := db.ExecContext(ctx, "CREATE TABLE IF NOT EXISTS notes (id INTEGER PRIMARY KEY, text TEXT NOT NULL)")
	return err
}

func main() {
	if err := run(); err != nil {
		slog.Error("Service stopped", "error", err)
		os.Exit(1)
	}
}

func run() error {
	address := os.Getenv("SERVICE_LISTEN")
	if address == "" {
		address = "127.0.0.1:8080"
	}
	host, _, err := net.SplitHostPort(address)
	if err != nil || !net.ParseIP(host).IsLoopback() {
		return errors.New("SERVICE_LISTEN must use a loopback IP and port")
	}
	path := os.Getenv("SERVICE_DB")
	if path == "" {
		path = "notes.sqlite3"
	}
	db, err := SQLite(path)
	if err != nil {
		return err
	}
	defer db.Close()
	ctx, stop := signal.NotifyContext(context.Background(), os.Interrupt, syscall.SIGTERM)
	defer stop()
	if err := prepareNotes(ctx, db); err != nil {
		return err
	}
	listener, err := net.Listen("tcp", address)
	if err != nil {
		return err
	}
	var lab *releaseLab
	if os.Getenv("DEVLINE_POSTGRES_URL") != "" {
		initializing, cancel := context.WithTimeout(ctx, 20*time.Second)
		lab, err = newReleaseLab(initializing)
		cancel()
		if err != nil {
			return err
		}
		defer lab.close()
	}
	server := &http.Server{Handler: handler(db, lab), ReadHeaderTimeout: 3 * time.Second, ReadTimeout: 6 * time.Second, WriteTimeout: 30 * time.Second, IdleTimeout: 30 * time.Second}
	go func() {
		<-ctx.Done()
		shutdown, cancel := context.WithTimeout(context.Background(), 3*time.Second)
		defer cancel()
		_ = server.Shutdown(shutdown)
	}()
	slog.Info("Service ready", "address", listener.Addr().String())
	if err := server.Serve(listener); !errors.Is(err, http.ErrServerClosed) {
		return err
	}
	return nil
}

func localRequests(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		host := r.Host
		if address, _, err := net.SplitHostPort(host); err == nil {
			host = address
		}
		if host != "localhost" && !net.ParseIP(strings.Trim(host, "[]")).IsLoopback() {
			http.Error(w, "Open the app through localhost", http.StatusForbidden)
			return
		}
		next.ServeHTTP(w, r)
	})
}
