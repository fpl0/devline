package main

import (
	"context"
	"io"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
)

func TestServiceClientsConstructOffline(t *testing.T) {
	ctx := context.Background()
	pg, err := PostgreSQL(ctx, "postgresql://test:test@127.0.0.1:5432/devline")
	if err != nil {
		t.Fatal(err)
	}
	defer pg.Close()
	if pg.Stat().TotalConns() != 0 {
		t.Fatal("PostgreSQL pool opened a connection eagerly")
	}
	mysql, err := MySQL("devline:test@tcp(127.0.0.1:3306)/devline")
	if err != nil {
		t.Fatal(err)
	}
	defer mysql.Close()
	if mysql.Stats().OpenConnections != 0 {
		t.Fatal("MySQL pool opened a connection eagerly")
	}
	cache, err := Redis("redis://:test@127.0.0.1:6379/0")
	if err != nil {
		t.Fatal(err)
	}
	defer cache.Close()
	if uri, err := RabbitMQURL("amqp://test:test@127.0.0.1:5672/%2f"); err != nil || uri.Port != 5672 {
		t.Fatal("invalid RabbitMQ fixture")
	}
	if config, err := NATSOptions("nats://127.0.0.1:4222"); err != nil || len(config.Servers) != 1 {
		t.Fatal("invalid NATS fixture")
	}
	if _, err := ObjectStorage("http://127.0.0.1:8333", "us-east-1", "local", "local-secret"); err != nil {
		t.Fatal(err)
	}
	if _, err := NATSOptions("http://127.0.0.1:4222"); err == nil {
		t.Fatal("accepted a non-NATS scheme")
	}
}

func TestNotesHTTPAndSQLite(t *testing.T) {
	db, err := SQLite(":memory:")
	if err != nil {
		t.Fatal(err)
	}
	defer db.Close()
	if err := prepareNotes(context.Background(), db); err != nil {
		t.Fatal(err)
	}
	server := httptest.NewServer(handler(db, nil))
	defer server.Close()
	client := HTTPClient()
	created, err := client.Post(server.URL+"/notes", "application/json", strings.NewReader(`{"text":"works offline"}`))
	if err != nil {
		t.Fatal(err)
	}
	created.Body.Close()
	if created.StatusCode != http.StatusCreated {
		t.Fatalf("POST returned %d", created.StatusCode)
	}
	notes, err := client.Get(server.URL + "/notes")
	if err != nil {
		t.Fatal(err)
	}
	defer notes.Body.Close()
	contents, err := io.ReadAll(notes.Body)
	if err != nil {
		t.Fatal(err)
	}
	if notes.StatusCode != http.StatusOK || strings.TrimSpace(string(contents)) != `["works offline"]` {
		t.Fatalf("GET returned %d: %s", notes.StatusCode, contents)
	}
}

func TestJWTRejectsWrongKey(t *testing.T) {
	key := []byte("0123456789abcdefghijklmnopqrstuv")
	token, err := MintToken(key, "developer")
	if err != nil {
		t.Fatal(err)
	}
	claims, err := VerifyToken(key, token)
	if err != nil || claims.Subject != "developer" {
		t.Fatal("could not verify subject", err)
	}
	if _, err := VerifyToken([]byte("xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"), token); err == nil {
		t.Fatal("accepted a token signed with another key")
	}
}
