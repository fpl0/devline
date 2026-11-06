package main

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
)

func TestHealth(t *testing.T) {
	response := httptest.NewRecorder()
	routes().ServeHTTP(response, httptest.NewRequest(http.MethodGet, "/healthz", nil))
	var body map[string]string
	if err := json.Unmarshal(response.Body.Bytes(), &body); err != nil {
		t.Fatal(err)
	}
	if response.Code != http.StatusOK || body["status"] != "ok" {
		t.Fatalf("unexpected health response: %d %s", response.Code, response.Body)
	}
}
