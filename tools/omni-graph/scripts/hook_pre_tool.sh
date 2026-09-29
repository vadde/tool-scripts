#!/usr/bin/env bash
# ==============================================================================
# Omni-Graph PreToolUse Guardrail Hook
# Intercepts brute-force grep, broad search, and unbounded source file reads (>150 lines)
# that exhaust LLM context windows. Enforces high-density AST retrieval.
# ==============================================================================

set -e

# Read hook payload from stdin
PAYLOAD=$(cat)

python3 -c '
import sys, json, os, urllib.request

try:
    # 1. Daemon Liveness Check (Fail-Open Safeguard)
    # If Omni-Graph daemon is offline, allow all tools immediately so agents are never blocked
    try:
        req = urllib.request.Request("http://localhost:8080/api/health", headers={"User-Agent": "OmniHook/1.0"})
        with urllib.request.urlopen(req, timeout=0.5) as resp:
            if resp.status != 200:
                print(json.dumps({"decision": "allow"}))
                sys.exit(0)
    except Exception:
        # Daemon is offline or unreachable -> Fail-Open
        print(json.dumps({"decision": "allow"}))
        sys.exit(0)

    # 2. Parse Tool Call Payload
    raw_input = sys.argv[1] if len(sys.argv) > 1 else sys.stdin.read()
    data = json.loads(raw_input) if raw_input.strip() else {}
    tool_call = data.get("toolCall", {})
    tool_name = tool_call.get("name", "")
    args = tool_call.get("args", {})

    NON_CODE_EXTS = {
        ".md", ".markdown", ".mdown", ".mkd", ".txt", ".text",
        ".json", ".json5", ".yaml", ".yml", ".toml", ".ini", ".cfg", ".conf",
        ".csv", ".tsv", ".svg", ".png", ".jpg", ".jpeg", ".gif", ".ico", ".webp",
        ".lock", ".sum", ".env", ".example", ".sample", ".pdf", ".mp4", ".mov",
        ".log", ".tmp", ".bak", ".patch", ".diff", ".pid", ".socket", ".wasm",
        ".bin", ".exe", ".o", ".a", ".so", ".dylib", ".dll", ".class", ".jar"
    }

    NON_CODE_FILENAMES = {
        "makefile", "dockerfile", "docker-compose.yml", "docker-compose.yaml",
        "license", "licence", "readme", "notice", "authors", "contributors",
        "cargo.lock", "package-lock.json", "pnpm-lock.yaml", "yarn.lock",
        "go.sum", "gemfile.lock", "poetry.lock", "mix.lock"
    }

    def is_source_code(file_path):
        if not file_path:
            return False
        base = os.path.basename(file_path).lower()
        if base in NON_CODE_FILENAMES:
            return False
        _, ext = os.path.splitext(base)
        if ext in NON_CODE_EXTS:
            return False
        return True

    # 3. Intercept view_file (>150 lines gate on source code)
    if tool_name == "view_file":
        target_path = args.get("AbsolutePath", "")
        if is_source_code(target_path):
            start_line = args.get("StartLine")
            end_line = args.get("EndLine")

            # Missing EndLine means reading up to full file limit (800 lines)
            if end_line is None:
                print(json.dumps({
                    "decision": "deny",
                    "reason": "BLOCKED: Reading >150 lines of source code directly exhausts agent context. Omni-Graph is active. Query the AST subgraph instead: GET http://localhost:8080/api/condense?symbol=<target_symbol>&hops=2 or query symbol definitions via GET http://localhost:8080/api/symbol?name=<target_symbol>. If you must view raw lines, narrow your slice to <= 150 lines."
                }))
                sys.exit(0)

            try:
                start = int(start_line) if start_line is not None else 1
                end = int(end_line)
                span = end - start + 1
                if span > 150:
                    print(json.dumps({
                        "decision": "deny",
                        "reason": f"BLOCKED: Requested line slice of {span} lines exceeds the 150-line gate. Omni-Graph is active. Query AST symbol definition (GET http://localhost:8080/api/symbol?name=<symbol>) or narrow your view slice to <= 150 lines."
                    }))
                    sys.exit(0)
            except (ValueError, TypeError):
                pass

    # 4. Intercept run_command (recursive/blanket search)
    elif tool_name == "run_command":
        cmd = args.get("CommandLine", "")
        blocked_patterns = [
            "grep -r", "grep -rn", "grep -ri", "grep -rin", "grep -rIn",
            "find . -name", "cat $(find", "ag -l", "rg -l"
        ]
        if any(p in cmd for p in blocked_patterns):
            print(json.dumps({
                "decision": "deny",
                "reason": "Blind recursive search blocked. Query Omni-Graph semantic vector search: GET http://localhost:8080/api/search?q=<query> or AST symbol lookups."
            }))
            sys.exit(0)

    # 5. Intercept grep_search (blanket search)
    elif tool_name == "grep_search":
        query = args.get("Query", "").strip()
        if len(query) < 2:
            print(json.dumps({
                "decision": "deny",
                "reason": "Broad blanket grep search without target query detected. Use Omni-Graph AST symbol & reference endpoints (GET http://localhost:8080/api/symbol?name=...)."
            }))
            sys.exit(0)

    # Default Allow
    print(json.dumps({"decision": "allow"}))

except Exception as e:
    # Fail-safe allow so agents are never hard-crashed on script error
    print(json.dumps({"decision": "allow"}))
' "$PAYLOAD"
