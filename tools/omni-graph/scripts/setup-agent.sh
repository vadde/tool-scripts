#!/usr/bin/env bash
# ==============================================================================
# setup-agent.sh — Omni-Graph Agent Enforcement Bootstrapper
# Implements: Dual Setup (Workspace .agents/ and Global Antigravity Profile)
# ==============================================================================

set -euo pipefail

TARGET="${1:-workspace}"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
TOOL_DIR="${REPO_ROOT}/tools/omni-graph"

echo "⚙️  Configuring Omni-Graph Agent Enforcement Shield..."
echo "  → Target mode: ${TARGET}"

# Ensure hook scripts are executable
chmod +x "${TOOL_DIR}/scripts/hook_pre_tool.sh"
chmod +x "${TOOL_DIR}/scripts/hook_pre_invocation.sh"
chmod +x "${TOOL_DIR}/scripts/hook_stop.sh"

if [[ "${TARGET}" == "workspace" ]]; then
  DEST_DIR="${REPO_ROOT}/.agents"
  echo "  → Target directory: ${DEST_DIR}"

  mkdir -p "${DEST_DIR}/rules"
  mkdir -p "${DEST_DIR}/skills/omni-graph/scripts"

  # 1. Install Workspace Rule
  cp "${TOOL_DIR}/rules/08-omni-graph-enforcement.md" "${DEST_DIR}/rules/08-omni-graph-enforcement.md"
  echo "  ✅ Installed rule: .agents/rules/08-omni-graph-enforcement.md"

  # 2. Install Workspace Skill
  cp -r "${TOOL_DIR}/skills/omni-graph/"* "${DEST_DIR}/skills/omni-graph/"
  echo "  ✅ Installed skill: .agents/skills/omni-graph/"

  # 3. Configure hooks.json
  cp "${TOOL_DIR}/hooks.json" "${DEST_DIR}/hooks.json"
  echo "  ✅ Configured lifecycle hooks: .agents/hooks.json"

elif [[ "${TARGET}" == "global" ]]; then
  DEST_DIR="${HOME}/.gemini/config"
  echo "  → Target directory: ${DEST_DIR}"

  mkdir -p "${DEST_DIR}/rules"
  mkdir -p "${DEST_DIR}/skills/omni-graph/scripts"

  # 1. Install Global Rule
  cp "${TOOL_DIR}/rules/08-omni-graph-enforcement.md" "${DEST_DIR}/rules/08-omni-graph-enforcement.md"
  echo "  ✅ Installed global rule: ~/.gemini/config/rules/08-omni-graph-enforcement.md"

  # 2. Install Global Skill
  cp -r "${TOOL_DIR}/skills/omni-graph/"* "${DEST_DIR}/skills/omni-graph/"
  echo "  ✅ Installed global skill: ~/.gemini/config/skills/omni-graph/"

  # 3. Configure global hooks
  cp "${TOOL_DIR}/hooks.json" "${DEST_DIR}/hooks.json"
  echo "  ✅ Configured global hooks: ~/.gemini/config/hooks.json"
fi

echo ""
echo "🚀 Omni-Graph Agent Shield successfully deployed to [${TARGET}]!"
