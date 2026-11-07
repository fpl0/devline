package main

import (
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"errors"
	"flag"
	"fmt"
	"io"
	"io/fs"
	"os"
	"path/filepath"
)

type fileRecord struct {
	Path   string `json:"path"`
	Bytes  int64  `json:"bytes"`
	SHA256 string `json:"sha256"`
}

func main() {
	if err := run(os.Args[1:], os.Stdout, os.Stderr); err != nil {
		fmt.Fprintln(os.Stderr, "file-index:", err)
		os.Exit(1)
	}
}

func run(args []string, out, diagnostics io.Writer) error {
	flags := flag.NewFlagSet("file-index", flag.ContinueOnError)
	flags.SetOutput(diagnostics)
	root := flags.String("root", ".", "directory to index")
	flags.Usage = func() {
		fmt.Fprintln(diagnostics, "Usage: file-index [--root DIRECTORY]\nEmit one JSON record per regular file in lexical path order.")
		flags.PrintDefaults()
	}
	if err := flags.Parse(args); err != nil {
		if errors.Is(err, flag.ErrHelp) {
			return nil
		}
		return err
	}
	if flags.NArg() != 0 {
		return errors.New("unexpected arguments; use --root DIRECTORY")
	}
	return indexFiles(*root, out)
}

func indexFiles(root string, out io.Writer) error {
	info, err := os.Lstat(root)
	if err != nil {
		return err
	}
	if !info.IsDir() {
		return errors.New("root must be a directory")
	}
	encoder := json.NewEncoder(out)
	return filepath.WalkDir(root, func(path string, entry fs.DirEntry, err error) error {
		if err != nil {
			return err
		}
		if !entry.Type().IsRegular() {
			return nil
		}
		relative, err := filepath.Rel(root, path)
		if err != nil {
			return err
		}
		file, err := os.Open(path)
		if err != nil {
			return err
		}
		digest := sha256.New()
		length, readError := io.Copy(digest, file)
		closeError := file.Close()
		if readError != nil || closeError != nil {
			return errors.Join(readError, closeError)
		}
		return encoder.Encode(fileRecord{Path: filepath.ToSlash(relative), Bytes: length, SHA256: hex.EncodeToString(digest.Sum(nil))})
	})
}
