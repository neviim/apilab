#!/usr/bin/env bash
#
# Modo desenvolvimento: debug build, logs verbose, somente localhost
#
set -euo pipefail

export RUST_LOG="${RUST_LOG:-debug}"
export MCP_HOST="127.0.0.1"
export MCP_PORT="${MCP_PORT:-3000}"

echo "=== apilab DEV ==="
echo "  bind: ${MCP_HOST}:${MCP_PORT}"
echo "  log:  ${RUST_LOG}"
echo ""

exec cargo run
