package main

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"sync/atomic"
	"time"

	"github.com/jackc/pgx/v5/pgxpool"
	"github.com/redis/go-redis/v9"
)

const cacheTTL = 30 * time.Second
const generationKey = "devline:taskboard:generation"

var errNotFound = errors.New("task not found")

type task struct {
	ID    int64
	Title string
	Done  bool
}

type taskStore interface {
	List(context.Context) ([]task, string, error)
	Add(context.Context, string) error
	Rename(context.Context, int64, string) error
	Toggle(context.Context, int64) error
	Delete(context.Context, int64) error
}

type boardStore struct {
	db          *pgxpool.Pool
	redis       *redis.Client
	bypassUntil atomic.Int64
}

func openBoard(ctx context.Context, config configuration) (*boardStore, error) {
	dbConfig, err := pgxpool.ParseConfig(config.postgres)
	if err != nil || !loopback(dbConfig.ConnConfig.Host) {
		return nil, errors.New("invalid local PostgreSQL configuration")
	}
	for _, fallback := range dbConfig.ConnConfig.Fallbacks {
		if !loopback(fallback.Host) {
			return nil, errors.New("PostgreSQL fallbacks must use loopback addresses")
		}
	}
	redisConfig, err := redis.ParseURL(config.redis)
	if err != nil {
		return nil, errors.New("invalid local Redis configuration")
	}
	redisConfig.DialTimeout = 300 * time.Millisecond
	redisConfig.ReadTimeout = 300 * time.Millisecond
	redisConfig.WriteTimeout = 300 * time.Millisecond
	redisConfig.MaxRetries = -1
	db, err := pgxpool.NewWithConfig(ctx, dbConfig)
	if err != nil {
		return nil, errors.New("could not open PostgreSQL connection pool")
	}
	cache := redis.NewClient(redisConfig)
	return &boardStore{db: db, redis: cache}, nil
}

func (b *boardStore) close() {
	b.db.Close()
	_ = b.redis.Close()
}

func (b *boardStore) migrate(ctx context.Context) error {
	_, err := b.db.Exec(ctx, `CREATE TABLE IF NOT EXISTS tasks (
		id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
		title text NOT NULL CHECK (length(title) BETWEEN 1 AND 120),
		done boolean NOT NULL DEFAULT false
	)`)
	return err
}

func (b *boardStore) List(ctx context.Context) ([]task, string, error) {
	key := b.cacheKey(ctx)
	if key != "" {
		var tasks []task
		if data, err := b.redis.Get(ctx, key).Bytes(); err == nil && json.Unmarshal(data, &tasks) == nil {
			return tasks, "Redis", nil
		}
	}
	rows, err := b.db.Query(ctx, "SELECT id, title, done FROM tasks ORDER BY done, id DESC")
	if err != nil {
		return nil, "", err
	}
	defer rows.Close()
	tasks := make([]task, 0)
	for rows.Next() {
		var t task
		if err := rows.Scan(&t.ID, &t.Title, &t.Done); err != nil {
			return nil, "", err
		}
		tasks = append(tasks, t)
	}
	if err := rows.Err(); err != nil {
		return nil, "", err
	}
	if key != "" {
		if data, err := json.Marshal(tasks); err == nil {
			_ = b.redis.Set(ctx, key, data, cacheTTL).Err()
		}
	}
	return tasks, "PostgreSQL", nil
}

func (b *boardStore) cacheKey(ctx context.Context) string {
	if time.Now().UnixNano() < b.bypassUntil.Load() {
		return ""
	}
	generation, err := b.redis.Get(ctx, generationKey).Int64()
	if err != nil && !errors.Is(err, redis.Nil) {
		return ""
	}
	return fmt.Sprintf("devline:taskboard:list:%d", generation)
}

func (b *boardStore) write(ctx context.Context, query string, args ...any) error {
	tag, err := b.db.Exec(ctx, query, args...)
	if err != nil {
		return err
	}
	if tag.RowsAffected() == 0 {
		return errNotFound
	}
	// Keep a read that began before this write from repopulating the current cache key.
	cacheContext, cancel := context.WithTimeout(context.WithoutCancel(ctx), time.Second)
	defer cancel()
	if err := b.redis.Incr(cacheContext, generationKey).Err(); err != nil {
		b.bypassUntil.Store(time.Now().Add(cacheTTL).UnixNano())
	}
	return nil
}

func (b *boardStore) Add(ctx context.Context, title string) error {
	return b.write(ctx, "INSERT INTO tasks (title) VALUES ($1)", title)
}
func (b *boardStore) Rename(ctx context.Context, id int64, title string) error {
	return b.write(ctx, "UPDATE tasks SET title = $2 WHERE id = $1", id, title)
}
func (b *boardStore) Toggle(ctx context.Context, id int64) error {
	return b.write(ctx, "UPDATE tasks SET done = NOT done WHERE id = $1", id)
}
func (b *boardStore) Delete(ctx context.Context, id int64) error {
	return b.write(ctx, "DELETE FROM tasks WHERE id = $1", id)
}
