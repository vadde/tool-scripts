#!/usr/bin/env bash
# ==============================================================================
# setup-agent.sh — Omni-Graph Agent Enforcement Bootstrapper
# Supports:
#   1. Current Workspace:  ./scripts/setup-agent.sh [workspace]
#   2. Global System:      ./scripts/setup-agent.sh global
#   3. Target Directory:   ./scripts/setup-agent.sh /path/to/any/codebase
#
# Robust Guardrails:
#   • Validates directory existence (no blind silent mkdir)
#   • Blocks protected OS system directories (/, /System, /Library, /usr, etc.)
#   • Catches file vs directory mistakes with helpful guidance
#   • Suggests matching sibling codebases on typos
#   • Preserves & merges existing hooks.json rather than overwriting
#   • Appends to existing AGENTS.md cleanly
# ==============================================================================

set -euo pipefail

TARGET="${1:-workspace}"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
TOOL_DIR="${REPO_ROOT}/tools/omni-graph"

# ─── Help Banner ─────────────────────────────────────────────────────────────
if [[ "${TARGET}" == "--help" || "${TARGET}" == "-h" || "${TARGET}" == "help" ]]; then
  echo "╔══════════════════════════════════════════════════════════════════════════╗"
  echo "║     🛡️  Omni-Graph Agent Enforcement Shield Setup                        ║"
  echo "╚══════════════════════════════════════════════════════════════════════════╝"
  echo ""
  echo "Usage:"
  echo "  make setup-agent [DIR=<path>|TARGET=<path>|global]"
  echo "  ./scripts/setup-agent.sh [<path>|workspace|global]"
  echo ""
  echo "Examples:"
  echo "  • make setup-agent                         # Setup current workspace (.agents)"
  echo "  • make setup-agent global                  # Setup machine-wide (~/.gemini/config)"
  echo "  • make setup-agent /path/to/codebase       # Setup specific target codebase"
  echo "  • make setup-agent DIR=/path/to/codebase   # Setup specific target codebase"
  exit 0
fi

echo "⚙️  Configuring Omni-Graph Agent Enforcement Shield..."
echo "  → Target mode: ${TARGET}"

# Ensure source scripts are executable
chmod +x "${TOOL_DIR}/scripts/hook_pre_tool.sh"
chmod +x "${TOOL_DIR}/scripts/hook_pre_invocation.sh"
chmod +x "${TOOL_DIR}/scripts/hook_stop.sh"
chmod +x "${TOOL_DIR}/scripts/omni.sh"
chmod +x "${TOOL_DIR}/skills/omni-graph/scripts/omni.sh"

# ─── Helper: Safe Hooks Merge ────────────────────────────────────────────────
merge_hooks_json() {
  local script_dir="$1"
  local output_file="$2"

  python3 -c "
import json, os, sys

script_dir = sys.argv[1]
output_file = sys.argv[2]

guardrail_config = {
    'enabled': True,
    'PreToolUse': [
        {
            'matcher': 'run_command|grep_search',
            'hooks': [
                {
                    'type': 'command',
                    'command': f'{script_dir}/hook_pre_tool.sh',
                    'timeout': 5
                }
            ]
        }
    ],
    'PreInvocation': [
        {
            'type': 'command',
            'command': f'{script_dir}/hook_pre_invocation.sh',
            'timeout': 5
        }
    ],
    'Stop': [
        {
            'type': 'command',
            'command': f'{script_dir}/hook_stop.sh',
            'timeout': 5
        }
    ]
}

data = {}
if os.path.exists(output_file):
    try:
        with open(output_file, 'r', encoding='utf-8') as f:
            data = json.load(f)
    except Exception:
        data = {}

data['omni-graph-guardrail'] = guardrail_config

with open(output_file, 'w', encoding='utf-8') as f:
    json.dump(data, f, indent=2)
" "${script_dir}" "${output_file}"
}

# ─── Target Resolution & Guardrails ──────────────────────────────────────────

if [[ "${TARGET}" == "global" || "${TARGET}" == "--global" ]]; then
  DEST_DIR="${HOME}/.gemini/config"
  echo "  → Target directory: ${DEST_DIR}"

  mkdir -p "${DEST_DIR}/rules"
  mkdir -p "${DEST_DIR}/skills/omni-graph/scripts"
  mkdir -p "${DEST_DIR}/scripts"

  # 1. Install Global Rule
  cp "${TOOL_DIR}/rules/08-omni-graph-enforcement.md" "${DEST_DIR}/rules/08-omni-graph-enforcement.md"
  echo "  ✅ Installed global rule: ~/.gemini/config/rules/08-omni-graph-enforcement.md"

  # 2. Install Global Skill
  cp -r "${TOOL_DIR}/skills/omni-graph/"* "${DEST_DIR}/skills/omni-graph/"
  chmod +x "${DEST_DIR}/skills/omni-graph/scripts/omni.sh"
  echo "  ✅ Installed global skill: ~/.gemini/config/skills/omni-graph/"

  # 3. Copy lifecycle hook scripts into global scripts directory
  cp "${TOOL_DIR}/scripts/hook_pre_tool.sh" "${DEST_DIR}/scripts/hook_pre_tool.sh"
  cp "${TOOL_DIR}/scripts/hook_pre_invocation.sh" "${DEST_DIR}/scripts/hook_pre_invocation.sh"
  cp "${TOOL_DIR}/scripts/hook_stop.sh" "${DEST_DIR}/scripts/hook_stop.sh"
  chmod +x "${DEST_DIR}/scripts/"*.sh

  # 4. Safely configure / merge global hooks
  merge_hooks_json "${DEST_DIR}/scripts" "${DEST_DIR}/hooks.json"
  echo "  ✅ Configured global hooks: ~/.gemini/config/hooks.json"

