#!/usr/bin/env bash
# ===========================================================================
# run-all-tests.sh — Discover and run tests across all tools
#
# Usage:
#   ./scripts/run-all-tests.sh [tool-name]
#
# If tool-name is provided, only that tool's tests are run.
# Otherwise, all tools with a Makefile `test` target are tested.
#
# Dependencies: make
# ===========================================================================

set -euo pipefail
IFS=$'\n\t'

readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
readonly TOOLS_DIR="${REPO_ROOT}/tools"

# ─── Colors ─────────────────────────────────────────────────────────────────
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# ─── Functions ──────────────────────────────────────────────────────────────

log_info() { echo -e "${CYAN}ℹ️  ${1}${NC}"; }
log_pass() { echo -e "${GREEN}✅ ${1}${NC}"; }
log_fail() { echo -e "${RED}❌ ${1}${NC}"; }
log_skip() { echo -e "${YELLOW}⏭️  ${1}${NC}"; }

run_tool_tests() {
  local tool_name="$1"
  local tool_dir="${TOOLS_DIR}/${tool_name}"

  if [[ ! -f "${tool_dir}/Makefile" ]]; then
    log_skip "${tool_name}: No Makefile found"
    return 0
  fi

  if ! grep -q '^test:' "${tool_dir}/Makefile"; then
    log_skip "${tool_name}: No 'test' target in Makefile"
    return 0
  fi

  log_info "Testing: ${tool_name}"

  if make -C "${tool_dir}" test 2>&1; then
    log_pass "${tool_name}: All tests passed"
    return 0
  else
    log_fail "${tool_name}: Tests FAILED"
    return 1
  fi
}

# ─── Main ───────────────────────────────────────────────────────────────────

main() {
  local total=0
  local passed=0
  local failed=0
  local skipped=0
  local failed_tools=()

  echo ""
  echo "╔══════════════════════════════════════════════════╗"
  echo "║  🧪 Tool-Scripts — Test Runner                  ║"
  echo "╚══════════════════════════════════════════════════╝"
  echo ""

  # If specific tool provided
  if [[ $# -ge 1 ]]; then
    local tool="$1"
    if [[ ! -d "${TOOLS_DIR}/${tool}" ]]; then
      log_fail "Tool not found: tools/${tool}/"
      exit 1
    fi
    run_tool_tests "${tool}"
    exit $?
  fi

  # Run all tools
  for tool_dir in "${TOOLS_DIR}"/*/; do
    local tool_name
    tool_name="$(basename "${tool_dir}")"

    # Skip the template
    if [[ "${tool_name}" == "_template" ]]; then
      continue
    fi

    ((total++)) || true

    if run_tool_tests "${tool_name}"; then
      if [[ -f "${tool_dir}/Makefile" ]] && grep -q '^test:' "${tool_dir}/Makefile"; then
        ((passed++)) || true
      else
        ((skipped++)) || true
      fi
    else
      ((failed++)) || true
      failed_tools+=("${tool_name}")
    fi

    echo ""
  done

  # Summary
  echo "╔══════════════════════════════════════════════════╗"
  echo "║  📊 Test Summary                                ║"
  echo "╠══════════════════════════════════════════════════╣"
  echo "║  Total:   ${total}"
  echo "║  Passed:  ${passed}"
  echo "║  Failed:  ${failed}"
  echo "║  Skipped: ${skipped}"
  echo "╚══════════════════════════════════════════════════╝"

  if [[ ${failed} -gt 0 ]]; then
    echo ""
    log_fail "Failed tools: ${failed_tools[*]}"
    exit 1
  fi

  if [[ ${total} -eq 0 ]]; then
    log_info "No tools found to test."
  fi
}

main "$@"
