#!/usr/bin/env bash
# ==============================================================================
# Omni-Graph Stop Hook
# Allows standard termination unless background tasks or health checks require hold
# ==============================================================================

echo '{"decision": "stop"}'
