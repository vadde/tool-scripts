// Package main provides data types for the Session Explorer tool.
//
// Implements: R-002 (Parse JSONL transcript files)
// See: specs/catalog/session-explorer.md#R-002
package main

import (
	"time"
)

// TranscriptEntry represents a single line from a JSONL transcript file.
// Implements: R-002
type TranscriptEntry struct {
	StepIndex   int        `json:"step_index"`
	Source      string     `json:"source"`       // USER_EXPLICIT, MODEL, SYSTEM
	Type        string     `json:"type"`         // USER_INPUT, PLANNER_RESPONSE, etc.
	Status      string     `json:"status"`       // DONE, ERROR
	CreatedAt   time.Time  `json:"created_at"`
	Content     string     `json:"content,omitempty"`
	Thinking    string     `json:"thinking,omitempty"`
	ToolCalls   []ToolCall `json:"tool_calls,omitempty"`
	IsTruncated bool       `json:"is_truncated,omitempty"`
}

// ToolCall represents a tool invocation within a PLANNER_RESPONSE.
type ToolCall struct {
	Name string                 `json:"name"`
	Args map[string]interface{} `json:"args,omitempty"`
}

// Session represents a discovered conversation session with computed metadata.
// Implements: R-001, R-008
type Session struct {
	ID                     string    `json:"id"`
	CreatedAt              time.Time `json:"created_at"`
	LastMessageAt          time.Time `json:"last_message_at"`
	Workspace              string    `json:"workspace"`
	ProjectName            string         `json:"project_name"`
	ProjectsTouched        []string       `json:"projects_touched,omitempty"`
	StepCount              int            `json:"step_count"`
	UserMessageCount       int            `json:"user_message_count"`
	AgentResponseCount     int            `json:"agent_response_count"`
	ToolCallCount          int            `json:"tool_call_count"`
	ToolCallsMap           map[string]int `json:"tool_calls_map,omitempty"`
	ErrorCount             int            `json:"error_count"`
	TranscriptSizeBytes    int64          `json:"transcript_size_bytes"`
	FirstUserPromptPreview string         `json:"first_user_prompt_preview"`
	HasFullTranscript      bool           `json:"has_full_transcript"`
}

// ProjectSummary provides clustered aggregate metrics for a repository or workspace.
// Implements: R-028 (Project & Repository Clustering)
type ProjectSummary struct {
	Name                string    `json:"name"`
	Path                string    `json:"path"`
	IsGit               bool      `json:"is_git"`
	SessionCount        int       `json:"session_count"`
	TotalSteps          int       `json:"total_steps"`
	TotalUserMessages   int       `json:"total_user_messages"`
	TotalAgentResponses int       `json:"total_agent_responses"`
	TotalToolCalls      int       `json:"total_tool_calls"`
	TotalErrors         int       `json:"total_errors"`
	LastActiveAt        time.Time `json:"last_active_at"`
	RecentPrompts       []string  `json:"recent_prompts"`
}

// SessionDetail is the full session with its conversation messages.
// Implements: R-009
type SessionDetail struct {
	Session  Session          `json:"session"`
	Messages []MessageDisplay `json:"messages"`
}

// MessageDisplay is a UI-friendly message representation.
type MessageDisplay struct {
	StepIndex   int               `json:"step_index"`
	Source      string            `json:"source"`
	Type        string            `json:"type"`
	Status      string            `json:"status"`
	CreatedAt   time.Time         `json:"created_at"`
	Content     string            `json:"content,omitempty"`
	HasThinking bool              `json:"has_thinking"`
	Thinking    string            `json:"thinking,omitempty"`
	ToolCalls   []ToolCallDisplay `json:"tool_calls,omitempty"`
}

// ToolCallDisplay is a UI-friendly tool call representation.
type ToolCallDisplay struct {
	Name        string                 `json:"name"`
	ArgsPreview string                 `json:"args_preview,omitempty"`
	Args        map[string]interface{} `json:"args,omitempty"`
}

// SearchResult represents a matching entry from full-text search.
// Implements: R-012
type SearchResult struct {
	SessionID      string    `json:"session_id"`
	StepIndex      int       `json:"step_index"`
	Type           string    `json:"type"`
	Source         string    `json:"source"`
	ContentPreview string    `json:"content_preview"`
	CreatedAt      time.Time `json:"created_at"`
	Workspace      string    `json:"workspace"`
}

// Stats represents aggregate statistics across all sessions.
// Implements: R-017, R-025
type Stats struct {
	TotalSessions       int            `json:"total_sessions"`
	TotalUserMessages   int            `json:"total_user_messages"`
	TotalAgentResponses int            `json:"total_agent_responses"`
	TotalToolCalls      int            `json:"total_tool_calls"`
	TotalErrors         int            `json:"total_errors"`
	DateRange           DateRange      `json:"date_range"`
	Workspaces          []string         `json:"workspaces"`
	Projects            []ProjectSummary `json:"projects"`
	TopToolCalls        []ToolFreq       `json:"top_tool_calls"`
	TopPrompts          []PromptFreq     `json:"top_prompts"`
	MessagesByType      map[string]int   `json:"messages_by_type"`
}

// DateRange represents a from/to date pair.
type DateRange struct {
	From time.Time `json:"from"`
	To   time.Time `json:"to"`
}

// ToolFreq tracks the frequency of tool calls.
type ToolFreq struct {
	Name  string `json:"name"`
	Count int    `json:"count"`
}

// PromptFreq tracks the frequency of common prompt patterns.
// Implements: R-025
type PromptFreq struct {
	Prompt string `json:"prompt"`
	Count  int    `json:"count"`
}

// SessionsResponse is the JSON response for GET /api/sessions.
type SessionsResponse struct {
	Sessions []Session `json:"sessions"`
	Stats    Stats     `json:"stats"`
}

// SessionDetailResponse is the JSON response for GET /api/sessions/:id.
type SessionDetailResponse struct {
	Session  Session          `json:"session"`
	Messages []MessageDisplay `json:"messages"`
}

// SearchResponse is the JSON response for GET /api/search.
type SearchResponse struct {
	Results    []SearchResult `json:"results"`
	TotalCount int            `json:"total_count"`
	Query      string         `json:"query"`
}

// Config holds the application configuration from CLI flags.
type Config struct {
	DataDir string
	Port    int
	NoOpen  bool
	Verbose bool
}
