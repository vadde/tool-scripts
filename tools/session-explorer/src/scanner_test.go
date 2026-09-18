// Tests for scanner.go
// Implements test coverage for R-001, R-002, R-003, R-004, R-005, R-010, R-011, R-012, R-014, R-017, NF-007
package main

import (
	"os"
	"path/filepath"
	"testing"
	"time"
)

// Helper to create a mock brain session directory
func createMockSession(t *testing.T, baseDir, sessionID, content string) {
	t.Helper()
	logDir := filepath.Join(baseDir, sessionID, ".system_generated", "logs")
	if err := os.MkdirAll(logDir, 0755); err != nil {
		t.Fatalf("failed to create mock log dir: %v", err)
	}

	filePath := filepath.Join(logDir, "transcript.jsonl")
	if err := os.WriteFile(filePath, []byte(content), 0644); err != nil {
		t.Fatalf("failed to write mock transcript: %v", err)
	}
}

// TestR001_SessionDiscovery tests session directory discovery
func TestR001_SessionDiscovery(t *testing.T) {
	tmpDir := t.TempDir()

	mockJSONL := `{"step_index":0,"source":"USER_EXPLICIT","type":"USER_INPUT","status":"DONE","created_at":"2026-09-18T10:00:00Z","content":"<USER_REQUEST>\nHello world test\n</USER_REQUEST>\n<ADDITIONAL_METADATA>\nActive Document: /Users/test/workspace/main.go\n</ADDITIONAL_METADATA>"}
{"step_index":1,"source":"MODEL","type":"PLANNER_RESPONSE","status":"DONE","created_at":"2026-09-18T10:01:00Z","content":"Here is the response","tool_calls":[{"name":"view_file","args":{"AbsolutePath":"/Users/test/workspace/main.go"}}]}`

	createMockSession(t, tmpDir, "session-abc-123", mockJSONL)
	createMockSession(t, tmpDir, "session-def-456", mockJSONL)

	idx := NewSessionIndex(tmpDir, false)
	if err := idx.ScanAll(); err != nil {
		t.Fatalf("ScanAll failed: %v", err)
	}

	if len(idx.sessions) != 2 {
		t.Errorf("expected 2 sessions, got %d", len(idx.sessions))
	}
}

// TestR002_ParseJSONL tests JSONL parsing and NF-007 malformed handling
func TestR002_ParseJSONL(t *testing.T) {
	tmpDir := t.TempDir()

	content := `{"step_index":0,"source":"USER_EXPLICIT","type":"USER_INPUT","status":"DONE","created_at":"2026-09-18T12:00:00Z","content":"<USER_REQUEST>First prompt</USER_REQUEST>"}
not-a-valid-json-line
{"step_index":1,"source":"MODEL","type":"PLANNER_RESPONSE","status":"DONE","created_at":"2026-09-18T12:01:00Z","content":"Response 1"}
{"step_index":2,"source":"SYSTEM","type":"ERROR_MESSAGE","status":"ERROR","created_at":"2026-09-18T12:02:00Z","content":"Something went wrong"}`

	createMockSession(t, tmpDir, "session-parse-test", content)

	idx := NewSessionIndex(tmpDir, false)
	if err := idx.ScanAll(); err != nil {
		t.Fatalf("ScanAll failed: %v", err)
	}

	if len(idx.sessions) != 1 {
		t.Fatalf("expected 1 session, got %d", len(idx.sessions))
	}

	s := idx.sessions[0]
	if s.StepCount != 3 {
		t.Errorf("expected 3 valid steps, got %d", s.StepCount)
	}
	if s.UserMessageCount != 1 {
		t.Errorf("expected 1 user message, got %d", s.UserMessageCount)
	}
	if s.AgentResponseCount != 1 {
		t.Errorf("expected 1 agent response, got %d", s.AgentResponseCount)
	}
	if s.ErrorCount != 1 {
		t.Errorf("expected 1 error count, got %d", s.ErrorCount)
	}
}

// TestR003_ExtractUserPrompt tests stripping metadata from prompts
func TestR003_ExtractUserPrompt(t *testing.T) {
	raw := `<USER_REQUEST>
Build a distributed cache in Go
</USER_REQUEST>
<ADDITIONAL_METADATA>
The current local time is: 2026-09-18T10:00:00.
Active Document: /foo/bar.go
</ADDITIONAL_METADATA>`

	extracted := extractUserPrompt(raw)
	expected := "Build a distributed cache in Go"
	if extracted != expected {
		t.Errorf("expected %q, got %q", expected, extracted)
	}
}

