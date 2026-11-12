package main

import (
	"errors"
	"strings"
	"time"
	"unicode/utf8"
)

var ErrMissing = errors.New("issue not found")
var ErrConflict = errors.New("version conflict; reload the issue before editing")
var ErrTransition = errors.New("allowed transitions: open → doing → done → open")
var ErrCapacity = errors.New("this local desk supports up to 10000 issues")

type Event struct {
	At     string `json:"at"`
	Kind   string `json:"kind"`
	Detail string `json:"detail"`
}
type Issue struct {
	ID          int     `json:"id"`
	Title       string  `json:"title"`
	Description string  `json:"description"`
	Priority    string  `json:"priority"`
	Status      string  `json:"status"`
	Version     int     `json:"version"`
	CreatedAt   string  `json:"created_at"`
	UpdatedAt   string  `json:"updated_at"`
	History     []Event `json:"history"`
}
type Input struct {
	Title       string `json:"title"`
	Description string `json:"description"`
	Priority    string `json:"priority"`
	Status      string `json:"status"`
	Version     int    `json:"version"`
}

func (in *Input) validate() error {
	in.Title = strings.TrimSpace(in.Title)
	in.Description = strings.TrimSpace(in.Description)
	if n := utf8.RuneCountInString(in.Title); n < 1 || n > 160 {
		return errors.New("title must contain 1–160 characters")
	}
	if utf8.RuneCountInString(in.Description) > 8000 {
		return errors.New("description must contain at most 8000 characters")
	}
	if in.Priority != "low" && in.Priority != "normal" && in.Priority != "high" {
		return errors.New("priority must be low, normal, or high")
	}
	if in.Status != "open" && in.Status != "doing" && in.Status != "done" {
		return errors.New("status must be open, doing, or done")
	}
	return nil
}
func timestamp() string { return time.Now().UTC().Format(time.RFC3339Nano) }
func allowed(from, to string) bool {
	return from == to || (from == "open" && to == "doing") || (from == "doing" && to == "done") || (from == "done" && to == "open")
}
