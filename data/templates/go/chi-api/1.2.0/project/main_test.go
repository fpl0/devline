package main

import (
	"bytes"
	"encoding/json"
	"errors"
	"net/http"
	"net/http/httptest"
	"os"
	"path/filepath"
	"sync"
	"testing"
)

func testStore(t *testing.T) *Store {
	t.Helper()
	s, err := OpenStore(filepath.Join(t.TempDir(), "issues.json"))
	if err != nil {
		t.Fatal(err)
	}
	t.Cleanup(func() { s.Close() })
	return s
}
func TestHTTPWorkflow(t *testing.T) {
	store := testStore(t)
	server := routes(store)
	request := func(method, path, body string, status int) []byte {
		t.Helper()
		r := httptest.NewRequest(method, path, bytes.NewBufferString(body))
		w := httptest.NewRecorder()
		server.ServeHTTP(w, r)
		if w.Code != status {
			t.Fatalf("%s %s: %d %s", method, path, w.Code, w.Body.String())
		}
		return w.Body.Bytes()
	}
	request("POST", "/issues", `{"title":"Fix sync","priority":"high"}`, 201)
	request("PUT", "/issues/1", `{"title":"Fix sync","description":"Reproduce offline","priority":"high","status":"doing","version":1}`, 200)
	request("PUT", "/issues/1", `{"title":"stale","priority":"high","status":"doing","version":1}`, 409)
	request("POST", "/issues/1/comments", `{"version":2,"body":"Reproduced on macOS"}`, 200)
	request("PUT", "/issues/1", `{"title":"Fix sync","priority":"high","status":"done","version":3}`, 200)
	var i Issue
	json.Unmarshal(request("GET", "/issues/1", "", 200), &i)
	if i.Version != 4 || len(i.History) != 4 || i.Status != "done" {
		t.Fatalf("%+v", i)
	}
	var list struct{ Total int }
	json.Unmarshal(request("GET", "/issues?status=done&q=sync", "", 200), &list)
	if list.Total != 1 {
		t.Fatal(list)
	}
	request("POST", "/issues", `{"title":" "}`, 422)
	request("POST", "/issues", `{"title":"x","surprise":true}`, 400)
	request("POST", "/issues", `{"title":"x"} {}`, 400)
	request("GET", "/issues/999", "", 404)
	request("GET", "/issues?limit=0", "", 400)
}
func TestPersistenceConflictAndAtomicFailure(t *testing.T) {
	path := filepath.Join(t.TempDir(), "issues.json")
	s, err := OpenStore(path)
	if err != nil {
		t.Fatal(err)
	}
	if _, err = OpenStore(path); err == nil {
		t.Fatal("accepted second writer")
	}
	i, err := s.Create(Input{Title: "Keep me"})
	if err != nil {
		t.Fatal(err)
	}
	s.Close()
	s, err = OpenStore(path)
	if err != nil {
		t.Fatal(err)
	}
	defer s.Close()
	got, err := s.Get(i.ID)
	if err != nil || got.Title != "Keep me" {
		t.Fatal(got, err)
	}
	if _, err = s.Update(i.ID, Input{Title: i.Title, Priority: i.Priority, Status: "done", Version: 1}); !errors.Is(err, ErrTransition) {
		t.Fatal(err)
	}
	oldPath := s.path
	s.path = filepath.Dir(path)
	if _, err = s.Create(Input{Title: "must not commit"}); err == nil {
		t.Fatal("expected failed save")
	}
	s.path = oldPath
	if len(s.List("", "")) != 1 {
		t.Fatal("failed transaction changed memory")
	}
}
func TestConcurrentCreatesAndCorruption(t *testing.T) {
	s := testStore(t)
	var wg sync.WaitGroup
	for range 20 {
		wg.Go(func() {
			if _, err := s.Create(Input{Title: "Concurrent"}); err != nil {
				t.Error(err)
			}
		})
	}
	wg.Wait()
	if len(s.List("", "")) != 20 {
		t.Fatal("lost writes")
	}
	path := filepath.Join(t.TempDir(), "issues.json")
	os.WriteFile(path, []byte(`{"schema":1,"next_id":1,"issues":[{}]}`), 0600)
	if _, err := OpenStore(path); err == nil {
		t.Fatal("accepted corruption")
	}
}

var _ http.Handler

func TestStoreBudgetAndInvalidCounters(t *testing.T) {
	s := testStore(t)
	s.maxBytes = 20
	if _, err := s.Create(Input{Title: "Too large"}); err == nil {
		t.Fatal("exceeded persistence budget")
	}
	if len(s.List("", "")) != 0 {
		t.Fatal("published failed write")
	}
	path := filepath.Join(t.TempDir(), "issues.json")
	os.WriteFile(path, []byte(`{"schema":1,"next_id":9223372036854775807,"issues":[]}`), 0600)
	if _, err := OpenStore(path); err == nil {
		t.Fatal("accepted invalid ID counter")
	}
}