elif [[ "${TARGET}" == "workspace" || "${TARGET}" == "." ]]; then
  DEST_DIR="${REPO_ROOT}/.agents"
  echo "  → Target directory: ${DEST_DIR}"

  mkdir -p "${DEST_DIR}/rules"
  mkdir -p "${DEST_DIR}/skills/omni-graph/scripts"
  mkdir -p "${DEST_DIR}/scripts"

  # 1. Install Workspace Rule
  cp "${TOOL_DIR}/rules/08-omni-graph-enforcement.md" "${DEST_DIR}/rules/08-omni-graph-enforcement.md"
  echo "  ✅ Installed rule: .agents/rules/08-omni-graph-enforcement.md"

  # 2. Install Workspace Skill
  cp -r "${TOOL_DIR}/skills/omni-graph/"* "${DEST_DIR}/skills/omni-graph/"
  chmod +x "${DEST_DIR}/skills/omni-graph/scripts/omni.sh"
  echo "  ✅ Installed skill: .agents/skills/omni-graph/"

  # 3. Copy scripts to .agents/scripts
  cp "${TOOL_DIR}/scripts/hook_pre_tool.sh" "${DEST_DIR}/scripts/hook_pre_tool.sh"
  cp "${TOOL_DIR}/scripts/hook_pre_invocation.sh" "${DEST_DIR}/scripts/hook_pre_invocation.sh"
  cp "${TOOL_DIR}/scripts/hook_stop.sh" "${DEST_DIR}/scripts/hook_stop.sh"
  chmod +x "${DEST_DIR}/scripts/"*.sh

  # 4. Configure hooks.json
  merge_hooks_json "${DEST_DIR}/scripts" "${DEST_DIR}/hooks.json"
  echo "  ✅ Configured lifecycle hooks: .agents/hooks.json"