// TestR005_ExtractWorkspace tests workspace extraction patterns
func TestR005_ExtractWorkspace(t *testing.T) {
	contentWithDoc := `Active Document: /Users/test/projects/my-cool-app/src/main.go`
	ws := extractWorkspace(contentWithDoc)
	if ws != "/Users/test/projects/my-cool-app/src" {
		t.Errorf("expected /Users/test/projects/my-cool-app/src, got %q", ws)
	}

	proj := extractProjectName(ws)
	if proj != "my-cool-app" {
		t.Errorf("expected my-cool-app, got %q", proj)
	}
}

// TestR010_FilterDateRange tests filtering sessions by date
func TestR010_FilterDateRange(t *testing.T) {
	tmpDir := t.TempDir()

	s1 := `{"step_index":0,"source":"USER_EXPLICIT","type":"USER_INPUT","status":"DONE","created_at":"2026-09-10T10:00:00Z","content":"<USER_REQUEST>Prompt 1</USER_REQUEST>"}`
	s2 := `{"step_index":0,"source":"USER_EXPLICIT","type":"USER_INPUT","status":"DONE","created_at":"2026-09-15T10:00:00Z","content":"<USER_REQUEST>Prompt 2</USER_REQUEST>"}`
	s3 := `{"step_index":0,"source":"USER_EXPLICIT","type":"USER_INPUT","status":"DONE","created_at":"2026-09-20T10:00:00Z","content":"<USER_REQUEST>Prompt 3</USER_REQUEST>"}`

	createMockSession(t, tmpDir, "sess-1", s1)
	createMockSession(t, tmpDir, "sess-2", s2)
	createMockSession(t, tmpDir, "sess-3", s3)

	idx := NewSessionIndex(tmpDir, false)
	if err := idx.ScanAll(); err != nil {
		t.Fatalf("ScanAll failed: %v", err)
	}

	from, _ := time.Parse(time.RFC3339, "2026-09-12T00:00:00Z")
	to, _ := time.Parse(time.RFC3339, "2026-09-18T00:00:00Z")

	filtered := idx.GetSessions("", "date", "desc", from, to)
	if len(filtered) != 1 {
		t.Fatalf("expected 1 session in date range, got %d", len(filtered))
	}
	if filtered[0].ID != "sess-2" {
		t.Errorf("expected sess-2, got %s", filtered[0].ID)
	}
}

// TestR012_Search tests full-text search across transcripts
func TestR012_Search(t *testing.T) {
	tmpDir := t.TempDir()

	s1 := `{"step_index":0,"source":"USER_EXPLICIT","type":"USER_INPUT","status":"DONE","created_at":"2026-09-18T10:00:00Z","content":"<USER_REQUEST>Implement Kubernetes ingress controller</USER_REQUEST>"}`
	s2 := `{"step_index":0,"source":"USER_EXPLICIT","type":"USER_INPUT","status":"DONE","created_at":"2026-09-18T11:00:00Z","content":"<USER_REQUEST>Write terraform modules for AWS VPC</USER_REQUEST>"}`

	createMockSession(t, tmpDir, "sess-k8s", s1)
	createMockSession(t, tmpDir, "sess-tf", s2)

	idx := NewSessionIndex(tmpDir, false)
	if err := idx.ScanAll(); err != nil {
		t.Fatalf("ScanAll failed: %v", err)
	}

	res := idx.Search("kubernetes", "", "", time.Time{}, time.Time{})
	if res.TotalCount != 1 {
		t.Fatalf("expected 1 search result, got %d", res.TotalCount)
	}
	if res.Results[0].SessionID != "sess-k8s" {
		t.Errorf("expected sess-k8s, got %s", res.Results[0].SessionID)
	}
}

// TestR017_Stats tests aggregate stats computation
func TestR017_Stats(t *testing.T) {
	tmpDir := t.TempDir()

	s1 := `{"step_index":0,"source":"USER_EXPLICIT","type":"USER_INPUT","status":"DONE","created_at":"2026-09-18T10:00:00Z","content":"<USER_REQUEST>Fix bug</USER_REQUEST>"}
{"step_index":1,"source":"MODEL","type":"PLANNER_RESPONSE","status":"DONE","created_at":"2026-09-18T10:01:00Z","content":"Fixed","tool_calls":[{"name":"replace_file_content","args":{}},{"name":"view_file","args":{}}]}`

	createMockSession(t, tmpDir, "sess-stats", s1)

	idx := NewSessionIndex(tmpDir, false)
	if err := idx.ScanAll(); err != nil {
		t.Fatalf("ScanAll failed: %v", err)
	}

	stats := idx.GetStats()
	if stats.TotalSessions != 1 {
		t.Errorf("expected 1 total session, got %d", stats.TotalSessions)
	}
	if stats.TotalUserMessages != 1 {
		t.Errorf("expected 1 user message, got %d", stats.TotalUserMessages)
	}
	if stats.TotalAgentResponses != 1 {
		t.Errorf("expected 1 agent response, got %d", stats.TotalAgentResponses)
	}
	if stats.TotalToolCalls != 2 {
		t.Errorf("expected 2 tool calls, got %d", stats.TotalToolCalls)
	}
	if len(stats.TopToolCalls) != 2 {
		t.Errorf("expected 2 top tools, got %d", len(stats.TopToolCalls))
	}
}

