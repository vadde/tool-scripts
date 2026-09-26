#!/usr/bin/env bash
# ==============================================================================
# Omni-Graph PreToolUse Guardrail Hook
# Intercepts brute-force grep/cat that burn LLM context windows and guides
# the agent towards AST-indexed semantic search and graph traversal.
# ==============================================================================

set -e

# Read hook payload from stdin
PAYLOAD=$(cat)

# Extract tool name and command using lightweight python / awk
python3 -c '
import sys, json

try:
    data = json.loads(sys.argv[1]) if len(sys.argv) > 1 else json.load(sys.stdin)
    tool_call = data.get("toolCall", {})
    tool_name = tool_call.get("name", "")
    args = tool_call.get("args", {})

    cmd = args.get("CommandLine", "")
    query = args.get("Query", "")
    search_path = args.get("SearchPath", "")

    # Detect whole-repo brute force text search patterns
    is_brute_force = False
    reason = ""

    if tool_name == "run_command":
        if any(p in cmd for p in ["grep -r", "grep -rn", "find . -name", "cat $(find"]):
            is_brute_force = True
            reason = "Blind recursive text scanning detected. Omni-Graph is active — query semantic vector search (GET http://localhost:8080/api/search?q=...) or symbol endpoints instead."

    elif tool_name == "grep_search":
        if query.strip() == "" or len(query) < 2:
            is_brute_force = True
            reason = "Broad blanket grep search without target query detected. Use Omni-Graph AST symbol & reference endpoints (GET http://localhost:8080/api/symbol?name=...)."

    if is_brute_force:
        print(json.dumps({
            "decision": "deny",
            "reason": reason
        }))
    else:
        print(json.dumps({
            "decision": "allow"
        }))

except Exception as e:
    # Fail-safe allow so agents are never hard-crashed on script error
    print(json.dumps({"decision": "allow"}))
' "$PAYLOAD"