else
  # ─── User-Specified Codebase Guardrails ──────────────────────────────────
  TARGET_PATH="${TARGET}"

  # Guardrail 1: Check if input is a file rather than directory
  if [[ -f "${TARGET_PATH}" ]]; then
    echo "❌ ERROR: Target path is a file, not a codebase directory:"
    echo "   '${TARGET_PATH}'"
    PARENT_SUGGESTION="$(dirname "${TARGET_PATH}")"
    echo ""
    echo "💡 Did you mean the parent directory?"
    echo "   make setup-agent \"${PARENT_SUGGESTION}\""
    exit 1
  fi

  # Guardrail 2: Check existence of target directory (prevent silent accidental folder creation)
  if [[ ! -e "${TARGET_PATH}" ]]; then
    echo "❌ ERROR: Target directory does not exist:"
    echo "   '${TARGET_PATH}'"
    
    PARENT_DIR="$(dirname "${TARGET_PATH}")"
    BASE_NAME="$(basename "${TARGET_PATH}")"
    
    if [[ -d "${PARENT_DIR}" ]]; then
      echo ""
      echo "💡 Existing sister directories found in '${PARENT_DIR}':"
      MATCHES=$(find "${PARENT_DIR}" -maxdepth 1 -mindepth 1 -type d | grep -i "${BASE_NAME:0:3}" | head -n 5 || true)
      if [[ -n "${MATCHES}" ]]; then
        while IFS= read -r match; do
          echo "   • ${match}"
        done <<< "${MATCHES}"
      else
        find "${PARENT_DIR}" -maxdepth 1 -mindepth 1 -type d | head -n 6 | while IFS= read -r d; do
          echo "   • ${d}"
        done || true
      fi
    fi

    # Interactive check
    if [ -t 0 ]; then
      echo ""
      read -r -t 15 -p "❓ Would you like to create this new directory now? [y/N]: " CONFIRM || CONFIRM="n"
      if [[ "${CONFIRM}" =~ ^[Yy]$ ]]; then
        mkdir -p "${TARGET_PATH}"
        echo "  📁 Created directory: ${TARGET_PATH}"
      else
        echo "🛑 Setup aborted. Please verify the target path."
        exit 1
      fi
    else
      echo ""
      echo "🛑 Setup aborted (non-interactive mode). Please verify the target path."
      exit 1
    fi
  fi

  # Resolve to absolute path
  TARGET_ABS="$(cd "${TARGET_PATH}" && pwd)"

  # Guardrail 3: Block dangerous root or system OS directories
  case "${TARGET_ABS}" in
    / | /root | /bin | /sbin | /usr | /usr/* | /System* | /Library* | /Applications* | /etc* | /var* | /private*)
      echo "🛑 ERROR: Refusing to configure agent shield on protected system directory: ${TARGET_ABS}"
      echo "   Please select a valid user repository or workspace."
      exit 1
      ;;
  esac

  # Guardrail 4: Check write permissions
  if [[ ! -w "${TARGET_ABS}" ]]; then
    echo "❌ ERROR: Insufficient permissions. Target directory is not writable:"
    echo "   '${TARGET_ABS}'"
    exit 1
  fi

  echo "  → Verified target codebase: ${TARGET_ABS}"
  DEST_DIR="${TARGET_ABS}/.agents"

  mkdir -p "${DEST_DIR}/rules"
  mkdir -p "${DEST_DIR}/skills/omni-graph/scripts"
  mkdir -p "${DEST_DIR}/scripts"

  # 1. Install Rule
  cp "${TOOL_DIR}/rules/08-omni-graph-enforcement.md" "${DEST_DIR}/rules/08-omni-graph-enforcement.md"
  echo "  ✅ Installed rule: ${TARGET_ABS}/.agents/rules/08-omni-graph-enforcement.md"

  # 2. Install Skill
  cp -r "${TOOL_DIR}/skills/omni-graph/"* "${DEST_DIR}/skills/omni-graph/"
  chmod +x "${DEST_DIR}/skills/omni-graph/scripts/omni.sh"
  echo "  ✅ Installed skill: ${TARGET_ABS}/.agents/skills/omni-graph/"

  # 3. Copy scripts to .agents/scripts
  cp "${TOOL_DIR}/scripts/hook_pre_tool.sh" "${DEST_DIR}/scripts/hook_pre_tool.sh"
  cp "${TOOL_DIR}/scripts/hook_pre_invocation.sh" "${DEST_DIR}/scripts/hook_pre_invocation.sh"
  cp "${TOOL_DIR}/scripts/hook_stop.sh" "${DEST_DIR}/scripts/hook_stop.sh"
  chmod +x "${DEST_DIR}/scripts/"*.sh

  # 4. Safely merge hooks.json with absolute script paths
  merge_hooks_json "${DEST_DIR}/scripts" "${DEST_DIR}/hooks.json"
  echo "  ✅ Configured lifecycle hooks: ${TARGET_ABS}/.agents/hooks.json"

  # 5. Provide or cleanly update AGENTS.md in the target codebase
  if [[ ! -f "${TARGET_ABS}/AGENTS.md" ]]; then
    cat << EOF > "${TARGET_ABS}/AGENTS.md"
# 🤖 AGENTS.md — Omni-Graph Grounded Workspace

> **This workspace is integrated with Omni-Graph semantic AST & knowledge hub.**
> Before reading files or performing blind regex scans, follow Rule 08 and query the AST.

---

## Agent Operating Protocol

1. **AST & Call Graph Lookup**: Use the \`omni-graph\` skill or query \`http://localhost:8080/api/symbol\`
2. **Context Condensation**: Retrieve 2-hop topological call chains (<1500 tokens) instead of reading multiple raw files.
3. **Semantic Search**: Use \`http://localhost:8080/api/search?q=...\` for vector similarity lookup across functions, classes, and types.
4. **Enforced Rules**: Read [\`.agents/rules/08-omni-graph-enforcement.md\`](.agents/rules/08-omni-graph-enforcement.md).
EOF
    echo "  ✅ Created: ${TARGET_ABS}/AGENTS.md"
  else
    # If AGENTS.md exists, verify if Rule 08 is already referenced
    if ! grep -q "08-omni-graph-enforcement" "${TARGET_ABS}/AGENTS.md"; then
      cat << 'EOF' >> "${TARGET_ABS}/AGENTS.md"

---

## 🧭 Omni-Graph Semantic Knowledge Hub Integration
This workspace is monitored and enforced by Omni-Graph.
- Query AST definitions: `http://localhost:8080/api/symbol?name=<sym>`
- Trace references & callers: `http://localhost:8080/api/references?symbol=<sym>`
- Enforced Rule: Read [`.agents/rules/08-omni-graph-enforcement.md`](.agents/rules/08-omni-graph-enforcement.md)
EOF
      echo "  ✅ Appended Omni-Graph guidance to existing: ${TARGET_ABS}/AGENTS.md"
    else
      echo "  ℹ️  Existing AGENTS.md already references Omni-Graph rule in ${TARGET_ABS}/"
    fi
  fi
fi

echo ""
echo "🚀 Omni-Graph Agent Shield successfully deployed to [${TARGET}]!"
