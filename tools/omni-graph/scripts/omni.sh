#!/usr/bin/env bash
# ==============================================================================
# omni.sh — Command Line Interface for Omni-Graph
# Provides fast agent & user friendly commands for semantic search, AST
# condensation, community clustering, multi-workspace ingestion, and health checks.
# ==============================================================================

set -euo pipefail

# Ensure python3 is available (required for JSON formatting)
command -v python3 >/dev/null 2>&1 || { echo "❌ python3 is required but not found. Install Python 3."; exit 1; }

ENDPOINT="${OMNI_API_URL:-http://localhost:8080}"

usage() {
  cat <<'EOF'
Usage: omni.sh <command> [args...]

Commands:
  health                     Check service health (Rust API, SurrealDB, TEI)
  workspaces                 List all indexed codebases/workspaces and stats
  search <query> [ws] [k]    Vector similarity search (optional workspace filter)
  symbol <name> [ws]         Symbol definition lookup (LSP definition)
  references <symbol> [ws]   Find all callers/references (LSP references)
  condense <sym> [ws] [hops] Multi-hop AST subgraph slice (<1500 tokens for agents)
  query <prompt> [ws] [k]    Hybrid Graph-RAG retrieval (seeds + AST + community)
  cluster [workspace]        Run Louvain/Leiden community detection clustering
  galaxies [workspace]       Inspect architectural galaxy subsystems
  ingest <path> [project]    Index a codebase/directory into SurrealDB
  stats                      Show aggregated graph stats
EOF
  exit 1
}

CMD="${1:-}"
shift || true

norm_ws() {
  local raw="${1:-}"
  if [ -n "${raw}" ] && [[ "${raw}" == *"/"* ]]; then
    local trimmed="${raw%/}"
    echo "${trimmed##*/}"
  else
    echo "${raw}"
  fi
}

case "${CMD}" in
  health)
    curl -sf "${ENDPOINT}/api/health" | python3 -m json.tool 2>/dev/null || curl -s "${ENDPOINT}/api/health"
    ;;

  workspaces)
    python3 -c "
import urllib.request, json
url = '${ENDPOINT}/api/workspaces'
req = urllib.request.Request(url)
try:
    with urllib.request.urlopen(req) as resp:
        data = json.loads(resp.read().decode())
        if not data:
            print('No codebases ingested yet. Run: make ingest PATH=/path/to/codebase')
            exit(0)
        print(f'{\"WORKSPACE\":<25} {\"TOTAL NODES\":<15} {\"LANGUAGES\":<20} {\"FILES\":<10}')
        print('='*75)
        for item in data:
            ws = item.get('workspace', 'default')
            nodes = item.get('total_nodes', 0)
            langs = ', '.join(item.get('languages', []))
            files = len(item.get('files', []))
            print(f'{ws:<25} {nodes:<15} {langs:<20} {files:<10}')
except Exception as e:
    print(f'Error connecting to Omni-Graph API: {e}')
"
    ;;

  search)
    QUERY="${1:-}"
    WS="$(norm_ws "${2:-}")"
    K="${3:-10}"
    if [ -z "${QUERY}" ]; then echo "❌ Missing query argument"; exit 1; fi
    python3 -c "
import urllib.request, urllib.parse, json, sys
params = {'q': sys.argv[1], 'k': sys.argv[3]}
if sys.argv[2]:
    params['workspace'] = sys.argv[2]
url = '${ENDPOINT}/api/search?' + urllib.parse.urlencode(params)
req = urllib.request.Request(url)
with urllib.request.urlopen(req) as resp:
    data = json.loads(resp.read().decode())
    print(json.dumps(data, indent=2))
" "${QUERY}" "${WS}" "${K}"
    ;;

  symbol)
    SYM="${1:-}"
    WS="$(norm_ws "${2:-}")"
    if [ -z "${SYM}" ]; then echo "❌ Missing symbol name"; exit 1; fi
    python3 -c "
import urllib.request, urllib.parse, json, sys
params = {'name': sys.argv[1]}
if sys.argv[2]:
    params['workspace'] = sys.argv[2]
url = '${ENDPOINT}/api/symbol?' + urllib.parse.urlencode(params)
req = urllib.request.Request(url)
with urllib.request.urlopen(req) as resp:
    data = json.loads(resp.read().decode())
    print(json.dumps(data, indent=2))
" "${SYM}" "${WS}"
    ;;

  references)
    SYM="${1:-}"
    WS="$(norm_ws "${2:-}")"
    if [ -z "${SYM}" ]; then echo "❌ Missing symbol name"; exit 1; fi
    python3 -c "