// TestR028_ProjectClustering tests repository and project clustering from tool calls and metadata
func TestR028_ProjectClustering(t *testing.T) {
	tmpDir := t.TempDir()

	// Mock session where user has unrelated active doc, but agent performs tool calls inside tutor-intelligence
	sTutor := `{"step_index":0,"source":"USER_EXPLICIT","type":"USER_INPUT","status":"DONE","created_at":"2026-09-18T10:00:00Z","content":"<USER_REQUEST>\nLet's implement the robot puzzle\n</USER_REQUEST>\n<ADDITIONAL_METADATA>\nActive Document: /Users/test/knowledge/vadde.github.io/README.md\n</ADDITIONAL_METADATA>"}
{"step_index":1,"source":"MODEL","type":"PLANNER_RESPONSE","status":"DONE","created_at":"2026-09-18T10:01:00Z","content":"Inspecting files","tool_calls":[{"name":"view_file","args":{"AbsolutePath":"/Users/test/knowledge/tutor-intelligence/src/robot.py"}},{"name":"run_command","args":{"Cwd":"/Users/test/knowledge/tutor-intelligence","CommandLine":"pytest"}}]}`

	// Mock session for tool-scripts
	sTools := `{"step_index":0,"source":"USER_EXPLICIT","type":"USER_INPUT","status":"DONE","created_at":"2026-09-18T11:00:00Z","content":"<USER_REQUEST>\nBuild the session explorer tool\n</USER_REQUEST>"}
{"step_index":1,"source":"MODEL","type":"PLANNER_RESPONSE","status":"DONE","created_at":"2026-09-18T11:01:00Z","content":"Writing code","tool_calls":[{"name":"write_to_file","args":{"TargetFile":"/Users/test/knowledge/tool-scripts/tools/session-explorer/src/main.go"}}]}`

	createMockSession(t, tmpDir, "sess-tutor", sTutor)
	createMockSession(t, tmpDir, "sess-tools", sTools)

	idx := NewSessionIndex(tmpDir, false)
	// Register test known projects
	idx.knownProjects["tutor-intelligence"] = "/Users/test/knowledge/tutor-intelligence"
	idx.knownProjects["tool-scripts"] = "/Users/test/knowledge/tool-scripts"
	idx.knownProjects["vadde.github.io"] = "/Users/test/knowledge/vadde.github.io"

	if err := idx.ScanAll(); err != nil {
		t.Fatalf("ScanAll failed: %v", err)
	}

	if len(idx.sessions) != 2 {
		t.Fatalf("expected 2 sessions, got %d", len(idx.sessions))
	}

	// Verify tutor-intelligence session was accurately assigned
	var tutorSess, toolsSess *Session
	for i := range idx.sessions {
		if idx.sessions[i].ID == "sess-tutor" {
			tutorSess = &idx.sessions[i]
		} else if idx.sessions[i].ID == "sess-tools" {
			toolsSess = &idx.sessions[i]
		}
	}

	if tutorSess == nil {
		t.Fatal("sess-tutor not found")
	}
	if tutorSess.ProjectName != "tutor-intelligence" {
		t.Errorf("expected ProjectName tutor-intelligence, got %q", tutorSess.ProjectName)
	}
	if len(tutorSess.ProjectsTouched) == 0 || tutorSess.ProjectsTouched[0] != "tutor-intelligence" {
		t.Errorf("expected ProjectsTouched to have tutor-intelligence first, got %v", tutorSess.ProjectsTouched)
	}

	if toolsSess == nil {
		t.Fatal("sess-tools not found")
	}
	if toolsSess.ProjectName != "tool-scripts" {
		t.Errorf("expected ProjectName tool-scripts, got %q", toolsSess.ProjectName)
	}

	// Verify stats.Projects contains tutor-intelligence and tool-scripts
	stats := idx.GetStats()
	if len(stats.Projects) == 0 {
		t.Fatal("expected stats.Projects to be populated")
	}

	foundTutor := false
	for _, p := range stats.Projects {
		if p.Name == "tutor-intelligence" {
			foundTutor = true
			if p.SessionCount != 1 {
				t.Errorf("expected 1 session for tutor-intelligence, got %d", p.SessionCount)
			}
			if len(p.RecentPrompts) != 1 || p.RecentPrompts[0] != "Let's implement the robot puzzle" {
				t.Errorf("unexpected recent prompts for tutor-intelligence: %v", p.RecentPrompts)
			}
		}
	}
	if !foundTutor {
		t.Error("tutor-intelligence project not found in stats.Projects")
	}
}
