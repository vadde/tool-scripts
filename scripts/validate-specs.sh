#!/usr/bin/env bash
# ===========================================================================
# validate-specs.sh — Validate all specs are complete and well-formed
#
# Usage:
#   ./scripts/validate-specs.sh
#
# Checks:
#   1. Every tool has a spec in specs/catalog/
#   2. Every spec has required sections
#   3. Every tool has STATUS.md with valid status
#   4. Every tool has required files (README, CHANGELOG, Makefile)
#
# Dependencies: grep
# ===========================================================================

set -euo pipefail
IFS=$'\n\t'

readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
readonly TOOLS_DIR="${REPO_ROOT}/tools"
readonly SPECS_DIR="${REPO_ROOT}/specs/catalog"

# ─── Colors ─────────────────────────────────────────────────────────────────
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m'

# ─── Counters ───────────────────────────────────────────────────────────────
ERRORS=0
WARNINGS=0
TOOLS_CHECKED=0

# ─── Functions ──────────────────────────────────────────────────────────────

error() { echo -e "${RED}  ❌ ERROR: ${1}${NC}"; ((ERRORS++)) || true; }
warn()  { echo -e "${YELLOW}  ⚠️  WARN:  ${1}${NC}"; ((WARNINGS++)) || true; }
pass()  { echo -e "${GREEN}  ✅ ${1}${NC}"; }

validate_tool() {
  local tool_name="$1"
  local tool_dir="${TOOLS_DIR}/${tool_name}"
  local spec_file="${SPECS_DIR}/${tool_name}.md"

  echo ""
  echo "🔍 Validating: ${tool_name}"

  ((TOOLS_CHECKED++)) || true

  # Check required files
  local required_files=("README.md" "STATUS.md" "CHANGELOG.md" "Makefile" "spec.md")
  for file in "${required_files[@]}"; do
    if [[ -f "${tool_dir}/${file}" ]]; then
      pass "${file} exists"
    else
      error "${file} missing in tools/${tool_name}/"
    fi
  done

  # Check required directories
  local required_dirs=("src" "tests" "examples")
  for dir in "${required_dirs[@]}"; do
    if [[ -d "${tool_dir}/${dir}" ]]; then
      pass "${dir}/ exists"
    else
      error "${dir}/ missing in tools/${tool_name}/"
    fi
  done

  # Check spec exists
  if [[ -f "${spec_file}" ]]; then
    pass "Spec exists: specs/catalog/${tool_name}.md"

    # Check spec has requirements
    if grep -q "R-001" "${spec_file}"; then
      pass "Spec has numbered requirements"
    else
      warn "Spec missing numbered requirements (R-001, R-002...)"
    fi

    # Check spec has required sections
    local sections=("Overview" "Requirements" "Interface Contract" "Acceptance Criteria")
    for section in "${sections[@]}"; do
      if grep -qi "${section}" "${spec_file}"; then
        pass "Spec has '${section}' section"
      else
        warn "Spec missing '${section}' section"
      fi
    done
  else
    error "Spec missing: specs/catalog/${tool_name}.md"
  fi

  # Validate STATUS.md
  if [[ -f "${tool_dir}/STATUS.md" ]]; then
    local status
    status="$(grep '^status:' "${tool_dir}/STATUS.md" 2>/dev/null | head -1 | sed 's/^status: *//')"
    local valid_statuses="draft spec-review in-progress testing review released deprecated"

    if [[ -n "${status}" ]] && echo "${valid_statuses}" | grep -qw "${status}"; then
      pass "STATUS.md has valid status: ${status}"
    else
      error "STATUS.md has invalid status: '${status}'"
    fi
  fi
}

# ─── Main ───────────────────────────────────────────────────────────────────

main() {
  echo "╔══════════════════════════════════════════════════╗"
  echo "║  📋 Spec & Structure Validator                  ║"
  echo "╚══════════════════════════════════════════════════╝"

  for tool_dir in "${TOOLS_DIR}"/*/; do
    local tool_name
    tool_name="$(basename "${tool_dir}")"

    # Skip the template
    if [[ "${tool_name}" == "_template" ]]; then
      continue
    fi

    validate_tool "${tool_name}"
  done

  # Summary
  echo ""
  echo "╔══════════════════════════════════════════════════╗"
  echo "║  📊 Validation Summary                          ║"
  echo "╠══════════════════════════════════════════════════╣"
  echo "║  Tools checked: ${TOOLS_CHECKED}"
  echo "║  Errors:        ${ERRORS}"
  echo "║  Warnings:      ${WARNINGS}"
  echo "╚══════════════════════════════════════════════════╝"

  if [[ ${TOOLS_CHECKED} -eq 0 ]]; then
    echo ""
    echo "ℹ️  No tools found to validate. Create one with: make new-tool NAME=<name>"
  fi

  if [[ ${ERRORS} -gt 0 ]]; then
    echo ""
    echo -e "${RED}❌ Validation FAILED with ${ERRORS} error(s).${NC}"
    exit 1
  fi

  echo ""
  echo -e "${GREEN}✅ All validations passed!${NC}"
}

main "$@"
