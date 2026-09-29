// Tests for api.go
// Implements test coverage for R-006, R-008, R-009, R-011, R-012, R-013, R-017
package main

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"
)

func setupTestIndex(t *testing.T) *SessionIndex {
	tmpDir := t.TempDir()

	content := `{"step_index":0,"source":"USER_EXPLICIT","type":"USER_INPUT","status":"DONE","created_at":"2026-09-18T10:00:00Z","content":"<USER_REQUEST>Test prompt for API</USER_REQUEST>\n<ADDITIONAL_METADATA>\nActive Document: /Users/test/projects/demo/main.go\n</ADDITIONAL_METADATA>","tool_calls":[]}
{"step_index":1,"source":"MODEL","type":"PLANNER_RESPONSE","status":"DONE","created_at":"2026-09-18T10:01:00Z","content":"Response content from model","tool_calls":[{"name":"view_file","args":{"AbsolutePath":"/Users/test/projects/demo/main.go"}}]}`

	createMockSession(t, tmpDir, "session-api-1", content)

	idx := NewSessionIndex(tmpDir, false)
	if err := idx.ScanAll(); err != nil {
		t.Fatalf("failed to scan test index: %v", err)
	}
	return idx
}

func TestAPI_HandleSessions(t *testing.T) {
	idx := setupTestIndex(t)
	handler := NewAPIHandler(idx, false)

	req := httptest.NewRequest(http.MethodGet, "/api/sessions", nil)
	rec := httptest.NewRecorder()

	handler.HandleSessions(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("expected status 200, got %d", rec.Code)
	}

	var resp SessionsResponse
	if err := json.NewDecoder(rec.Body).Decode(&resp); err != nil {
		t.Fatalf("failed to decode JSON response: %v", err)
	}

	if len(resp.Sessions) != 1 {
		t.Errorf("expected 1 session, got %d", len(resp.Sessions))
	}
	if resp.Sessions[0].ID != "session-api-1" {
		t.Errorf("expected session-api-1, got %s", resp.Sessions[0].ID)
	}
}

func TestAPI_HandleSessionDetail(t *testing.T) {
	idx := setupTestIndex(t)
	handler := NewAPIHandler(idx, false)

	req := httptest.NewRequest(http.MethodGet, "/api/sessions/session-api-1", nil)
	rec := httptest.NewRecorder()

	handler.HandleSessionDetail(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("expected status 200, got %d", rec.Code)
	}

	var resp SessionDetailResponse
	if err := json.NewDecoder(rec.Body).Decode(&resp); err != nil {
		t.Fatalf("failed to decode JSON response: %v", err)
	}

	if resp.Session.ID != "session-api-1" {
		t.Errorf("expected session-api-1, got %s", resp.Session.ID)
	}
	if len(resp.Messages) != 2 {
		t.Errorf("expected 2 messages, got %d", len(resp.Messages))
	}
}

func TestAPI_HandleSearch(t *testing.T) {
	idx := setupTestIndex(t)
	handler := NewAPIHandler(idx, false)

	req := httptest.NewRequest(http.MethodGet, "/api/search?q=prompt", nil)
	rec := httptest.NewRecorder()

	handler.HandleSearch(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("expected status 200, got %d", rec.Code)
	}

	var resp SearchResponse
	if err := json.NewDecoder(rec.Body).Decode(&resp); err != nil {
		t.Fatalf("failed to decode search response: %v", err)
	}

	if resp.TotalCount != 1 {
		t.Errorf("expected 1 match, got %d", resp.TotalCount)
	}
}

func TestAPI_HandleStats(t *testing.T) {
	idx := setupTestIndex(t)
	handler := NewAPIHandler(idx, false)

	req := httptest.NewRequest(http.MethodGet, "/api/stats", nil)
	rec := httptest.NewRecorder()

	handler.HandleStats(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("expected status 200, got %d", rec.Code)
	}

	var stats Stats
	if err := json.NewDecoder(rec.Body).Decode(&stats); err != nil {
		t.Fatalf("failed to decode stats: %v", err)
	}

	if stats.TotalSessions != 1 {
		t.Errorf("expected 1 session in stats, got %d", stats.TotalSessions)
	}
	if len(stats.Projects) == 0 {
		t.Errorf("expected stats.Projects to be populated, got empty")
	} else if stats.Projects[0].Name != "demo" {
		t.Errorf("expected project demo, got %s", stats.Projects[0].Name)
	}
}

