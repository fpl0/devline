package main

import (
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"log"
	"net/http"
	"os"
	"slices"
	"strconv"
	"strings"
	"sync"
	"time"
	"unicode/utf8"

	"github.com/go-chi/chi/v5"
)

type taskInput struct {
	Title     string `json:"title"`
	Completed bool   `json:"completed"`
}

type task struct {
	ID int `json:"id"`
	taskInput
}

type taskStore struct {
	mu   sync.RWMutex
	next int
	rows map[int]task
}

func (store *taskStore) list() []task {
	store.mu.RLock()
	defer store.mu.RUnlock()
	tasks := make([]task, 0, len(store.rows))
	for _, item := range store.rows {
		tasks = append(tasks, item)
	}
	slices.SortFunc(tasks, func(a, b task) int { return a.ID - b.ID })
	return tasks
}

func (store *taskStore) add(input taskInput) task {
	store.mu.Lock()
	defer store.mu.Unlock()
	store.next++
	item := task{ID: store.next, taskInput: input}
	store.rows[item.ID] = item
	return item
}

func (store *taskStore) get(id int) (task, bool) {
	store.mu.RLock()
	defer store.mu.RUnlock()
	item, found := store.rows[id]
	return item, found
}

func (store *taskStore) replace(id int, input taskInput) (task, bool) {
	store.mu.Lock()
	defer store.mu.Unlock()
	if _, found := store.rows[id]; !found {
		return task{}, false
	}
	item := task{ID: id, taskInput: input}
	store.rows[id] = item
	return item, true
}

func (store *taskStore) remove(id int) bool {
	store.mu.Lock()
	defer store.mu.Unlock()
	_, found := store.rows[id]
	delete(store.rows, id)
	return found
}

func writeJSON(w http.ResponseWriter, status int, body any) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	if err := json.NewEncoder(w).Encode(body); err != nil {
		log.Printf("write response: %v", err)
	}
}

func writeError(w http.ResponseWriter, status int, code, message string) {
	writeJSON(w, status, map[string]any{
		"error": map[string]string{"code": code, "message": message},
	})
}

func readTask(w http.ResponseWriter, r *http.Request) (taskInput, bool) {
	var input taskInput
	decoder := json.NewDecoder(http.MaxBytesReader(w, r.Body, 16<<10))
	decoder.DisallowUnknownFields()
	if err := decoder.Decode(&input); err != nil {
		var limit *http.MaxBytesError
		if errors.As(err, &limit) {
			writeError(w, http.StatusRequestEntityTooLarge, "body_too_large", "Send a JSON body under 16 KiB.")
		} else {
			writeError(w, http.StatusBadRequest, "invalid_json", "Send a JSON object with title and optional completed.")
		}
		return taskInput{}, false
	}
	if err := decoder.Decode(new(any)); err != io.EOF {
		writeError(w, http.StatusBadRequest, "invalid_json", "Send exactly one JSON object.")
		return taskInput{}, false
	}
	input.Title = strings.TrimSpace(input.Title)
	if input.Title == "" || utf8.RuneCountInString(input.Title) > 120 {
		writeError(w, http.StatusUnprocessableEntity, "invalid_title", "Use a title between 1 and 120 characters.")
		return taskInput{}, false
	}
	return input, true
}

func taskID(w http.ResponseWriter, r *http.Request) (int, bool) {
	id, err := strconv.Atoi(chi.URLParam(r, "id"))
	if err != nil || id < 1 {
		writeError(w, http.StatusBadRequest, "invalid_id", "Use a positive task ID.")
		return 0, false
	}
	return id, true
}

func taskMissing(w http.ResponseWriter) {
	writeError(w, http.StatusNotFound, "task_not_found", "That task does not exist.")
}

func routes() http.Handler {
	store := &taskStore{rows: make(map[int]task)}
	router := chi.NewRouter()
	router.Get("/healthz", func(w http.ResponseWriter, _ *http.Request) {
		writeJSON(w, http.StatusOK, map[string]string{"status": "ok"})
	})
	router.Get("/version", func(w http.ResponseWriter, _ *http.Request) {
		writeJSON(w, http.StatusOK, map[string]string{"service": "__PROJECT_NAME__"})
	})
	router.Route("/tasks", func(r chi.Router) {
		r.Get("/", func(w http.ResponseWriter, _ *http.Request) {
			writeJSON(w, http.StatusOK, store.list())
		})
		r.Post("/", func(w http.ResponseWriter, r *http.Request) {
			input, ok := readTask(w, r)
			if !ok {
				return
			}
			item := store.add(input)
			w.Header().Set("Location", fmt.Sprintf("/tasks/%d", item.ID))
			writeJSON(w, http.StatusCreated, item)
		})
		r.Get("/{id}", func(w http.ResponseWriter, r *http.Request) {
			id, ok := taskID(w, r)
			if !ok {
				return
			}
			item, found := store.get(id)
			if !found {
				taskMissing(w)
				return
			}
			writeJSON(w, http.StatusOK, item)
		})
		r.Put("/{id}", func(w http.ResponseWriter, r *http.Request) {
			id, ok := taskID(w, r)
			if !ok {
				return
			}
			input, ok := readTask(w, r)
			if !ok {
				return
			}
			item, found := store.replace(id, input)
			if !found {
				taskMissing(w)
				return
			}
			writeJSON(w, http.StatusOK, item)
		})
		r.Delete("/{id}", func(w http.ResponseWriter, r *http.Request) {
			id, ok := taskID(w, r)
			if !ok {
				return
			}
			if !store.remove(id) {
				taskMissing(w)
				return
			}
			w.WriteHeader(http.StatusNoContent)
		})
	})
	router.NotFound(func(w http.ResponseWriter, _ *http.Request) {
		writeError(w, http.StatusNotFound, "route_not_found", "That route does not exist.")
	})
	return router
}

func main() {
	address := os.Getenv("LISTEN_ADDR")
	if address == "" {
		address = "127.0.0.1:8080"
	}
	server := &http.Server{
		Addr:              address,
		Handler:           routes(),
		ReadHeaderTimeout: 5 * time.Second,
		ReadTimeout:       10 * time.Second,
		WriteTimeout:      10 * time.Second,
		IdleTimeout:       60 * time.Second,
	}
	log.Printf("Serving on http://%s", server.Addr)
	log.Fatal(server.ListenAndServe())
}
