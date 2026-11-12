package main

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"os"
	"path/filepath"
	"slices"
	"strings"
	"sync"
	"time"
)

type Target struct {
	Name     string `json:"name"`
	URL      string `json:"url"`
	Expected int    `json:"expected_status"`
}
type Config struct {
	IntervalSeconds     int      `json:"interval_seconds"`
	TimeoutMilliseconds int      `json:"timeout_ms"`
	Targets             []Target `json:"targets"`
}

func (c Config) Validate() error {
	if c.IntervalSeconds < 1 || c.IntervalSeconds > 3600 || c.TimeoutMilliseconds < 50 || c.TimeoutMilliseconds > 30000 {
		return errors.New("interval_seconds must be 1–3600; timeout_ms must be 50–30000")
	}
	if len(c.Targets) == 0 || len(c.Targets) > 50 {
		return errors.New("configure 1–50 targets")
	}
	seen := map[string]bool{}
	for _, t := range c.Targets {
		u, err := url.Parse(t.URL)
		if strings.TrimSpace(t.Name) == "" || len(t.Name) > 80 || seen[t.Name] || err != nil || u.Host == "" || (u.Scheme != "http" && u.Scheme != "https") || u.User != nil || t.Expected < 100 || t.Expected > 599 {
			return fmt.Errorf("invalid or duplicate target %q", t.Name)
		}
		seen[t.Name] = true
	}
	return nil
}

type Check struct {
	At           time.Time `json:"at"`
	Milliseconds int64     `json:"milliseconds"`
	Status       int       `json:"status"`
	OK           bool      `json:"ok"`
	Error        string    `json:"error,omitempty"`
}
type Record struct {
	Target       Target  `json:"target"`
	Checks       []Check `json:"checks"`
	Latest       *Check  `json:"latest"`
	Availability string  `json:"availability"`
}
type Monitor struct {
	mu      sync.RWMutex
	cfg     Config
	history map[string][]Check
	file    string
	client  *http.Client
}

func NewMonitor(cfg Config, file string) (*Monitor, error) {
	if err := cfg.Validate(); err != nil {
		return nil, err
	}
	m := &Monitor{cfg: cfg, file: file, history: map[string][]Check{}, client: &http.Client{Timeout: time.Duration(cfg.TimeoutMilliseconds) * time.Millisecond, CheckRedirect: func(_ *http.Request, _ []*http.Request) error { return http.ErrUseLastResponse }}}
	if file != "" {
		data, err := os.ReadFile(file)
		if err == nil {
			var saved struct {
				Version int                `json:"version"`
				History map[string][]Check `json:"history"`
			}
			if json.Unmarshal(data, &saved) != nil || saved.Version != 1 || saved.History == nil {
				return nil, errors.New("invalid history file; restore or move it before starting")
			}
			for _, t := range cfg.Targets {
				checks := saved.History[t.Name+"\n"+t.URL]
				if len(checks) > 100 {
					checks = checks[len(checks)-100:]
				}
				m.history[t.Name+"\n"+t.URL] = checks
			}
		} else if !os.IsNotExist(err) {
			return nil, err
		}
	}
	return m, nil
}
func (m *Monitor) check(ctx context.Context, t Target) Check {
	start := time.Now()
	check := Check{At: start.UTC()}
	req, err := http.NewRequestWithContext(ctx, http.MethodGet, t.URL, nil)
	if err == nil {
		req.Header.Set("User-Agent", "Watchtower/1.0")
		var response *http.Response
		response, err = m.client.Do(req)
		if err == nil {
			check.Status = response.StatusCode
			check.OK = response.StatusCode == t.Expected
			_, bodyError := io.Copy(io.Discard, io.LimitReader(response.Body, 4096))
			if bodyError != nil {
				check.OK = false
				check.Error = "read response body: " + bodyError.Error()
			}
			response.Body.Close()
			if !check.OK && check.Error == "" {
				check.Error = fmt.Sprintf("expected HTTP %d, received %d", t.Expected, check.Status)
			}
		}
	}
	if err != nil {
		check.Error = err.Error()
	}
	check.Milliseconds = time.Since(start).Milliseconds()
	return check
}
func (m *Monitor) Poll(ctx context.Context) error {
	type result struct {
		target Target
		check  Check
	}
	jobs := make(chan Target)
	results := make(chan result, len(m.cfg.Targets))
	var wg sync.WaitGroup
	for range min(4, len(m.cfg.Targets)) {
		wg.Go(func() {
			for t := range jobs {
				results <- result{t, m.check(ctx, t)}
			}
		})
	}
	go func() {
		defer close(jobs)
		for _, t := range m.cfg.Targets {
			select {
			case jobs <- t:
			case <-ctx.Done():
				return
			}
		}
	}()
	wg.Wait()
	close(results)
	if ctx.Err() != nil {
		return ctx.Err()
	}
	m.mu.Lock()
	defer m.mu.Unlock()
	for r := range results {
		key := r.target.Name + "\n" + r.target.URL
		checks := append(m.history[key], r.check)
		if len(checks) > 100 {
			checks = checks[len(checks)-100:]
		}
		m.history[key] = checks
	}
	return m.save()
}
func (m *Monitor) save() error {
	if m.file == "" {
		return nil
	}
	if err := os.MkdirAll(filepath.Dir(m.file), 0700); err != nil {
		return err
	}
	f, err := os.CreateTemp(filepath.Dir(m.file), ".watchtower-*")
	if err != nil {
		return err
	}
	defer os.Remove(f.Name())
	defer f.Close()
	value := struct {
		Version int                `json:"version"`
		History map[string][]Check `json:"history"`
	}{1, m.history}
	if err = json.NewEncoder(f).Encode(value); err != nil {
		return err
	}
	if err = f.Sync(); err != nil {
		return err
	}
	if err = f.Close(); err != nil {
		return err
	}
	return os.Rename(f.Name(), m.file)
}
func (m *Monitor) Records() []Record {
	m.mu.RLock()
	defer m.mu.RUnlock()
	rows := []Record{}
	for _, t := range m.cfg.Targets {
		checks := slices.Clone(m.history[t.Name+"\n"+t.URL])
		row := Record{Target: t, Checks: checks, Availability: "—"}
		if len(checks) > 0 {
			last := checks[len(checks)-1]
			row.Latest = &last
			up := 0
			for _, c := range checks {
				if c.OK {
					up++
				}
			}
			row.Availability = fmt.Sprintf("%.1f%%", 100*float64(up)/float64(len(checks)))
		}
		rows = append(rows, row)
	}
	return rows
}
