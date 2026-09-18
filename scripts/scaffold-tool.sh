#!/usr/bin/env bash
# ===========================================================================
# scaffold-tool.sh — Create a new tool from the canonical template
#
# Usage:
#   ./scripts/scaffold-tool.sh <tool-name>
#
# This script:
#   1. Copies tools/_template/ to tools/<tool-name>/
#   2. Creates specs/catalog/<tool-name>.md from template
#   3. Updates tool-specific placeholders
#   4. Adds entry to tools/README.md catalog
#
# Dependencies: sed, cp, date
# ===========================================================================

set -euo pipefail
IFS=$'\n\t'

# ─── Constants ──────────────────────────────────────────────────────────────
readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
readonly TEMPLATE_DIR="${REPO_ROOT}/tools/_template"
readonly SPECS_TEMPLATE="${REPO_ROOT}/specs/_templates/tool-spec.md"
readonly CATALOG_FILE="${REPO_ROOT}/tools/README.md"
readonly TODAY="$(date +%Y-%m-%d)"

# ─── Functions ──────────────────────────────────────────────────────────────

die() {
  echo "❌ ERROR: ${1}" >&2
  exit "${2:-1}"
}

usage() {
  echo "Usage: $(basename "$0") <tool-name>"
  echo ""
  echo "Arguments:"
  echo "  tool-name    Name of the tool (kebab-case, e.g., json-validator)"
  echo ""
  echo "Example:"
  echo "  $(basename "$0") json-validator"
}

validate_name() {
  local name="$1"
  if [[ ! "$name" =~ ^[a-z][a-z0-9-]*[a-z0-9]$ ]]; then
    die "Tool name must be kebab-case (lowercase, hyphens, start with letter): '${name}'"
  fi
  if [[ ${#name} -lt 2 ]]; then
    die "Tool name must be at least 2 characters: '${name}'"
  fi
}

# ─── Main ───────────────────────────────────────────────────────────────────

main() {
  # Parse arguments
  if [[ $# -lt 1 ]] || [[ "$1" == "-h" ]] || [[ "$1" == "--help" ]]; then
    usage
    exit 0
  fi

  local tool_name="$1"
  local tool_dir="${REPO_ROOT}/tools/${tool_name}"
  local spec_file="${REPO_ROOT}/specs/catalog/${tool_name}.md"

  # Validate
  validate_name "${tool_name}"

  if [[ -d "${tool_dir}" ]]; then
    die "Tool directory already exists: tools/${tool_name}/"
  fi

  if [[ ! -d "${TEMPLATE_DIR}" ]]; then
    die "Template directory not found: ${TEMPLATE_DIR}"
  fi

  echo "🔧 Scaffolding new tool: ${tool_name}"
  echo ""

  # Step 1: Copy template
  echo "  📁 Creating tools/${tool_name}/..."
  cp -r "${TEMPLATE_DIR}" "${tool_dir}"

  # Step 2: Update STATUS.md
  echo "  📋 Configuring STATUS.md..."
  if command -v sed &>/dev/null; then
    sed -i'' -e "s/tool: template/tool: ${tool_name}/" "${tool_dir}/STATUS.md"
    sed -i'' -e "s/created: 2026-01-01/created: ${TODAY}/" "${tool_dir}/STATUS.md"
    sed -i'' -e "s/last_updated: 2026-01-01/last_updated: ${TODAY}/" "${tool_dir}/STATUS.md"
    sed -i'' -e "s|spec: ../../specs/catalog/template.md|spec: ../../specs/catalog/${tool_name}.md|" "${tool_dir}/STATUS.md"
    sed -i'' -e "s/YYYY-MM-DD/${TODAY}/" "${tool_dir}/STATUS.md"
  fi

  # Step 2b: Configure CONTEXT.md
  echo "  🧭 Configuring CONTEXT.md..."
  if command -v sed &>/dev/null; then
    sed -i'' -e "s/tool: template/tool: ${tool_name}/" "${tool_dir}/CONTEXT.md"
    sed -i'' -e "s/Tool Context — template/Tool Context — ${tool_name}/" "${tool_dir}/CONTEXT.md"
    sed -i'' -e "s|specs/catalog/template.md|specs/catalog/${tool_name}.md|" "${tool_dir}/CONTEXT.md"
    sed -i'' -e "s/YYYY-MM-DD/${TODAY}/g" "${tool_dir}/CONTEXT.md"
  fi

  # Step 2c: Configure DEVLOG.md
  echo "  📓 Configuring DEVLOG.md..."
  if command -v sed &>/dev/null; then
    sed -i'' -e "s/Development Log — template/Development Log — ${tool_name}/" "${tool_dir}/DEVLOG.md"
  fi

  # Step 3: Create spec
  echo "  📝 Creating specs/catalog/${tool_name}.md..."
  cp "${SPECS_TEMPLATE}" "${spec_file}"
  if command -v sed &>/dev/null; then
    sed -i'' -e "s|\[Tool Name\]|${tool_name}|" "${spec_file}"
    sed -i'' -e "s|tools/<tool-name>/|tools/${tool_name}/|" "${spec_file}"
    sed -i'' -e "s/YYYY-MM-DD/${TODAY}/g" "${spec_file}"
  fi

  # Step 4: Update tool README
  echo "  📖 Updating tool README.md..."
  if command -v sed &>/dev/null; then
    sed -i'' -e "s/# Tool Name/# ${tool_name}/" "${tool_dir}/README.md"
    sed -i'' -e "s|tools/<tool-name>|tools/${tool_name}|" "${tool_dir}/README.md"
    sed -i'' -e "s|<tool-name>|${tool_name}|g" "${tool_dir}/spec.md"
  fi

  # Step 5: Update catalog
  echo "  📚 Updating tool catalog..."
  # Insert before the CATALOG-END marker
  if command -v sed &>/dev/null; then
    sed -i'' -e "s|_No tools yet.*|<!-- (removed placeholder) -->|" "${CATALOG_FILE}"
    sed -i'' -e "s|<!-- CATALOG-END -->|| [${tool_name}](${tool_name}/) | Utilities | — | \`draft\` | _Description pending_ |\n<!-- CATALOG-END -->|" "${CATALOG_FILE}"
  fi

  echo ""
  echo "✅ Tool '${tool_name}' scaffolded successfully!"
  echo ""
  echo "Next steps:"
  echo "  1. Edit the spec:     \$EDITOR specs/catalog/${tool_name}.md"
  echo "  2. Set the language:  \$EDITOR tools/${tool_name}/Makefile"
  echo "  3. Write the code:    \$EDITOR tools/${tool_name}/src/"
  echo "  4. Write tests:       \$EDITOR tools/${tool_name}/tests/"
  echo "  5. Commit:            git add tools/${tool_name} specs/catalog/${tool_name}.md"
}

main "$@"
