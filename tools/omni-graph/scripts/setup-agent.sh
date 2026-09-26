#!/usr/bin/env bash
# ==============================================================================
# setup-agent.sh — Omni-Graph Agent Enforcement Bootstrapper
# Supports:
#   1. Current Workspace:  ./scripts/setup-agent.sh [workspace]
#   2. Global System:      ./scripts/setup-agent.sh global
#   3. Target Directory:   ./scripts/setup-agent.sh /path/to/any/codebase
# ==============================================================================

set -euo pipefail

TARGET="${1:-workspace}"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
TOOL_DIR="${REPO_ROOT}/tools/omni-graph"

echo "⚙️  Configuring Omni-Graph Agent Enforcement Shield..."
echo "  → Target mode: ${TARGET}"

# Ensure source scripts are executable
chmod +x "${TOOL_DIR}/scripts/hook_pre_tool.sh"
chmod +x "${TOOL_DIR}/scripts/hook_pre_invocation.sh"
chmod +x "${TOOL_DIR}/scripts/hook_stop.sh"
chmod +x "${TOOL_DIR}/scripts/omni.sh"
chmod +x "${TOOL_DIR}/skills/omni-graph/scripts/omni.sh"

generate_hooks_json() {
  local script_dir="$1"
  local output_file="$2"
  cat << EOF > "${output_file}"
{
  "omni-graph-guardrail": {
    "enabled": true,
    "PreToolUse": [
      {
        "matcher": "run_command|grep_search",
        "hooks": [
          {
            "type": "command",
            "command": "${script_dir}/hook_pre_tool.sh",
            "timeout": 5
          }
        ]
      }
    ],
    "PreInvocation": [
      {
        "type": "command",
        "command": "${script_dir}/hook_pre_invocation.sh",
        "timeout": 5
      }
    ],
    "Stop": [
      {
        "type": "command",
        "command": "${script_dir}/hook_stop.sh",
        "timeout": 5
      }
    ]
  }
}
EOF
}

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

  # 4. Configure global hooks with absolute path to ~/.gemini/config/scripts
  generate_hooks_json "${DEST_DIR}/scripts" "${DEST_DIR}/hooks.json"
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
  generate_hooks_json "${DEST_DIR}/scripts" "${DEST_DIR}/hooks.json"
  echo "  ✅ Configured lifecycle hooks: .agents/hooks.json"

else
  # Arbitrary target directory specified by user
  TARGET_PATH="${TARGET}"
  if [[ -d "${TARGET_PATH}" ]]; then
    TARGET_ABS="$(cd "${TARGET_PATH}" && pwd)"
  else
    mkdir -p "${TARGET_PATH}"
    TARGET_ABS="$(cd "${TARGET_PATH}" && pwd)"
  fi

  echo "  → Resolved target codebase: ${TARGET_ABS}"
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

  # 4. Configure hooks.json with absolute script paths so it works reliably
  generate_hooks_json "${DEST_DIR}/scripts" "${DEST_DIR}/hooks.json"
  echo "  ✅ Configured lifecycle hooks: ${TARGET_ABS}/.agents/hooks.json"

  # 5. Provide or update AGENTS.md in the target codebase if needed
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
    echo "  ℹ️  Existing AGENTS.md preserved in ${TARGET_ABS}/"
  fi
fi

echo ""
echo "🚀 Omni-Graph Agent Shield successfully deployed to [${TARGET}]!"
