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

  # Check required files (including new governance files)
  local required_files=("README.md" "STATUS.md" "CHANGELOG.md" "Makefile" "spec.md" "CONTEXT.md" "DEVLOG.md")
  for file in "${required_files[@]}"; do
    if [[ -f "${tool_dir}/${file}" ]]; then
      pass "${file} exists"
    else
      if [[ "${file}" == "CONTEXT.md" ]] || [[ "${file}" == "DEVLOG.md" ]]; then
        error "${file} missing in tools/${tool_name}/ (required by Rule 07)"
      else
        error "${file} missing in tools/${tool_name}/"
      fi
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

    # Check spec has Impl/Tested tracking columns
    if grep -q "Impl" "${spec_file}" && grep -q "Tested" "${spec_file}"; then
      pass "Spec has implementation tracking columns"
    else
      warn "Spec missing Impl/Tested tracking columns (per Rule 07)"
    fi
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

    # Phase-appropriate checks
    if [[ "${status}" == "in-progress" ]] || [[ "${status}" == "testing" ]] || \
       [[ "${status}" == "review" ]] || [[ "${status}" == "released" ]]; then
      # Must have traceability matrix
      if grep -q "Traceability Matrix" "${tool_dir}/STATUS.md"; then
        pass "STATUS.md has traceability matrix"
      else
        warn "STATUS.md missing traceability matrix (expected for ${status} phase)"
      fi
    fi

    if [[ "${status}" == "released" ]]; then
      # CHANGELOG must have a version entry
      if grep -qE '^\#\# \[' "${tool_dir}/CHANGELOG.md" 2>/dev/null; then
        pass "CHANGELOG.md has version entries"
      else
        error "Released tool must have CHANGELOG version entries"
      fi
      # No TODOs or FIXMEs
      if grep -rq "TODO\|FIXME\|TBD\|HACK" "${tool_dir}/src/" 2>/dev/null; then
        error "Released tool has TODO/FIXME/TBD/HACK in source code"
      else
        pass "No TODO/FIXME markers in source"
      fi
    fi
  fi

  # Validate CONTEXT.md frontmatter consistency
  if [[ -f "${tool_dir}/CONTEXT.md" ]] && [[ -f "${tool_dir}/STATUS.md" ]]; then
    local context_status
    context_status="$(grep '^status:' "${tool_dir}/CONTEXT.md" 2>/dev/null | head -1 | sed 's/^status: *//')"
    local status_status
    status_status="$(grep '^status:' "${tool_dir}/STATUS.md" 2>/dev/null | head -1 | sed 's/^status: *//')"

    if [[ -n "${context_status}" ]] && [[ -n "${status_status}" ]]; then
      if [[ "${context_status}" == "${status_status}" ]]; then
        pass "CONTEXT.md and STATUS.md statuses are in sync"
      else
        error "Status mismatch: CONTEXT.md='${context_status}' vs STATUS.md='${status_status}'"
      fi
    fi
  fi

  # Check DEVLOG.md has content (not just template) for in-progress+ tools
  if [[ -f "${tool_dir}/DEVLOG.md" ]]; then
    local status
    status="$(grep '^status:' "${tool_dir}/STATUS.md" 2>/dev/null | head -1 | sed 's/^status: *//')"
    if [[ "${status}" == "in-progress" ]] || [[ "${status}" == "testing" ]] || \
       [[ "${status}" == "review" ]] || [[ "${status}" == "released" ]]; then
      if grep -q "^### " "${tool_dir}/DEVLOG.md" 2>/dev/null; then
        pass "DEVLOG.md has session entries"
      else
        warn "DEVLOG.md has no session entries (expected for ${status} phase)"
      fi
    fi
  fi

  # Check catalog entry exists
  if grep -q "${tool_name}" "${TOOLS_DIR}/README.md" 2>/dev/null; then
    pass "Tool appears in catalog (tools/README.md)"
  else
    error "Tool missing from catalog (tools/README.md)"
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
