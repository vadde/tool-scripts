#!/usr/bin/env bash
# ============================================================================
# Example: session-explorer demonstration script
# Runs session-explorer against a sample mock dataset
# ============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TOOL_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "🚀 Building session-explorer..."
make -C "$TOOL_DIR" build

echo ""
echo "📌 Binary info:"
"$TOOL_DIR/bin/session-explorer" --version

echo ""
echo "🛠️ Creating temporary mock brain dataset..."
SAMPLE_DIR="$(mktemp -d)/sample-brain"
mkdir -p "$SAMPLE_DIR/sample-session-001/.system_generated/logs"

cat << 'EOF' > "$SAMPLE_DIR/sample-session-001/.system_generated/logs/transcript.jsonl"
{"step_index":0,"source":"USER_EXPLICIT","type":"USER_INPUT","status":"DONE","created_at":"2026-09-18T10:00:00Z","content":"<USER_REQUEST>\nLet us build a microservice architecture in Go with Docker\n</USER_REQUEST>\n<ADDITIONAL_METADATA>\nActive Document: /Users/developer/projects/microservices/main.go\n</ADDITIONAL_METADATA>"}
{"step_index":1,"source":"MODEL","type":"PLANNER_RESPONSE","status":"DONE","created_at":"2026-09-18T10:01:00Z","content":"Here is a comprehensive microservices architecture plan...","tool_calls":[{"name":"view_file","args":{"AbsolutePath":"/Users/developer/projects/microservices/main.go"}}]}
EOF

echo "✅ Sample dataset created at $SAMPLE_DIR"
echo ""
echo "🧪 Running session-explorer with sample data on port 9991 (no-open mode)..."
"$TOOL_DIR/bin/session-explorer" --data-dir "$SAMPLE_DIR" --port 9991 --no-open &
PID=$!

trap 'kill $PID 2>/dev/null || true; rm -rf "$(dirname "$SAMPLE_DIR")"' EXIT

sleep 1.5

echo ""
echo "📡 Querying /api/stats:"
curl -s http://127.0.0.1:9991/api/stats | python3 -m json.tool || true

echo ""
echo "📡 Querying /api/sessions:"
curl -s http://127.0.0.1:9991/api/sessions | python3 -m json.tool || true

echo ""
echo "🎉 Demo completed successfully!"
