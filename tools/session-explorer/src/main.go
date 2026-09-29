// Package main is the entry point for Session Explorer.
//
// Session Explorer is a self-contained CLI tool that scans Antigravity IDE
// conversation sessions and serves a premium glassmorphic web UI for browsing,
// searching, and filtering agent session histories.
//
// Implements: R-006 (Serve web UI), R-007 (Auto-open browser), R-023 (--data-dir),
//             R-024 (--port), NF-005 (Zero dependencies), NF-009 (Fast startup)
// See: specs/catalog/session-explorer.md
package main

import (
	"flag"
	"fmt"
	"log"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"time"
)

const (
	appName    = "session-explorer"
	appVersion = "0.1.0"
)

func main() {
	// CLI flag parsing — Implements: R-023, R-024
	homeDir, _ := os.UserHomeDir()
	defaultDataDir := filepath.Join(homeDir, ".gemini", "antigravity-ide", "brain")

	var cfg Config
	flag.StringVar(&cfg.DataDir, "data-dir", defaultDataDir, "Path to Antigravity brain directory")
	flag.StringVar(&cfg.DataDir, "d", defaultDataDir, "Path to Antigravity brain directory (shorthand)")
	flag.IntVar(&cfg.Port, "port", 9876, "Port to serve the web UI on")
	flag.IntVar(&cfg.Port, "p", 9876, "Port to serve the web UI on (shorthand)")
	flag.IntVar(&cfg.WatchInterval, "watch-interval", 10, "Seconds between background auto-rescans (0 = disabled)")
	flag.IntVar(&cfg.WatchInterval, "w", 10, "Seconds between background auto-rescans (shorthand)")
	flag.BoolVar(&cfg.NoOpen, "no-open", false, "Don't auto-open browser")
	flag.BoolVar(&cfg.Verbose, "verbose", false, "Enable verbose logging")
	flag.BoolVar(&cfg.Verbose, "v", false, "Enable verbose logging (shorthand)")

	showVersion := flag.Bool("version", false, "Show version")
	flag.Parse()

	if *showVersion {
		fmt.Printf("%s v%s\n", appName, appVersion)
		os.Exit(0)
	}

	// Validate data directory
	if _, err := os.Stat(cfg.DataDir); os.IsNotExist(err) {
		log.Fatalf("❌ Data directory not found: %s\n\nUse --data-dir to specify the path to your Antigravity IDE brain directory.", cfg.DataDir)
	}

	// Banner
	fmt.Printf(`
  ╔══════════════════════════════════════════╗
  ║     🔍 Session Explorer v%s          ║
  ║     Antigravity IDE Session Library      ║
  ╚══════════════════════════════════════════╝
`, appVersion)

	// Create session index and scan — Implements: R-001
	index := NewSessionIndex(cfg.DataDir, cfg.Verbose)

	log.Println("📂 Scanning sessions...")
	if err := index.ScanAll(); err != nil {
		log.Fatalf("❌ Failed to scan sessions: %v", err)
	}

	stats := index.GetStats()
	log.Printf("✅ Found %d sessions, %d user messages, %d agent responses across %d workspaces",
		stats.TotalSessions, stats.TotalUserMessages, stats.TotalAgentResponses, len(stats.Workspaces))

	// Auto-open browser — Implements: R-007
	if !cfg.NoOpen {
		url := fmt.Sprintf("http://127.0.0.1:%d", cfg.Port)
		go func() {
			time.Sleep(300 * time.Millisecond) // Wait for server to start
			if err := openBrowser(url); err != nil {
				log.Printf("⚠️  Could not open browser: %v\n  → Navigate to %s manually", err, url)
			}
		}()
	}

	// Start server — Implements: R-006
	if err := StartServer(cfg, index); err != nil {
		log.Fatalf("❌ Server error: %v", err)
	}
}

// openBrowser opens the default browser on the current OS.
// Implements: R-007
func openBrowser(url string) error {
	var cmd *exec.Cmd

	switch runtime.GOOS {
	case "darwin":
		cmd = exec.Command("open", url)
	case "linux":
		cmd = exec.Command("xdg-open", url)
	case "windows":
		cmd = exec.Command("rundll32", "url.dll,FileProtocolHandler", url)
	default:
		return fmt.Errorf("unsupported platform: %s", runtime.GOOS)
	}

	return cmd.Start()
}
