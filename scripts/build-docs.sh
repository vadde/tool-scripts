#!/usr/bin/env bash
# ==============================================================================
# build-docs.sh — Build Static Living Documentation Portal
# Packages docs/site/ into _site/ for GitHub Pages deployment and local serving.
# ==============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
SITE_DIR="${ROOT_DIR}/_site"
DOCS_SITE_DIR="${ROOT_DIR}/docs/site"

echo "╔══════════════════════════════════════════════════════════════════╗"
echo "║  📚 Building tool-scripts Living Documentation Portal            ║"
echo "╚══════════════════════════════════════════════════════════════════╝"

# 1. Clean and prepare output directory
rm -rf "${SITE_DIR}"
mkdir -p "${SITE_DIR}"
mkdir -p "${SITE_DIR}/raw"

# 2. Copy the liquid glass static portal
echo "  → Copying interactive liquid-glass portal assets..."
cp -r "${DOCS_SITE_DIR}/"* "${SITE_DIR}/"

# 3. Copy raw markdown books and guides for archive & deep links
echo "  → Archiving raw markdown guides and architecture ADRs..."
if [ -d "${ROOT_DIR}/docs/book" ]; then
  cp -r "${ROOT_DIR}/docs/book" "${SITE_DIR}/raw/"
fi
if [ -d "${ROOT_DIR}/docs/architecture" ]; then
  cp -r "${ROOT_DIR}/docs/architecture" "${SITE_DIR}/raw/"
fi
if [ -d "${ROOT_DIR}/docs/guides" ]; then
  cp -r "${ROOT_DIR}/docs/guides" "${SITE_DIR}/raw/"
fi

# 4. Verify integrity
if [ ! -f "${SITE_DIR}/index.html" ] || [ ! -f "${SITE_DIR}/style.css" ] || [ ! -f "${SITE_DIR}/app.js" ]; then
  echo "❌ Error: Required static assets missing in ${SITE_DIR}" >&2
  exit 1
fi

TOTAL_FILES=$(find "${SITE_DIR}" -type f | wc -l | tr -d ' ')
TOTAL_SIZE=$(du -sh "${SITE_DIR}" | cut -f1)

echo "✅ Documentation built successfully!"
echo "  • Output path: ${SITE_DIR}"
echo "  • Total files: ${TOTAL_FILES}"
echo "  • Total size:  ${TOTAL_SIZE}"
echo ""
echo "To serve locally:"
echo "  make docs-serve"
echo "  or: python3 -m http.server 4000 --directory _site"
