package main

import (
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"slices"
	"strings"
	"sync"
	"syscall"
)

const maxStoreBytes int64 = 128 << 20

type snapshot struct {
	Schema int     `json:"schema"`
	NextID int     `json:"next_id"`
	Issues []Issue `json:"issues"`
}
type Store struct {
	maxBytes int64
	mu       sync.Mutex
	path     string
	data     snapshot
	lock     *os.File
}

func OpenStore(path string) (*Store, error) {
	if err := os.MkdirAll(filepath.Dir(path), 0700); err != nil {
		return nil, err
	}
	lock, err := os.OpenFile(path+".lock", os.O_CREATE|os.O_RDWR, 0600)
	if err != nil {
		return nil, err
	}
	if err = syscall.Flock(int(lock.Fd()), syscall.LOCK_EX|syscall.LOCK_NB); err != nil {
		lock.Close()
		return nil, errors.New("another Issue Desk process is using this data file")
	}
	s := &Store{maxBytes: maxStoreBytes, path: path, data: snapshot{Schema: 1, NextID: 1, Issues: []Issue{}}, lock: lock}
	file, err := os.Open(path)
	if os.IsNotExist(err) {
		return s, nil
	}
	if err != nil {
		s.Close()
		return nil, err
	}
	defer file.Close()
	info, err := file.Stat()
	if err != nil || info.Size() > maxStoreBytes {
		s.Close()
		return nil, errors.New("store exceeds its 128 MiB limit or cannot be inspected")
	}
	decoder := json.NewDecoder(io.LimitReader(file, maxStoreBytes))
	decoder.DisallowUnknownFields()
	if err = decoder.Decode(&s.data); err != nil {
		s.Close()
		return nil, fmt.Errorf("invalid store: %w", err)
	}
	if err = decoder.Decode(new(any)); err != io.EOF {
		s.Close()
		return nil, errors.New("store contains trailing data")
	}
	seen := map[int]bool{}
	valid := s.data.Schema == 1 && s.data.NextID == len(s.data.Issues)+1 && s.data.Issues != nil && len(s.data.Issues) <= 10000
	for _, row := range s.data.Issues {
		in := Input{Title: row.Title, Description: row.Description, Priority: row.Priority, Status: row.Status}
		if in.validate() != nil || row.ID < 1 || row.ID >= s.data.NextID || seen[row.ID] || row.Version != len(row.History) || len(row.History) < 1 || len(row.History) > 1000 {
			valid = false
		}
		seen[row.ID] = true
	}
	if !valid {
		s.Close()
		return nil, errors.New("store schema or issue records are invalid; restore a valid backup")
	}
	return s, nil
}
func (s *Store) Close() error {
	if s.lock != nil {
		return s.lock.Close()
	}
	return nil
}
func clone(i Issue) Issue { i.History = slices.Clone(i.History); return i }
func (s *Store) List(status, query string) []Issue {
	s.mu.Lock()
	defer s.mu.Unlock()
	rows := []Issue{}
	query = strings.ToLower(query)
	for _, i := range s.data.Issues {
		if (status == "" || i.Status == status) && strings.Contains(strings.ToLower(i.Title+" "+i.Description), query) {
			rows = append(rows, clone(i))
		}
	}
	return rows
}
func (s *Store) Get(id int) (Issue, error) {
	s.mu.Lock()
	defer s.mu.Unlock()
	for _, i := range s.data.Issues {
		if i.ID == id {
			return clone(i), nil
		}
	}
	return Issue{}, ErrMissing
}
func (s *Store) commit(next snapshot) error {
	f, err := os.CreateTemp(filepath.Dir(s.path), ".issues-*")
	if err != nil {
		return err
	}
	defer os.Remove(f.Name())
	defer f.Close()
	if err = json.NewEncoder(f).Encode(next); err != nil {
		return err
	}
	info, err := f.Stat()
	if err != nil {
		return err
	}
	if info.Size() > s.maxBytes {
		return errors.New("local store exceeds its 128 MiB capacity; no changes were committed")
	}
	if err = f.Sync(); err != nil {
		return err
	}
	if err = f.Close(); err != nil {
		return err
	}
	if err = os.Rename(f.Name(), s.path); err != nil {
		return err
	}
	s.data = next
	return nil
}
func (s *Store) Create(in Input) (Issue, error) {
	if in.Priority == "" {
		in.Priority = "normal"
	}
	if in.Status == "" {
		in.Status = "open"
	}
	if err := in.validate(); err != nil {
		return Issue{}, err
	}
	if in.Status != "open" || in.Version != 0 {
		return Issue{}, errors.New("new issues must be open with no version")
	}
	s.mu.Lock()
	defer s.mu.Unlock()
	if len(s.data.Issues) >= 10000 {
		return Issue{}, ErrCapacity
	}
	now := timestamp()
	i := Issue{ID: s.data.NextID, Title: in.Title, Description: in.Description, Priority: in.Priority, Status: "open", Version: 1, CreatedAt: now, UpdatedAt: now, History: []Event{{At: now, Kind: "created", Detail: in.Title}}}
	next := s.data
	next.NextID++
	next.Issues = append(slices.Clone(s.data.Issues), i)
	if err := s.commit(next); err != nil {
		return Issue{}, fmt.Errorf("save issue: %w", err)
	}
	return clone(i), nil
}
func (s *Store) Update(id int, in Input) (Issue, error) {
	if err := in.validate(); err != nil {
		return Issue{}, err
	}
	s.mu.Lock()
	defer s.mu.Unlock()
	for index, old := range s.data.Issues {
		if old.ID != id {
			continue
		}
		if old.Version != in.Version {
			return Issue{}, ErrConflict
		}
		if !allowed(old.Status, in.Status) {
			return Issue{}, ErrTransition
		}
		if len(old.History) >= 1000 {
			return Issue{}, errors.New("issue history limit reached (1000 events)")
		}
		i := clone(old)
		i.Title = in.Title
		i.Description = in.Description
		i.Priority = in.Priority
		i.Status = in.Status
		i.Version++
		i.UpdatedAt = timestamp()
		i.History = append(i.History, Event{At: i.UpdatedAt, Kind: "updated", Detail: old.Status + " → " + i.Status})
		next := s.data
		next.Issues = slices.Clone(s.data.Issues)
		next.Issues[index] = i
		if err := s.commit(next); err != nil {
			return Issue{}, fmt.Errorf("save issue: %w", err)
		}
		return clone(i), nil
	}
	return Issue{}, ErrMissing
}
func (s *Store) Comment(id, version int, body string) (Issue, error) {
	body = strings.TrimSpace(body)
	if n := len([]rune(body)); n < 1 || n > 2000 {
		return Issue{}, errors.New("comment must contain 1–2000 characters")
	}
	s.mu.Lock()
	defer s.mu.Unlock()
	for index, old := range s.data.Issues {
		if old.ID != id {
			continue
		}
		if old.Version != version {
			return Issue{}, ErrConflict
		}
		if len(old.History) >= 1000 {
			return Issue{}, errors.New("issue history limit reached (1000 events)")
		}
		i := clone(old)
		i.Version++
		i.UpdatedAt = timestamp()
		i.History = append(i.History, Event{At: i.UpdatedAt, Kind: "comment", Detail: body})
		next := s.data
		next.Issues = slices.Clone(s.data.Issues)
		next.Issues[index] = i
		if err := s.commit(next); err != nil {
			return Issue{}, fmt.Errorf("save comment: %w", err)
		}
		return clone(i), nil
	}
	return Issue{}, ErrMissing
}