import urllib.request, urllib.parse, json, sys
params = {'symbol': sys.argv[1]}
if sys.argv[2]:
    params['workspace'] = sys.argv[2]
url = '${ENDPOINT}/api/references?' + urllib.parse.urlencode(params)
req = urllib.request.Request(url)
with urllib.request.urlopen(req) as resp:
    data = json.loads(resp.read().decode())
    print(json.dumps(data, indent=2))
" "${SYM}" "${WS}"
    ;;

  condense)
    SYMBOL="${1:-}"
    WS="$(norm_ws "${2:-}")"
    HOPS="${3:-2}"
    if [ -z "${SYMBOL}" ]; then echo "❌ Missing symbol argument"; exit 1; fi
    python3 -c "
import urllib.request, urllib.parse, json, sys
params = {'symbol': sys.argv[1], 'hops': sys.argv[3]}
if sys.argv[2]:
    params['workspace'] = sys.argv[2]
url = '${ENDPOINT}/api/condense?' + urllib.parse.urlencode(params)
req = urllib.request.Request(url)
with urllib.request.urlopen(req) as resp:
    data = json.loads(resp.read().decode())
    print(data.get('formatted_markdown', ''))
" "${SYMBOL}" "${WS}" "${HOPS}"
    ;;

  query)
    PROMPT="${1:-}"
    WS="$(norm_ws "${2:-}")"
    K="${3:-5}"
    if [ -z "${PROMPT}" ]; then echo "❌ Missing prompt argument"; exit 1; fi
    python3 -c "
import urllib.request, json, sys
url = '${ENDPOINT}/api/query'
payload_data = {'prompt': sys.argv[1], 'top_k': int(sys.argv[3])}
if sys.argv[2]:
    payload_data['workspace'] = sys.argv[2]
payload = json.dumps(payload_data).encode('utf-8')
req = urllib.request.Request(url, data=payload, headers={'Content-Type': 'application/json'})
with urllib.request.urlopen(req) as resp:
    data = json.loads(resp.read().decode())
    print('╔══════════════════════════════════════════════════════════════════════════╗')
    print('║                🏛️  MACROSCOPIC ARCHITECTURAL SUMMARY                      ║')
    print('╚══════════════════════════════════════════════════════════════════════════╝')
    print(data.get('macroscopic_summary', ''))
    print('\n╔══════════════════════════════════════════════════════════════════════════╗')
    print('║                🔬 CONDENSED AST SUBGRAPH SLICE                           ║')
    print('╚══════════════════════════════════════════════════════════════════════════╝')
    print(data.get('expanded_subgraph', ''))
" "${PROMPT}" "${WS}" "${K}"
    ;;

  cluster)
    WS="$(norm_ws "${1:-}")"
    python3 -c "
import urllib.request, json, sys
url = '${ENDPOINT}/api/cluster'
payload_data = {}
if sys.argv[1]:
    payload_data['workspace'] = sys.argv[1]
