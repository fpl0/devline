package main

import (
	"context"
	"net/http"
	"net/http/httptest"
	"net/url"
	"strings"
	"testing"
)

type testStore struct {
	tasks map[int64]task
	next  int64
}

func (m *testStore) List(context.Context) ([]task, string, error) {
	result := make([]task, 0, len(m.tasks))
	for _, t := range m.tasks {
		result = append(result, t)
	}
	return result, "test store", nil
}
func (m *testStore) Add(_ context.Context, title string) error {
	m.next++
	m.tasks[m.next] = task{ID: m.next, Title: title}
	return nil
}
func (m *testStore) Rename(_ context.Context, id int64, title string) error {
	t, ok := m.tasks[id]
	if !ok {
		return errNotFound
	}
	t.Title = title
	m.tasks[id] = t
	return nil
}
func (m *testStore) Toggle(_ context.Context, id int64) error {
	t, ok := m.tasks[id]
	if !ok {
		return errNotFound
	}
	t.Done = !t.Done
	m.tasks[id] = t
	return nil
}
func (m *testStore) Delete(_ context.Context, id int64) error {
	if _, ok := m.tasks[id]; !ok {
		return errNotFound
	}
	delete(m.tasks, id)
	return nil
}

func exercise(t *testing.T, app http.Handler, method, path, title string, status int) *httptest.ResponseRecorder {
	t.Helper()
	body := url.Values{"title": {title}}.Encode()
	r := httptest.NewRequest(method, "http://localhost"+path, strings.NewReader(body))
	r.Header.Set("Content-Type", "application/x-www-form-urlencoded")
	response := httptest.NewRecorder()
	app.ServeHTTP(response, r)
	if response.Code != status {
		t.Fatalf("%s %s returned %d, want %d: %s", method, path, response.Code, status, response.Body)
	}
	return response
}

func TestBoardCRUD(t *testing.T) {
	store := &testStore{tasks: make(map[int64]task)}
	app := handler(store)
	if empty := exercise(t, app, "GET", "/", "", 200).Body.String(); !strings.Contains(empty, "No tasks yet") {
		t.Fatal("empty board is unclear")
	}
	exercise(t, app, "POST", "/tasks", "  ", 400)
	exercise(t, app, "POST", "/tasks", strings.Repeat("x", 121), 400)
	if response := exercise(t, app, "POST", "/tasks", "<script>alert(1)</script>", 303); response.Header().Get("Location") != "/" {
		t.Fatal("expected the board redirect")
	}
	page := exercise(t, app, "GET", "/", "", 200).Body.String()
	if !strings.Contains(page, "&lt;script&gt;alert(1)&lt;/script&gt;") || strings.Contains(page, "<script>") {
		t.Fatal("task title must be HTML escaped")
	}
	exercise(t, app, "POST", "/tasks/1/rename", "Pack a bag", 303)
	exercise(t, app, "POST", "/tasks/1/toggle", "", 303)
	page = exercise(t, app, "GET", "/", "", 200).Body.String()
	if !strings.Contains(page, "1 of 1 complete") || !strings.Contains(page, "Pack a bag") {
		t.Fatal("rename/toggle did not update the board")
	}
	exercise(t, app, "POST", "/tasks/1/delete", "", 303)
	exercise(t, app, "POST", "/tasks/1/toggle", "", 404)
	exercise(t, app, "POST", "/tasks/-1/toggle", "", 404)
	if len(store.tasks) != 0 {
		t.Fatal("delete did not remove the task")
	}
}

func TestRejectsExternalWrites(t *testing.T) {
	app := handler(&testStore{tasks: make(map[int64]task)})
	for _, host := range []string{"http://example.net/tasks", "http://localhost/tasks"} {
		r := httptest.NewRequest("POST", host, nil)
		r.Header.Set("Sec-Fetch-Site", "cross-site")
		response := httptest.NewRecorder()
		app.ServeHTTP(response, r)
		if response.Code != 403 {
			t.Fatalf("%s returned %d", host, response.Code)
		}
	}
}

func TestCredentialURLUsesLocalHostAndEncodesPassword(t *testing.T) {
	values := map[string]string{"DEVLINE_POSTGRES_PASSWORD": "a@b?c", "DEVLINE_POSTGRES_DATABASE_PORT": "45432"}
	dsn, err := localServiceURL(values, "POSTGRES", "DATABASE", "postgres", "devline", "/devline")
	if err != nil {
		t.Fatal(err)
	}
	parsed, err := url.Parse(dsn)
	if err != nil {
		t.Fatal(err)
	}
	password, _ := parsed.User.Password()
	if parsed.Hostname() != "127.0.0.1" || parsed.Port() != "45432" || password != "a@b?c" {
		t.Fatalf("unexpected URL: %s", parsed.Redacted())
	}
	if localURL("postgres://example.net/database", "postgres") {
		t.Fatal("accepted a remote URL")
	}
}
