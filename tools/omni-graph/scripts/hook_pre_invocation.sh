#!/usr/bin/env bash
# ==============================================================================
# Omni-Graph PreInvocation Hook
# Dynamically resolves the active workspace, checks Omni-Graph status,
# and injects actionable, workspace-scoped AST endpoints into the agent prompt.
# ==============================================================================

set -e

python3 -c '
import json, os, subprocess, urllib.request

try:
    # 1. Check Omni-Graph Health (fast 500ms timeout)
    req = urllib.request.Request("http://localhost:8080/api/health", headers={"User-Agent": "OmniHook/1.0"})
    with urllib.request.urlopen(req, timeout=0.5) as resp:
        if resp.status != 200:
            print(json.dumps({"injectSteps": []}))
            exit(0)

    # 2. Dynamically determine current workspace name
    cwd = os.getcwd()
    try:
        git_root = subprocess.check_output(
            ["git", "rev-parse", "--show-toplevel"],
            stderr=subprocess.DEVNULL,
            encoding="utf-8"
        ).strip()
        ws_candidate = os.path.basename(git_root)
    except Exception:
        ws_candidate = os.path.basename(cwd)

    # 3. Query workspaces from Omni-Graph
    req_ws = urllib.request.Request("http://localhost:8080/api/workspaces", headers={"User-Agent": "OmniHook/1.0"})
    workspaces = []
    with urllib.request.urlopen(req_ws, timeout=0.8) as resp_ws:
        workspaces = json.loads(resp_ws.read().decode("utf-8"))

    matched_ws = None
    node_count = 0

    # Match by workspace name or canonical root_path
    for w in workspaces:
        name = w.get("workspace", "")
        root_path = w.get("root_path", "")
        if name.lower() == ws_candidate.lower() or (root_path and (cwd == root_path or cwd.startswith(root_path + "/"))):
            matched_ws = name
            node_count = w.get("total_nodes", 0)
            break

    if not matched_ws:
        matched_ws = ws_candidate

    if node_count > 0:
        msg = (
            f"🧭 [Omni-Graph Active | Workspace: {matched_ws} ({node_count:,} AST nodes)]:\n"
            f"• AST Condense: curl -s \"http://localhost:8080/api/condense?symbol=<sym>&workspace={matched_ws}&hops=2\"\n"
            f"• Callers/Blast Radius: curl -s \"http://localhost:8080/api/references?symbol=<sym>&workspace={matched_ws}\"\n"
            f"• Symbol Def: curl -s \"http://localhost:8080/api/symbol?name=<sym>&workspace={matched_ws}\"\n"
            f"• Semantic Search: curl -s \"http://localhost:8080/api/search?q=<query>&workspace={matched_ws}&k=5\"\n"
            "*Note: Direct view_file >150 lines on source code is gated by PreToolUse.*"
        )
    else:
        msg = (
            f"🧭 [Omni-Graph Active | Workspace: {matched_ws}]:\n"
            f"• Index Codebase: curl -s -X POST http://localhost:8080/api/ingest -H \"Content-Type: application/json\" -d \"{{\\\"path\\\": \\\"{cwd}\\\"}}\"\n"
            f"• Semantic Search: curl -s \"http://localhost:8080/api/search?q=<query>&k=5\"\n"
            "*Note: Direct view_file >150 lines on source code is gated by PreToolUse.*"
        )

    print(json.dumps({
        "injectSteps": [
            {
                "ephemeralMessage": msg
            }
        ]
    }))

except Exception:
    # Fail-safe empty injectSteps
    print(json.dumps({"injectSteps": []}))
'
