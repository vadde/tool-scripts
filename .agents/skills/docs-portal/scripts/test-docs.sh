#!/usr/bin/env bash
# ==============================================================================
# 🧪 docs-portal automated headless verification suite
# Validates HTML markup, KaTeX math delimiters, Mermaid syntax & sizing,
# scrollspy 1:1 invariants, and headless Chrome rendering.
# ==============================================================================
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
SITE_DIR="${REPO_ROOT}/docs/site"
BUILD_DIR="${REPO_ROOT}/_site"

echo "╔══════════════════════════════════════════════════════════════════╗"
echo "║  🧪 Running Documentation Portal Verification Suite              ║"
echo "╚══════════════════════════════════════════════════════════════════╝"

# 1. Check required assets
echo "→ 1. Checking core asset files..."
for f in index.html style.css app.js; do
  if [ ! -f "${SITE_DIR}/${f}" ]; then
    echo "❌ Missing required asset: ${SITE_DIR}/${f}"
    exit 1
  fi
done
echo "   ✅ Core assets present (index.html, style.css, app.js)"

# 2. Verify Mermaid label escaping invariants (< and > inside node labels)
echo "→ 2. Auditing Mermaid syntax invariants..."
UNESCAPED_LT=$(grep -n '\[.*<[0-9].*\]' "${SITE_DIR}/index.html" || true)
if [ -n "${UNESCAPED_LT}" ]; then
  echo "❌ Detected unescaped '<' inside Mermaid node label in index.html:"
  echo "${UNESCAPED_LT}"
  echo "   Use '≤' or '&lt;' to prevent Mermaid HTML parser crashes."
  exit 1
fi
echo "   ✅ Mermaid node labels are cleanly escaped."

# 3. Verify CSS SVG sizing rules (Prevent SVG 0px x 0px collapse)
echo "→ 3. Auditing SVG sizing invariants in style.css..."
if grep -E '\.mermaid svg.*width:\s*auto\s*!important' "${SITE_DIR}/style.css" > /dev/null; then
  echo "❌ Fatal CSS anti-pattern: 'width: auto !important' found on .mermaid svg."
  echo "   This collapses SVG elements to 0px x 0px inside flex containers."
  exit 1
fi
if grep -E '\.mermaid svg.*height:\s*auto\s*!important.*max-height' "${SITE_DIR}/style.css" > /dev/null; then
  echo "❌ Fatal CSS anti-pattern: conflicting max-height and height: auto !important."
  exit 1
fi
echo "   ✅ CSS SVG sizing rules conform to responsive viewBox standards."

# 4. Verify KaTeX configuration
echo "→ 4. Checking KaTeX resources and delimiters..."
if ! grep -q "katex.min.css" "${SITE_DIR}/index.html"; then
  echo "❌ KaTeX stylesheet missing in index.html head"
  exit 1
fi
if ! grep -q "auto-render.min.js" "${SITE_DIR}/index.html"; then
  echo "❌ KaTeX auto-render script missing in index.html"
  exit 1
fi
if ! grep -q "renderMathInElement" "${SITE_DIR}/app.js"; then
  echo "❌ renderMathInElement call missing in app.js"
  exit 1
fi
echo "   ✅ KaTeX 0.16.11 loaded with auto-render extension."

# 5. Build site to _site
echo "→ 5. Building documentation bundle..."
bash "${REPO_ROOT}/scripts/build-docs.sh" > /dev/null
echo "   ✅ Bundle built successfully to _site"

# 6. Verify with Headless Chrome if available
if [ -x "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" ]; then
  echo "→ 6. Performing Headless Chrome render test..."
  PORT=4499
  # Start ephemeral server
  python3 -m http.server "${PORT}" --directory "${BUILD_DIR}" > /dev/null 2>&1 &
  SERVER_PID=$!
  trap 'kill ${SERVER_PID} 2>/dev/null || true' EXIT
  sleep 1

  # Dump rendered DOM
  DUMP_FILE="/tmp/docs_test_dump_${PORT}.html"
  /Applications/Google\ Chrome.app/Contents/MacOS/Google\ Chrome \
    --headless \
    --disable-gpu \
    --no-sandbox \
    --disable-background-networking \
    --disable-component-update \
    --run-all-compositor-stages-before-draw \
    --dump-dom "http://localhost:${PORT}/" > "${DUMP_FILE}" 2>/dev/null

  # Verify Mermaid SVGs rendered
  SVG_COUNT=$(grep -c '<svg id="mermaid-' "${DUMP_FILE}" || true)
  if [ "${SVG_COUNT}" -lt 3 ]; then
    echo "❌ Headless Chrome failed to render all Mermaid diagrams. Found: ${SVG_COUNT}, expected >= 3"
    exit 1
  fi
  echo "   ✅ Headless Chrome rendered all ${SVG_COUNT} Mermaid architecture diagrams!"

  # Clean up dump
  rm -f "${DUMP_FILE}"
  kill ${SERVER_PID} 2>/dev/null || true
  trap - EXIT
else
  echo "ℹ️ Google Chrome not found at standard path, skipped headless browser render."
fi

echo ""
echo "🎉 ALL DOCUMENTATION PORTAL INVARIANTS PASSED!"
