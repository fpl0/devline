package main

import (
	"bytes"
	"encoding/json"
	"io"
	"os"
	"path/filepath"
	"testing"
)

func TestFileIndexHashesInOrderAndSkipsSymlink(t *testing.T) {
	root := t.TempDir()
	for _, name := range []string{"b.txt", "a.txt"} {
		if err := os.WriteFile(filepath.Join(root, name), []byte("abc"), 0600); err != nil {
			t.Fatal(err)
		}
	}
	if err := os.Symlink("a.txt", filepath.Join(root, "link")); err != nil {
		t.Fatal(err)
	}
	var stdout, stderr bytes.Buffer
	if err := run([]string{"--root", root}, &stdout, &stderr); err != nil {
		t.Fatal(err)
	}
	if stderr.Len() != 0 {
		t.Fatalf("unexpected diagnostics: %s", &stderr)
	}
	decoder := json.NewDecoder(&stdout)
	for _, want := range []string{"a.txt", "b.txt"} {
		var item fileRecord
		if err := decoder.Decode(&item); err != nil {
			t.Fatal(err)
		}
		if item.Path != want || item.Bytes != 3 || item.SHA256 != "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad" {
			t.Fatalf("unexpected record: %#v", item)
		}
	}
	if err := decoder.Decode(new(fileRecord)); err != io.EOF {
		t.Fatal("expected exactly two records", err)
	}
}

func TestFileIndexRefusesNonDirectory(t *testing.T) {
	file := filepath.Join(t.TempDir(), "file")
	if err := os.WriteFile(file, nil, 0600); err != nil {
		t.Fatal(err)
	}
	if err := indexFiles(file, io.Discard); err == nil {
		t.Fatal("accepted a non-directory root")
	}
}
