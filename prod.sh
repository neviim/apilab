#!/usr/bin/env bash
#
# Modo producao: release build, logs info, acesso remoto (0.0.0.0)
#
set -euo pipefail

export RUST_LOG="${RUST_LOG:-info}"
export MCP_HOST="0.0.0.0"
export MCP_PORT="${MCP_PORT:-3000}"

echo "=== apilab PROD ==="
echo "  bind: ${MCP_HOST}:${MCP_PORT}"
echo "  log:  ${RUST_LOG}"
echo ""

cargo build --release 2>&1
exec ./target/release/apilab
