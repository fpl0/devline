package main

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
)

func request(t *testing.T, handler http.Handler, method, path, body string, status int) *httptest.ResponseRecorder {
	t.Helper()
	response := httptest.NewRecorder()
	handler.ServeHTTP(response, httptest.NewRequest(method, path, strings.NewReader(body)))
	if response.Code != status {
		t.Fatalf("%s %s: wanted status %d, got %d: %s", method, path, status, response.Code, response.Body)
	}
	return response
}

func TestTaskLifecycle(t *testing.T) {
	handler := routes()
	request(t, handler, http.MethodGet, "/healthz", "", http.StatusOK)
	created := request(t, handler, http.MethodPost, "/tasks", `{"title":"  Write a test  "}`, http.StatusCreated)
	location := created.Header().Get("Location")
	if location != "/tasks/1" {
		t.Fatalf("unexpected Location: %q", location)
	}
	updated := request(t, handler, http.MethodPut, location, `{"title":"Write a test","completed":true}`, http.StatusOK)
	var item task
	if err := json.Unmarshal(updated.Body.Bytes(), &item); err != nil {
		t.Fatal(err)
	}
	if item.ID != 1 || item.Title != "Write a test" || !item.Completed {
		t.Fatalf("unexpected task: %+v", item)
	}
	request(t, handler, http.MethodGet, location, "", http.StatusOK)
	list := request(t, handler, http.MethodGet, "/tasks", "", http.StatusOK)
	var tasks []task
	if err := json.Unmarshal(list.Body.Bytes(), &tasks); err != nil || len(tasks) != 1 || tasks[0] != item {
		t.Fatalf("unexpected list: %s", list.Body)
	}
	request(t, handler, http.MethodDelete, location, "", http.StatusNoContent)
	request(t, handler, http.MethodGet, location, "", http.StatusNotFound)
}

func TestTaskValidation(t *testing.T) {
	handler := routes()
	tests := []struct {
		body   string
		status int
		code   string
	}{
		{`{"title":" "}`, http.StatusUnprocessableEntity, "invalid_title"},
		{`{"title":"ok","extra":true}`, http.StatusBadRequest, "invalid_json"},
		{`{"title":"ok"} {}`, http.StatusBadRequest, "invalid_json"},
		{`{"title":"` + strings.Repeat("x", 17<<10) + `"}`, http.StatusRequestEntityTooLarge, "body_too_large"},
	}
	for _, test := range tests {
		response := request(t, handler, http.MethodPost, "/tasks", test.body, test.status)
		if !strings.Contains(response.Body.String(), `"code":"`+test.code+`"`) {
			t.Fatalf("expected error %s, got %s", test.code, response.Body)
		}
	}
}
