package main

import (
	"bytes"
	"context"
	"encoding/json"
	"os"
	"path/filepath"
	"testing"
)

func TestSnapshotVerifyWorkflow(t *testing.T) {
	root := t.TempDir()
	manifest := filepath.Join(root, "manifest.json")
	write := func(name, text string) {
		t.Helper()
		if err := os.WriteFile(filepath.Join(root, name), []byte(text), 0600); err != nil {
			t.Fatal(err)
		}
	}
	write("a.txt", "hello")
	write("gone.txt", "old")
	var out, errs bytes.Buffer
	args := []string{"snapshot", "--root", root, "--manifest", manifest, "--workers", "2"}
	if code := run(context.Background(), args, &out, &errs); code != 0 {
		t.Fatalf("%d: %s", code, &errs)
	}
	args[0] = "verify"
	if code := run(context.Background(), args, &out, &errs); code != 0 {
		t.Fatalf("verify: %d %s", code, &errs)
	}
	write("a.txt", "changed")
	write("new.txt", "new")
	os.Remove(filepath.Join(root, "gone.txt"))
	out.Reset()
	if code := run(context.Background(), args, &out, &errs); code != 1 {
		t.Fatalf("wanted differences: %d %s", code, &errs)
	}
	var d Difference
	if err := json.Unmarshal(out.Bytes(), &d); err != nil {
		t.Fatal(err)
	}
	if len(d.Added) != 1 || d.Added[0] != "new.txt" || len(d.Changed) != 1 || len(d.Removed) != 1 {
		t.Fatalf("%+v", d)
	}
}
func TestScanDeterministicAndRejectsLinks(t *testing.T) {
	root := t.TempDir()
	os.WriteFile(filepath.Join(root, "b"), []byte("b"), 0600)
	os.WriteFile(filepath.Join(root, "a"), []byte("a"), 0600)
	rows, err := Scan(context.Background(), root, "unused", 4)
	if err != nil || len(rows) != 2 || rows[0].Path != "a" {
		t.Fatalf("%+v %v", rows, err)
	}
	if rows[0].SHA256 != "ca978112ca1bbdcafac231b39a23dc4da786eff8147c4e72b9807785afee48bb" {
		t.Fatal(rows[0])
	}
	os.Symlink("a", filepath.Join(root, "link"))
	if _, err = Scan(context.Background(), root, "unused", 2); err == nil {
		t.Fatal("accepted symlink")
	}
}
func TestInvalidAndCancelled(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	cancel()
	if _, err := Scan(ctx, t.TempDir(), "unused", 2); err == nil {
		t.Fatal("ignored cancellation")
	}
	if _, err := Scan(context.Background(), t.TempDir(), "unused", 0); err == nil {
		t.Fatal("accepted zero workers")
	}
	if err := (Manifest{Version: 1, Entries: []Entry{{Path: "../escape", SHA256: "bad"}}}).Validate(); err == nil {
		t.Fatal("accepted corrupt manifest")
	}
	var out bytes.Buffer
	if run(context.Background(), []string{"verify", "--manifest", "missing"}, &out, &out) != 2 {
		t.Fatal("missing manifest exit")
	}
}

func TestOversizeManifestPreservesPreviousFile(t *testing.T) {
	name := filepath.Join(t.TempDir(), "inventory.json")
	if err := os.WriteFile(name, []byte("keep me"), 0600); err != nil {
		t.Fatal(err)
	}
	if err := saveManifest(name, Manifest{Version: 1, Entries: []Entry{}}, 5); err == nil {
		t.Fatal("accepted oversized manifest")
	}
	raw, _ := os.ReadFile(name)
	if string(raw) != "keep me" {
		t.Fatal("replaced prior file on failure")
	}
}
