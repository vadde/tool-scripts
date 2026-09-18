// Package main provides REST API handlers for the Session Explorer.
//
// Implements: R-010 (Date filter), R-011 (Workspace filter), R-012 (Search),
//             R-013 (Type filter), R-014 (Sort)
// See: specs/catalog/session-explorer.md
package main

import (
	"encoding/json"
	"log"
	"net/http"
	"strings"
	"time"
)

// APIHandler holds the session index and provides HTTP handlers.
type APIHandler struct {
	index   *SessionIndex
	verbose bool
}

// NewAPIHandler creates a new API handler.
func NewAPIHandler(index *SessionIndex, verbose bool) *APIHandler {
	return &APIHandler{
		index:   index,
		verbose: verbose,
	}
}

// HandleSessions handles GET /api/sessions
// Query params: workspace, sort_by, sort_order, from, to
// Implements: R-008, R-010, R-011, R-014
func (h *APIHandler) HandleSessions(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}

	q := r.URL.Query()
	workspace := q.Get("workspace")
	sortBy := q.Get("sort_by")
	sortOrder := q.Get("sort_order")

	if sortBy == "date_asc" {
		sortBy = "date"
		sortOrder = "asc"
	}
	if sortBy == "" {
		sortBy = "date"
	}
	if sortOrder == "" {
		sortOrder = "desc"
	}

	from := parseDate(q.Get("from"), false)
	to := parseDate(q.Get("to"), true)

	sessions := h.index.GetSessions(workspace, sortBy, sortOrder, from, to)
	stats := h.index.GetStats()

	resp := SessionsResponse{
		Sessions: sessions,
		Stats:    stats,
	}

	writeJSON(w, resp)
}

// HandleSessionDetail handles GET /api/sessions/{id}
// Implements: R-009
func (h *APIHandler) HandleSessionDetail(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}

	// Extract session ID from path: /api/sessions/{id}
	parts := strings.Split(strings.TrimPrefix(r.URL.Path, "/api/sessions/"), "/")
	if len(parts) == 0 || parts[0] == "" {
		http.Error(w, "Session ID required", http.StatusBadRequest)
		return
	}
	sessionID := parts[0]

	detail, err := h.index.GetSessionDetail(sessionID)
	if err != nil {
		if strings.Contains(err.Error(), "not found") {
			http.Error(w, err.Error(), http.StatusNotFound)
		} else {
			http.Error(w, err.Error(), http.StatusInternalServerError)
		}
		return
	}

	writeJSON(w, detail)
}

// HandleSearch handles GET /api/search
// Query params: q, workspace, type, from, to
// Implements: R-012, R-013
func (h *APIHandler) HandleSearch(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}

	q := r.URL.Query()
	query := q.Get("q")
	workspace := q.Get("workspace")
	msgType := q.Get("type")
	from := parseDate(q.Get("from"), false)
	to := parseDate(q.Get("to"), true)

	if query == "" {
		writeJSON(w, SearchResponse{Results: []SearchResult{}, TotalCount: 0, Query: ""})
		return
	}

	results := h.index.Search(query, workspace, msgType, from, to)
	writeJSON(w, results)
}

// HandleStats handles GET /api/stats
// Implements: R-017
func (h *APIHandler) HandleStats(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}
	writeJSON(w, h.index.GetStats())
}

// --- Helpers ---

func writeJSON(w http.ResponseWriter, data interface{}) {
	w.Header().Set("Content-Type", "application/json")
	w.Header().Set("Cache-Control", "no-cache")
	if err := json.NewEncoder(w).Encode(data); err != nil {
		log.Printf("[api] JSON encode error: %v", err)
		http.Error(w, "Internal server error", http.StatusInternalServerError)
	}
}

func parseDate(s string, isEnd bool) time.Time {
	if s == "" {
		return time.Time{}
	}
	s = strings.TrimSpace(s)
	// If date-only string (e.g., "2026-09-18")
	if len(s) == 10 {
		if t, err := time.Parse("2006-01-02", s); err == nil {
			if isEnd {
				// Cover the entire 24 hours of the end day
				return time.Date(t.Year(), t.Month(), t.Day(), 23, 59, 59, int(time.Second-time.Nanosecond), time.UTC)
			}
			return time.Date(t.Year(), t.Month(), t.Day(), 0, 0, 0, 0, time.UTC)
		}
	}

	// Try multiple timestamp formats
	formats := []string{
		time.RFC3339Nano,
		time.RFC3339,
		"2006-01-02T15:04:05Z",
		"2006-01-02T15:04:05",
	}
	for _, f := range formats {
		if t, err := time.Parse(f, s); err == nil {
			return t
		}
	}
	return time.Time{}
}
