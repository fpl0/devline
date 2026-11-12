package main

import (
	"context"
	"crypto/sha256"
	"encoding/hex"
	"errors"
	"fmt"
	"io"
	"io/fs"
	"os"
	"path"
	"path/filepath"
	"slices"
	"strings"
	"sync"
)

type Entry struct {
	Path   string `json:"path"`
	Bytes  int64  `json:"bytes"`
	SHA256 string `json:"sha256"`
}
type Manifest struct {
	Version int     `json:"version"`
	Entries []Entry `json:"entries"`
}
type Difference struct {
	Added   []string `json:"added"`
	Changed []string `json:"changed"`
	Removed []string `json:"removed"`
}

func (m Manifest) Validate() error {
	if m.Version != 1 || m.Entries == nil {
		return errors.New("unsupported or incomplete manifest")
	}
	seen := map[string]bool{}
	for _, e := range m.Entries {
		digest, err := hex.DecodeString(e.SHA256)
		if !fs.ValidPath(e.Path) || e.Path == "." || strings.Contains(e.Path, "\\") || path.Clean(e.Path) != e.Path || seen[e.Path] || e.Bytes < 0 || err != nil || len(digest) != 32 {
			return fmt.Errorf("invalid or duplicate manifest entry %q", e.Path)
		}
		seen[e.Path] = true
	}
	return nil
}

// Scan hashes ordinary files using a bounded worker pool. Symlinks are rejected,
// so a snapshot cannot silently include files outside its selected directory.
func Scan(ctx context.Context, root, excluded string, workers int) ([]Entry, error) {
	if workers < 1 || workers > 32 {
		return nil, errors.New("workers must be between 1 and 32")
	}
	root, err := filepath.Abs(root)
	if err != nil {
		return nil, err
	}
	excluded, err = filepath.Abs(excluded)
	if err != nil {
		return nil, err
	}
	info, err := os.Lstat(root)
	if err != nil {
		return nil, err
	}
	if !info.IsDir() {
		return nil, errors.New("root must be an ordinary directory")
	}
	ctx, cancel := context.WithCancel(ctx)
	defer cancel()
	jobs := make(chan string)
	results := make(chan Entry)
	failures := make(chan error, 1)
	fail := func(err error) {
		select {
		case failures <- err:
		default:
		}
		cancel()
	}
	var group sync.WaitGroup
	for range workers {
		group.Go(func() {
			for name := range jobs {
				if ctx.Err() != nil {
					return
				}
				entry, err := hashFile(root, name)
				if err != nil {
					fail(err)
					return
				}
				select {
				case results <- entry:
				case <-ctx.Done():
					return
				}
			}
		})
	}
	walkDone := make(chan struct{})
	go func() {
		defer close(walkDone)
		defer close(jobs)
		err := filepath.WalkDir(root, func(name string, d fs.DirEntry, err error) error {
			if err != nil {
				return err
			}
			if ctx.Err() != nil {
				return ctx.Err()
			}
			if name == excluded {
				return nil
			}
			if d.IsDir() {
				return nil
			}
			if !d.Type().IsRegular() {
				return fmt.Errorf("unsupported file type: %s", name)
			}
			select {
			case jobs <- name:
				return nil
			case <-ctx.Done():
				return ctx.Err()
			}
		})
		if err != nil {
			fail(err)
		}
	}()
	go func() { group.Wait(); close(results) }()
	entries := []Entry{}
	for e := range results {
		entries = append(entries, e)
	}
	<-walkDone
	select {
	case err := <-failures:
		return nil, err
	default:
	}
	if err = ctx.Err(); err != nil {
		return nil, err
	}
	slices.SortFunc(entries, func(a, b Entry) int { return strings.Compare(a.Path, b.Path) })
	return entries, nil
}
func hashFile(root, name string) (Entry, error) {
	f, err := os.Open(name)
	if err != nil {
		return Entry{}, err
	}
	defer f.Close()
	before, err := f.Stat()
	if err != nil {
		return Entry{}, err
	}
	if !before.Mode().IsRegular() {
		return Entry{}, fmt.Errorf("not a regular file: %s", name)
	}
	digest := sha256.New()
	size, err := io.Copy(digest, f)
	if err != nil {
		return Entry{}, err
	}
	after, err := f.Stat()
	if err != nil {
		return Entry{}, err
	}
	if size != before.Size() || before.Size() != after.Size() || !before.ModTime().Equal(after.ModTime()) {
		return Entry{}, fmt.Errorf("file changed while hashing: %s", name)
	}
	relative, err := filepath.Rel(root, name)
	if err != nil {
		return Entry{}, err
	}
	return Entry{Path: filepath.ToSlash(relative), Bytes: size, SHA256: hex.EncodeToString(digest.Sum(nil))}, nil
}
func Compare(before, after []Entry) Difference {
	result := Difference{Added: []string{}, Changed: []string{}, Removed: []string{}}
	old := map[string]Entry{}
	for _, e := range before {
		old[e.Path] = e
	}
	for _, e := range after {
		previous, ok := old[e.Path]
		if !ok {
			result.Added = append(result.Added, e.Path)
		} else if previous.Bytes != e.Bytes || previous.SHA256 != e.SHA256 {
			result.Changed = append(result.Changed, e.Path)
		}
		delete(old, e.Path)
	}
	for name := range old {
		result.Removed = append(result.Removed, name)
	}
	slices.Sort(result.Added)
	slices.Sort(result.Changed)
	slices.Sort(result.Removed)
	return result
}
