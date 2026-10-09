#!/usr/bin/env bash
# ==============================================================================
# Omni-Graph PreInvocation Hook
# Dynamically resolves the active workspace, checks Omni-Graph status,
# records workspace indexing status for PreToolUse recon gate,
# and injects actionable, workspace-scoped AST endpoints into the agent prompt.
# ==============================================================================

set -e

python3 << 'EOF'
import json, os, subprocess, urllib.request

try:
    # 1. Check Omni-Graph Health (fast 500ms timeout)
    req = urllib.request.Request("http://localhost:8080/api/health", headers={"User-Agent": "OmniHook/1.0"})
    with urllib.request.urlopen(req, timeout=0.5) as resp:
        if resp.status != 200:
            print(json.dumps({"injectSteps": []}))
            exit(0)

    # 2. Dynamically determine current workspace name & detect Git Worktree
    cwd = os.getcwd()
    git_root = ""
    try:
        git_root = subprocess.check_output(
            ["git", "rev-parse", "--show-toplevel"],
            stderr=subprocess.DEVNULL,
            encoding="utf-8"
        ).strip()
        ws_candidate = os.path.basename(git_root)
    except Exception:
        ws_candidate = os.path.basename(cwd)

    # Detect if working inside a Git Worktree
    wt_info = None
    git_file = os.path.join(git_root, ".git") if git_root else os.path.join(cwd, ".git")
    if os.path.isfile(git_file):
        try:
            with open(git_file, "r") as gf:
                first_line = gf.readline().strip()
            if first_line.startswith("gitdir:"):
                gitdir_path = first_line[7:].strip()
                if not os.path.isabs(gitdir_path):
                    gitdir_path = os.path.normpath(os.path.join(os.path.dirname(git_file), gitdir_path))
                head_file = os.path.join(gitdir_path, "HEAD")
                branch = "detached"
                if os.path.isfile(head_file):
                    with open(head_file, "r") as hf:
                        head_line = hf.readline().strip()
                        if head_line.startswith("ref: refs/heads/"):
                            branch = head_line[16:]
                parts = gitdir_path.replace("\\", "/").split("/")
                parent_ws = "parent"
                if ".git" in parts:
                    idx = parts.index(".git")
                    if idx > 0:
                        parent_ws = parts[idx - 1]
                wt_name = os.path.basename(git_root) or os.path.basename(cwd)
                wt_info = {
                    "is_worktree": True,
                    "worktree_name": wt_name,
                    "parent_workspace": parent_ws,
                    "branch": branch
                }
        except Exception:
            pass

    # 3. Query workspaces from Omni-Graph
    req_ws = urllib.request.Request("http://localhost:8080/api/workspaces", headers={"User-Agent": "OmniHook/1.0"})
    workspaces = []
    with urllib.request.urlopen(req_ws, timeout=0.8) as resp_ws:
        workspaces = json.loads(resp_ws.read().decode("utf-8"))

    # Cache list of indexed workspace names for fast PreToolUse lookup
    try:
        indexed_names = [w.get("workspace") for w in workspaces if w.get("total_nodes", 0) > 0]
        with open("/tmp/omni_indexed_workspaces.json", "w") as f:
            json.dump(indexed_names, f)
    except Exception:
        pass

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
        try:
            with open(f"/tmp/omni_indexed_{matched_ws}.flag", "w") as f:
                f.write(str(node_count))
        except Exception:
            pass

        # Ambient Live-Watch Auto-Enrollment: Ensure active indexed workspace is actively watched
        for w in workspaces:
            if w.get("workspace") == matched_ws and not w.get("is_watching", False):
                try:
                    target_watch_path = git_root or cwd
                    watch_payload = json.dumps({"workspace": matched_ws, "path": target_watch_path}).encode("utf-8")
                    watch_req = urllib.request.Request(
                        "http://localhost:8080/api/watch/start",
                        data=watch_payload,
                        headers={"Content-Type": "application/json", "User-Agent": "OmniHook/1.0"}
                    )
                    urllib.request.urlopen(watch_req, timeout=0.8)
                except Exception:
                    pass
                break

        wt_suffix = f" (Worktree: {wt_info['branch']} -> {wt_info['parent_workspace']})" if wt_info else ""
        msg = (
            f"🧭 [Omni-Graph Active | Workspace: {matched_ws}{wt_suffix} ({node_count:,} AST nodes)]:\n"
            f"• Tier 1 (Graph-RAG Query):  curl -s -X POST http://localhost:8080/api/query -H \"Content-Type: application/json\" -d '{{\"prompt\": \"<question>\", \"workspace\": \"{matched_ws}\"}}'\n"
            f"• Tier 2 (Galaxy Clusters):  curl -s \"http://localhost:8080/api/galaxies?workspace={matched_ws}\"\n"
            f"• Tier 3 (AST Subgraph):     curl -s \"http://localhost:8080/api/condense?symbol=<sym>&workspace={matched_ws}&hops=2\"\n"
            f"• Blast Radius / Callers:    curl -s \"http://localhost:8080/api/references?symbol=<sym>&workspace={matched_ws}\"\n"
            f"• Symbol Definition:         curl -s \"http://localhost:8080/api/symbol?name=<sym>&workspace={matched_ws}\"\n"
            f"• Semantic Vector Search:    curl -s \"http://localhost:8080/api/search?q=<query>&workspace={matched_ws}&k=5\"\n"
            f"⚠️ Direct source code reads (>150 lines) and recursive grep are gated until Omni-Graph recon is performed."
        )
    else:
        try:
            flag = f"/tmp/omni_indexed_{matched_ws}.flag"
            if os.path.exists(flag):
                os.remove(flag)
        except Exception:
            pass

        if wt_info:
            msg = (
                f"🧭 [Omni-Graph Active | Unindexed Git Worktree: {matched_ws} ({wt_info['branch']} -> {wt_info['parent_workspace']})]:\n"
                f"• Ingest & Auto-Watch: curl -s -X POST http://localhost:8080/api/ingest -H \"Content-Type: application/json\" -d '{{\"path\": \"{cwd}\", \"watch\": true}}'\n"
                f"• Auto-inherits galaxy cluster seeds from parent: {wt_info['parent_workspace']}\n"
                f"*Note: Direct view_file >150 lines on source code is gated by PreToolUse.*"
            )
        else:
            msg = (
                f"🧭 [Omni-Graph Active | Workspace: {matched_ws}]:\n"
                f"• Index Codebase: curl -s -X POST http://localhost:8080/api/ingest -H \"Content-Type: application/json\" -d \"{{\\\"path\\\": \\\"{cwd}\\\"}}\"\n"
                f"• Semantic Search: curl -s \"http://localhost:8080/api/search?q=<query>&k=5\"\n"
                f"*Note: Direct view_file >150 lines on source code is gated by PreToolUse.*"
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
EOF
