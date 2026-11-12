package main

import (
	"context"
	"encoding/json"
	"errors"
	"flag"
	"fmt"
	"io"
	"os"
	"os/signal"
	"path/filepath"
)

func run(ctx context.Context, args []string, out, stderr io.Writer) int {
	if len(args) == 0 {
		fmt.Fprintln(stderr, "Usage: file-vault snapshot|verify --root DIRECTORY --manifest FILE [--workers 4]")
		return 2
	}
	command := args[0]
	if command != "snapshot" && command != "verify" {
		fmt.Fprintln(stderr, "Choose snapshot or verify.")
		return 2
	}
	flags := flag.NewFlagSet(command, flag.ContinueOnError)
	flags.SetOutput(stderr)
	root := flags.String("root", ".", "directory to inspect")
	name := flags.String("manifest", "inventory.json", "manifest file (excluded from the inventory)")
	workers := flags.Int("workers", 4, "concurrent hashing workers (1–32)")
	if err := flags.Parse(args[1:]); err != nil {
		return 2
	}
	if flags.NArg() != 0 {
		fmt.Fprintln(stderr, "Unexpected positional arguments.")
		return 2
	}
	fail := func(err error) int { fmt.Fprintln(stderr, "file-vault:", err); return 2 }
	if command == "snapshot" {
		entries, err := Scan(ctx, *root, *name, *workers)
		if err != nil {
			return fail(err)
		}
		manifest := Manifest{Version: 1, Entries: entries}
		if err = Save(*name, manifest); err != nil {
			return fail(err)
		}
		fmt.Fprintf(out, "Saved %d files to %s\n", len(entries), *name)
		return 0
	}
	manifest, err := Load(*name)
	if err != nil {
		return fail(err)
	}
	entries, err := Scan(ctx, *root, *name, *workers)
	if err != nil {
		return fail(err)
	}
	result := Compare(manifest.Entries, entries)
	if err = json.NewEncoder(out).Encode(result); err != nil {
		return fail(err)
	}
	if len(result.Added)+len(result.Changed)+len(result.Removed) > 0 {
		return 1
	}
	return 0
}

func main() {
	ctx, stop := signal.NotifyContext(context.Background(), os.Interrupt)
	defer stop()
	os.Exit(run(ctx, os.Args[1:], os.Stdout, os.Stderr))
}

const maxManifestBytes int64 = 64 << 20

func Save(name string, value Manifest) error { return saveManifest(name, value, maxManifestBytes) }
func saveManifest(name string, value Manifest, limit int64) error {
	if err := os.MkdirAll(filepath.Dir(name), 0700); err != nil {
		return err
	}
	file, err := os.CreateTemp(filepath.Dir(name), ".inventory-*")
	if err != nil {
		return err
	}
	defer os.Remove(file.Name())
	defer file.Close()
	encoder := json.NewEncoder(file)
	encoder.SetIndent("", "  ")
	if err = encoder.Encode(value); err != nil {
		return err
	}
	info, err := file.Stat()
	if err != nil {
		return err
	}
	if info.Size() > limit {
		return errors.New("manifest exceeds the 64 MiB capacity; previous file was preserved")
	}
	if err = file.Sync(); err != nil {
		return err
	}
	if err = file.Close(); err != nil {
		return err
	}
	return os.Rename(file.Name(), name)
}
func Load(name string) (Manifest, error) {
	file, err := os.Open(name)
	if err != nil {
		return Manifest{}, err
	}
	defer file.Close()
	var m Manifest
	decoder := json.NewDecoder(io.LimitReader(file, maxManifestBytes))
	decoder.DisallowUnknownFields()
	if err = decoder.Decode(&m); err != nil {
		return m, fmt.Errorf("read manifest: %w", err)
	}
	if err = decoder.Decode(new(any)); err != io.EOF {
		return m, errors.New("manifest must contain one JSON document")
	}
	if err = m.Validate(); err != nil {
		return m, err
	}
	return m, nil
}
