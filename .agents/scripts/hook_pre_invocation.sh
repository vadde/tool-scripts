#!/usr/bin/env bash
# ==============================================================================
# Omni-Graph PreInvocation Hook
# Injects ephemeral system guidance alerting the model that Omni-Graph AST
# knowledge hub is online for semantic search and call tracing.
# ==============================================================================

set -e

# Fast check if Omni-Graph API is responding on port 8080
if curl -sf --connect-timeout 1 http://localhost:8080/api/health >/dev/null 2>&1; then
    cat << 'EOF'
{
  "injectSteps": [
    {
      "ephemeralMessage": "🧭 [Omni-Graph Active]: Query http://localhost:8080/api/search or /api/graph for structural code intelligence instead of raw text file walking."
    }
  ]
}
EOF
else
    # Output empty injectSteps when offline
    echo '{"injectSteps": []}'
fi
