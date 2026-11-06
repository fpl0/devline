package main

import (
	"encoding/json"
	"log"
	"net/http"
	"time"

	"github.com/go-chi/chi/v5"
)

func routes() http.Handler {
	router := chi.NewRouter()
	router.Get("/healthz", func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		_ = json.NewEncoder(w).Encode(map[string]string{"status": "ok"})
	})
	return router
}

func main() {
	server := &http.Server{
		Addr:              "127.0.0.1:8080",
		Handler:           routes(),
		ReadHeaderTimeout: 5 * time.Second,
	}
	log.Printf("Serving on http://%s", server.Addr)
	log.Fatal(server.ListenAndServe())
}
