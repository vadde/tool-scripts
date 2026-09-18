#!/usr/bin/env bash
# ===========================================================================
# update-catalog.sh — Regenerate the tool catalog in tools/README.md
#
# Usage:
#   ./scripts/update-catalog.sh
#
# Reads STATUS.md from each tool and rebuilds the catalog table.
#
# Dependencies: grep, sed, awk
# ===========================================================================

set -euo pipefail
IFS=$'\n\t'

readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
readonly TOOLS_DIR="${REPO_ROOT}/tools"
readonly CATALOG_FILE="${TOOLS_DIR}/README.md"

# ─── Functions ──────────────────────────────────────────────────────────────

extract_field() {
  local file="$1"
  local field="$2"
  grep "^${field}:" "${file}" 2>/dev/null | head -1 | sed "s/^${field}: *//" | tr -d '"' | tr -d "'"
}

get_tool_description() {
  local readme="$1"
  # Extract the first line after the title that starts with ">"
  sed -n '/^>/p' "${readme}" 2>/dev/null | head -1 | sed 's/^> *//'
}

# ─── Main ───────────────────────────────────────────────────────────────────

main() {
  echo "📚 Regenerating tool catalog..."
  echo ""

  local catalog_lines=()
  local tool_count=0

  for tool_dir in "${TOOLS_DIR}"/*/; do
    local tool_name
    tool_name="$(basename "${tool_dir}")"

    # Skip the template
    if [[ "${tool_name}" == "_template" ]]; then
      continue
    fi

    local status_file="${tool_dir}/STATUS.md"
    local readme_file="${tool_dir}/README.md"

    if [[ ! -f "${status_file}" ]]; then
      echo "  ⚠️  Skipping ${tool_name}: no STATUS.md"
      continue
    fi

    local status version language category description
    status="$(extract_field "${status_file}" "status")"
    version="$(extract_field "${status_file}" "version")"
    language="$(extract_field "${status_file}" "language")"
    category="$(extract_field "${status_file}" "category")"

    if [[ -f "${readme_file}" ]]; then
      description="$(get_tool_description "${readme_file}")"
    else
      description="_No description_"
    fi

    catalog_lines+=("| [${tool_name}](${tool_name}/) | ${category:-—} | ${language:-—} | \`${status:-draft}\` v${version:-0.0.0} | ${description:-—} |")
    ((tool_count++)) || true

    echo "  ✅ ${tool_name} (${status:-draft}, ${language:-unknown})"
  done

  # Build the new catalog section
  local new_catalog=""
  if [[ ${tool_count} -eq 0 ]]; then
    new_catalog="| _No tools yet_ | — | — | — | _Use \`make new-tool NAME=<name>\` to create your first tool_ |"
  else
    new_catalog="$(printf '%s\n' "${catalog_lines[@]}")"
  fi

  # Replace the catalog section in README.md
  local temp_file
  temp_file="$(mktemp)"

  awk -v catalog="${new_catalog}" '
    /<!-- CATALOG-START/ { print; in_catalog=1; next }
    /<!-- CATALOG-END/ { print catalog; in_catalog=0 }
    !in_catalog { print }
  ' "${CATALOG_FILE}" > "${temp_file}"

  mv "${temp_file}" "${CATALOG_FILE}"

  # Update quick stats
  echo ""
  echo "📊 Catalog updated: ${tool_count} tool(s)"
  echo "✅ Done!"
}

main "$@"