payload = json.dumps(payload_data).encode('utf-8')
req = urllib.request.Request(url, data=payload, headers={'Content-Type': 'application/json'})
try:
    with urllib.request.urlopen(req) as resp:
        data = json.loads(resp.read().decode())
        print('✅ Community clustering completed successfully.')
        print(f\"Total Communities Detected: {data.get('total_communities', 0)}\")
        if data.get('workspace'):
            print(f\"Workspace: {data.get('workspace')}\")
        print('')
        print(f'{\"COMMUNITY ID\":<15} {\"NODES\":<10} {\"NAME / SUBSYSTEM\":<40}')
        print('='*75)
        for comm in data.get('communities', []):
            cid = comm.get('id', 0)
            cnt = comm.get('node_count', 0)
            name = comm.get('name', '')
            print(f'#{cid:<14} {cnt:<10} {name:<40}')
except Exception as e:
    print(f'Error executing cluster: {e}')
" "${WS}"
    ;;

  galaxies)
    WS="$(norm_ws "${1:-}")"
    python3 -c "
import urllib.request, urllib.parse, json, sys
url = '${ENDPOINT}/api/galaxies'
if sys.argv[1]:
    url += '?' + urllib.parse.urlencode({'workspace': sys.argv[1]})
req = urllib.request.Request(url)
try:
    with urllib.request.urlopen(req) as resp:
        data = json.loads(resp.read().decode())
        galaxies = data.get('galaxies', [])
        ws = data.get('workspace') or 'All Workspaces (Omniverse)'
        print(f'🌌 Architectural Subsystems for [{ws}] — Total Galaxies: {len(galaxies)}\n')
        if not galaxies:
            print('No clusters computed yet. Run: make cluster')
            exit(0)
        print(f'{\"GALAXY ID\":<12} {\"NODES\":<8} {\"SUBSYSTEM / DOMINANT PATH\":<35} {\"LANGUAGES\":<15} {\"SAMPLE SYMBOLS\"}')
        print('='*105)
        for g in galaxies:
            gid = f\"#{g.get('id', 0)}\"
            cnt = g.get('node_count', 0)
            name = g.get('name', '')
            if len(name) > 33: name = name[:30] + '...'
            langs = ', '.join(g.get('languages', []))
            syms = ', '.join(g.get('sample_symbols', [])[:4])
            if len(syms) > 40: syms = syms[:37] + '...'
            print(f'{gid:<12} {cnt:<8} {name:<35} {langs:<15} {syms}')
except Exception as e:
    print(f'Error querying galaxies: {e}')
" "${WS}"
    ;;

  ingest)
    RAW_PATH="${1:-/workspace}"
    PROJECT="${2:-}"

    # Guardrails: Check path validity before making Docker API call
    if [ "${RAW_PATH}" != "/workspace" ]; then
      if [ -f "${RAW_PATH}" ]; then
        echo "❌ ERROR: Target path is a file, not a directory: ${RAW_PATH}"
        echo "💡 Omni-Graph indexes entire directory trees. Did you mean: $(dirname "${RAW_PATH}")"
        exit 1
      fi

      if [ ! -e "${RAW_PATH}" ]; then
        echo "❌ ERROR: Target directory does not exist on host:"
        echo "   '${RAW_PATH}'"
        PARENT_DIR="$(dirname "${RAW_PATH}")"
        BASE_NAME="$(basename "${RAW_PATH}")"
        if [ -d "${PARENT_DIR}" ]; then
          echo ""
          echo "💡 Existing folders in '${PARENT_DIR}':"
          find "${PARENT_DIR}" -maxdepth 1 -mindepth 1 -type d | grep -i "${BASE_NAME:0:3}" | head -n 5 | while IFS= read -r match; do
            echo "   • ${match}"
          done || true
        fi
        exit 1
      fi

      RESOLVED_PATH="$(cd "${RAW_PATH}" && pwd)"

      case "${RESOLVED_PATH}" in
        / | /root | /bin | /sbin | /usr | /System* | /Library* | /Applications* | /etc* | /var* | /private*)
          echo "🛑 ERROR: Refusing to ingest protected system directory: ${RESOLVED_PATH}"
          exit 1
          ;;
      esac
    else
      RESOLVED_PATH="${RAW_PATH}"
    fi

    echo "📡 Sending ingestion request for: ${RESOLVED_PATH}"
    if [ -n "${PROJECT}" ]; then
      echo "🏷️  Target workspace namespace: ${PROJECT}"
    fi

    python3 -c "
import urllib.request, urllib.error, json, sys
url = '${ENDPOINT}/api/ingest'
payload_data = {'path': sys.argv[1]}
if sys.argv[2]:
    payload_data['project'] = sys.argv[2]
payload = json.dumps(payload_data).encode('utf-8')
req = urllib.request.Request(url, data=payload, headers={'Content-Type': 'application/json'})
try:
    with urllib.request.urlopen(req, timeout=300) as resp:
        res = json.loads(resp.read().decode())
        result = res.get('result', {})
        print('\n✅ Ingestion complete!')
        print(f\"  • Workspace:     {result.get('workspace')}\")
        print(f\"  • Files Scanned: {result.get('files_scanned')}\")
        print(f\"  • Files Indexed: {result.get('files_indexed')}\")
        print(f\"  • Files Skipped: {result.get('files_skipped')} (unchanged cache)\")
        print(f\"  • Nodes Created: {result.get('nodes_created')}\")
        print(f\"  • Edges Created: {result.get('edges_created')}\")
        if result.get('clusters_computed') is not None:
            print(f\"  • Clusters:      {result.get('clusters_computed')} modular galaxy subsystems\")
        print(f\"  • Duration:      {result.get('duration_ms')}ms\")
except urllib.error.HTTPError as e:
    body = e.read().decode()
    print(f'❌ Ingestion failed ({e.code}): {body}')
    sys.exit(1)
except urllib.error.URLError as e:
    print(f'❌ Ingestion failed (Connection error): {e.reason}')
    print('   Please check if omni-graph stack is running: make up')
    sys.exit(1)
except Exception as e:
    print(f'❌ Ingestion failed: {e}')
    sys.exit(1)
" "${RESOLVED_PATH}" "${PROJECT}"
    ;;

  stats)
    curl -sf "${ENDPOINT}/api/stats" | python3 -m json.tool 2>/dev/null || curl -s "${ENDPOINT}/api/stats"
    ;;

  *)
    usage
    ;;
esac
