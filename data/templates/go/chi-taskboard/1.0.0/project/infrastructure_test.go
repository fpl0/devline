//go:build integration

package main

import (
	"context"
	"fmt"
	"strings"
	"testing"
	"time"
)

func TestLiveInfrastructure(t *testing.T) {
	config, err := loadConfig()
	if err != nil {
		t.Fatal(err)
	}
	ctx, cancel := context.WithTimeout(context.Background(), 20*time.Second)
	defer cancel()
	store, err := openBoard(ctx, config)
	if err != nil {
		t.Fatal(err)
	}
	defer store.close()
	if err := store.redis.Ping(ctx).Err(); err != nil {
		t.Fatal("could not reach Redis")
	}
	if err := store.migrate(ctx); err != nil {
		t.Fatal("could not initialize board table")
	}
	app := handler(store)
	title := fmt.Sprintf("Taskboard smoke test %d", time.Now().UnixNano())
	exercise(t, app, "POST", "/tasks", title, 303)
	var id int64
	if err := store.db.QueryRow(ctx, "SELECT id FROM tasks WHERE title = $1", title).Scan(&id); err != nil {
		t.Fatal("created task missing from PostgreSQL")
	}
	defer func() { _ = store.Delete(context.Background(), id) }()
	first := exercise(t, app, "GET", "/", "", 200)
	if !strings.Contains(first.Body.String(), title) || first.Header().Get("X-Taskboard-Read-From") != "PostgreSQL" {
		t.Fatal("first read did not load persisted task")
	}
	second := exercise(t, app, "GET", "/", "", 200)
	if second.Header().Get("X-Taskboard-Read-From") != "Redis" {
		t.Fatal("second read missed Redis cache")
	}
	path := fmt.Sprintf("/tasks/%d", id)
	exercise(t, app, "POST", path+"/rename", title+" renamed", 303)
	exercise(t, app, "POST", path+"/toggle", "", 303)
	fresh := exercise(t, handler(&boardStore{db: store.db, redis: store.redis}), "GET", "/", "", 200)
	if !strings.Contains(fresh.Body.String(), title+" renamed") || fresh.Header().Get("X-Taskboard-Read-From") != "PostgreSQL" {
		t.Fatal("mutation did not persist and invalidate Redis")
	}
	var done bool
	if err := store.db.QueryRow(ctx, "SELECT done FROM tasks WHERE id = $1", id).Scan(&done); err != nil || !done {
		t.Fatal("completion is not persisted")
	}
	exercise(t, app, "POST", path+"/delete", "", 303)
	last := exercise(t, app, "GET", "/", "", 200)
	if strings.Contains(last.Body.String(), title) {
		t.Fatal("deleted task still appears")
	}
}