func TestAPI_HandleSessions_DateFilterToday(t *testing.T) {
	tmpDir := t.TempDir()

	// Session created on 2026-09-18 at 14:30:00 UTC (after midnight UTC)
	contentToday := `{"step_index":0,"source":"USER_EXPLICIT","type":"USER_INPUT","status":"DONE","created_at":"2026-09-18T14:30:00Z","content":"<USER_REQUEST>Today prompt</USER_REQUEST>"}
{"step_index":1,"source":"MODEL","type":"PLANNER_RESPONSE","status":"DONE","created_at":"2026-09-18T14:31:00Z","content":"Today response"}`
	createMockSession(t, tmpDir, "session-today", contentToday)

	// Session created on 2026-09-10
	contentPast := `{"step_index":0,"source":"USER_EXPLICIT","type":"USER_INPUT","status":"DONE","created_at":"2026-09-10T10:00:00Z","content":"<USER_REQUEST>Past prompt</USER_REQUEST>"}`
	createMockSession(t, tmpDir, "session-past", contentPast)

	idx := NewSessionIndex(tmpDir, false)
	if err := idx.ScanAll(); err != nil {
		t.Fatalf("failed to scan test index: %v", err)
	}

	handler := NewAPIHandler(idx, false)

	// Query from=2026-09-18&to=2026-09-18 (equivalent to "Today" preset)
	req := httptest.NewRequest(http.MethodGet, "/api/sessions?from=2026-09-18&to=2026-09-18", nil)
	rec := httptest.NewRecorder()
	handler.HandleSessions(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("expected status 200, got %d", rec.Code)
	}

	var resp SessionsResponse
	if err := json.NewDecoder(rec.Body).Decode(&resp); err != nil {
		t.Fatalf("failed to decode response: %v", err)
	}

	if len(resp.Sessions) != 1 {
		t.Fatalf("expected 1 session for Today date filter, got %d", len(resp.Sessions))
	}
	if resp.Sessions[0].ID != "session-today" {
		t.Errorf("expected session-today, got %s", resp.Sessions[0].ID)
	}
}

func TestAPI_HandleSessions_SortDateAsc(t *testing.T) {
	tmpDir := t.TempDir()

	content1 := `{"step_index":0,"source":"USER_EXPLICIT","type":"USER_INPUT","status":"DONE","created_at":"2026-09-10T10:00:00Z","content":"<USER_REQUEST>First</USER_REQUEST>"}`
	createMockSession(t, tmpDir, "session-1", content1)

	content2 := `{"step_index":0,"source":"USER_EXPLICIT","type":"USER_INPUT","status":"DONE","created_at":"2026-09-18T10:00:00Z","content":"<USER_REQUEST>Second</USER_REQUEST>"}`
	createMockSession(t, tmpDir, "session-2", content2)

	idx := NewSessionIndex(tmpDir, false)
	if err := idx.ScanAll(); err != nil {
		t.Fatalf("failed to scan test index: %v", err)
	}

	handler := NewAPIHandler(idx, false)

	// Query sort_by=date_asc (oldest first)
	req := httptest.NewRequest(http.MethodGet, "/api/sessions?sort_by=date_asc", nil)
	rec := httptest.NewRecorder()
	handler.HandleSessions(rec, req)

	var resp SessionsResponse
	if err := json.NewDecoder(rec.Body).Decode(&resp); err != nil {
		t.Fatalf("failed to decode response: %v", err)
	}

	if len(resp.Sessions) != 2 {
		t.Fatalf("expected 2 sessions, got %d", len(resp.Sessions))
	}
	if resp.Sessions[0].ID != "session-1" {
		t.Errorf("expected oldest session-1 first with date_asc, got %s", resp.Sessions[0].ID)
	}
}

func TestAPI_HandleRefresh(t *testing.T) {
	tmpDir := t.TempDir()

	content1 := `{"step_index":0,"source":"USER_EXPLICIT","type":"USER_INPUT","status":"DONE","created_at":"2026-09-10T10:00:00Z","content":"<USER_REQUEST>Initial</USER_REQUEST>"}`
	createMockSession(t, tmpDir, "session-1", content1)

	idx := NewSessionIndex(tmpDir, false)
	if err := idx.ScanAll(); err != nil {
		t.Fatalf("failed to scan test index: %v", err)
	}

	handler := NewAPIHandler(idx, false)

	// Verify initial count is 1
	if len(idx.GetSessions("", "date", "desc", time.Time{}, time.Time{})) != 1 {
		t.Fatalf("expected 1 session initially")
	}

	// Add second session to disk while server is running
	content2 := `{"step_index":0,"source":"USER_EXPLICIT","type":"USER_INPUT","status":"DONE","created_at":"2026-09-18T10:00:00Z","content":"<USER_REQUEST>Added later</USER_REQUEST>"}`
	createMockSession(t, tmpDir, "session-2", content2)

	// Hit /api/refresh
	req := httptest.NewRequest(http.MethodPost, "/api/refresh", nil)
	rec := httptest.NewRecorder()
	handler.HandleRefresh(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("expected status 200, got %d", rec.Code)
	}

	// Verify index now contains both sessions
	sessions := idx.GetSessions("", "date", "desc", time.Time{}, time.Time{})
	if len(sessions) != 2 {
		t.Fatalf("expected 2 sessions after refresh, got %d", len(sessions))
	}
}

