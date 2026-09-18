// Package main provides the HTTP server with embedded static assets.
//
// Implements: R-006 (Serve embedded web UI), NF-008 (Bind to localhost only)
// See: specs/catalog/session-explorer.md
package main

import (
	"embed"
	"fmt"
	"io/fs"
	"log"
	"net/http"
)

//go:embed web/dist/*
var staticFiles embed.FS

// StartServer creates and starts the HTTP server.
// Implements: R-006, NF-008
func StartServer(cfg Config, index *SessionIndex) error {
	api := NewAPIHandler(index, cfg.Verbose)

	mux := http.NewServeMux()

	// API routes
	mux.HandleFunc("/api/sessions", func(w http.ResponseWriter, r *http.Request) {
		// Route to detail if path has an ID segment
		path := r.URL.Path
		if path != "/api/sessions" && path != "/api/sessions/" {
			api.HandleSessionDetail(w, r)
			return
		}
		api.HandleSessions(w, r)
	})
	mux.HandleFunc("/api/sessions/", api.HandleSessionDetail)
	mux.HandleFunc("/api/search", api.HandleSearch)
	mux.HandleFunc("/api/stats", api.HandleStats)

	// Static files (embedded frontend with no-cache headers to avoid browser caching stale JS)
	staticFS, err := fs.Sub(staticFiles, "web/dist")
	if err != nil {
		return fmt.Errorf("failed to create sub-filesystem: %w", err)
	}
	fileServer := http.FileServer(http.FS(staticFS))
	mux.Handle("/", http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Cache-Control", "no-cache, no-store, must-revalidate")
		w.Header().Set("Pragma", "no-cache")
		w.Header().Set("Expires", "0")
		fileServer.ServeHTTP(w, r)
	}))

	addr := fmt.Sprintf("127.0.0.1:%d", cfg.Port)
	log.Printf("🚀 Session Explorer serving at http://%s", addr)

	server := &http.Server{
		Addr:    addr,
		Handler: mux,
	}

	return server.ListenAndServe()
}
