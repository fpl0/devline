package main

import (
	"context"
	"net/http"
	"net/http/httptest"
	"os"
	"path/filepath"
	"strings"
	"testing"
	"time"
)

func TestPollPersistenceAndDashboard(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path == "/down" {
			w.WriteHeader(503)
		} else {
			w.Write([]byte("ok"))
		}
	}))
	defer server.Close()
	cfg := Config{1, 1000, []Target{{"Healthy <service>", server.URL, 200}, {"Down", server.URL + "/down", 200}}}
	path := filepath.Join(t.TempDir(), "history.json")
	monitor, err := NewMonitor(cfg, path)
	if err != nil {
		t.Fatal(err)
	}
	if err = monitor.Poll(context.Background()); err != nil {
		t.Fatal(err)
	}
	rows := monitor.Records()
	if !rows[0].Latest.OK || rows[1].Latest.OK || rows[0].Availability != "100.0%" || rows[1].Latest.Status != 503 {
		t.Fatalf("%+v", rows)
	}
	restored, err := NewMonitor(cfg, path)
	if err != nil || len(restored.Records()[0].Checks) != 1 {
		t.Fatal(err)
	}
	w := httptest.NewRecorder()
	routes(restored).ServeHTTP(w, httptest.NewRequest("GET", "/", nil))
	if w.Code != 200 || !strings.Contains(w.Body.String(), "Healthy &lt;service&gt;") {
		t.Fatal(w.Body.String())
	}
}
func TestTimeoutRedirectAndHistoryBound(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path == "/slow" {
			time.Sleep(100 * time.Millisecond)
		} else if r.URL.Path == "/redirect" {
			http.Redirect(w, r, "/", 302)
		}
	}))
	defer server.Close()
	m, err := NewMonitor(Config{1, 50, []Target{{"Slow", server.URL + "/slow", 200}, {"Redirect", server.URL + "/redirect", 200}}}, "")
	if err != nil {
		t.Fatal(err)
	}
	m.Poll(context.Background())
	rows := m.Records()
	if rows[0].Latest.OK || rows[0].Latest.Error == "" || rows[1].Latest.Status != 302 {
		t.Fatal(rows)
	}
	key := m.cfg.Targets[0].Name + "\n" + m.cfg.Targets[0].URL
	m.history[key] = make([]Check, 100)
	m.Poll(context.Background())
	if len(m.Records()[0].Checks) != 100 {
		t.Fatal("unbounded history")
	}
}
func TestInvalidConfigurationAndHistory(t *testing.T) {
	if err := (Config{0, 100, []Target{{"x", "file:///etc/passwd", 200}}}).Validate(); err == nil {
		t.Fatal("accepted invalid config")
	}
	path := filepath.Join(t.TempDir(), "bad.json")
	os.WriteFile(path, []byte("broken"), 0600)
	if _, err := NewMonitor(Config{1, 100, []Target{{"x", "http://localhost", 200}}}, path); err == nil {
		t.Fatal("ignored corrupt history")
	}
}

func TestSlowBodyIsNotHealthy(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(200)
		w.(http.Flusher).Flush()
		time.Sleep(150 * time.Millisecond)
		w.Write([]byte("late body"))
	}))
	defer server.Close()
	monitor, err := NewMonitor(Config{1, 50, []Target{{"Slow body", server.URL, 200}}}, "")
	if err != nil {
		t.Fatal(err)
	}
	if err = monitor.Poll(context.Background()); err != nil {
		t.Fatal(err)
	}
	check := monitor.Records()[0].Latest
	if check.OK || !strings.Contains(check.Error, "read response body") {
		t.Fatalf("timed-out response marked healthy: %+v", check)
	}
}
