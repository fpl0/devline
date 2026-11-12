package main

import (
	"context"
	"embed"
	"encoding/json"
	"errors"
	"html/template"
	"log"
	"net/http"
	"os"
	"os/signal"
	"syscall"
	"time"
)

//go:embed web/*
var assets embed.FS
var page = template.Must(template.ParseFS(assets, "web/index.html"))

func routes(m *Monitor) http.Handler {
	mux := http.NewServeMux()
	mux.HandleFunc("GET /{$}", func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "text/html; charset=utf-8")
		w.Header().Set("Cache-Control", "no-store")
		if err := page.Execute(w, struct {
			Records  []Record
			Interval int
		}{m.Records(), m.cfg.IntervalSeconds}); err != nil {
			log.Print(err)
		}
	})
	mux.HandleFunc("GET /api/monitors", func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(m.Records())
	})
	mux.HandleFunc("GET /healthz", func(w http.ResponseWriter, r *http.Request) { w.Write([]byte("ok")) })
	mux.HandleFunc("GET /demo-service", func(w http.ResponseWriter, r *http.Request) { w.Write([]byte("The example service is healthy.")) })
	mux.HandleFunc("GET /demo-failing", func(w http.ResponseWriter, r *http.Request) {
		http.Error(w, "Example outage", http.StatusServiceUnavailable)
	})
	return mux
}
func main() {
	addr := os.Getenv("LISTEN_ADDR")
	if addr == "" {
		addr = "127.0.0.1:8080"
	}
	cfg := Config{IntervalSeconds: 10, TimeoutMilliseconds: 2000, Targets: []Target{{"Example service", "http://" + addr + "/demo-service", 200}, {"Example outage", "http://" + addr + "/demo-failing", 200}}}
	if file := os.Getenv("WATCHTOWER_CONFIG"); file != "" {
		f, err := os.Open(file)
		if err != nil {
			log.Fatal(err)
		}
		dec := json.NewDecoder(f)
		dec.DisallowUnknownFields()
		err = dec.Decode(&cfg)
		f.Close()
		if err != nil {
			log.Fatal(err)
		}
	}
	data := os.Getenv("WATCHTOWER_DATA")
	if data == "" {
		data = ".data/history.json"
	}
	monitor, err := NewMonitor(cfg, data)
	if err != nil {
		log.Fatal(err)
	}
	ctx, stop := signal.NotifyContext(context.Background(), os.Interrupt, syscall.SIGTERM)
	defer stop()
	server := &http.Server{Addr: addr, Handler: routes(monitor), ReadHeaderTimeout: 5 * time.Second, ReadTimeout: 10 * time.Second, WriteTimeout: 10 * time.Second, IdleTimeout: 30 * time.Second}
	go func() {
		timer := time.NewTimer(200 * time.Millisecond)
		defer timer.Stop()
		for {
			select {
			case <-ctx.Done():
				return
			case <-timer.C:
				if err := monitor.Poll(ctx); err != nil && ctx.Err() == nil {
					log.Printf("check/persistence error: %v", err)
				}
				timer.Reset(time.Duration(cfg.IntervalSeconds) * time.Second)
			}
		}
	}()
	go func() {
		<-ctx.Done()
		deadline, cancel := context.WithTimeout(context.Background(), 5*time.Second)
		defer cancel()
		server.Shutdown(deadline)
	}()
	log.Printf("Watchtower: http://%s", addr)
	if err = server.ListenAndServe(); err != nil && !errors.Is(err, http.ErrServerClosed) {
		log.Fatal(err)
	}
}
