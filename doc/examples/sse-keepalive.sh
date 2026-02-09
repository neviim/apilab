#!/usr/bin/env bash
#
# Exemplo: SSE keep-alive stream
#
# Demonstra a conexao SSE (GET /mcp) que mantem o canal aberto para
# mensagens futuras do servidor. Na Fase 1 o stream so envia keep-alive.
#
# O script abre o stream por 5 segundos e mostra os eventos recebidos.
# Prerequisito: o servidor deve estar rodando (cargo run)
#
set -euo pipefail

BASE="http://127.0.0.1:3000/mcp"

echo "=== Criar sessao ==="
SESSION_ID=$(curl -s -D - -X POST "$BASE" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "initialize",
    "params": {
      "protocolVersion": "2025-06-18",
      "capabilities": {},
      "clientInfo": { "name": "sse-demo" }
    }
  }' | grep -i 'mcp-session-id' | tr -d '\r' | awk '{print $2}')

curl -s -o /dev/null -X POST "$BASE" \
  -H "Content-Type: application/json" \
  -H "Mcp-Session-Id: $SESSION_ID" \
  -d '{"jsonrpc":"2.0","method":"notifications/initialized"}'

echo "Session ID: $SESSION_ID"
echo ""

echo "=== Abrindo stream SSE (5 segundos) ==="
echo "O servidor envia keep-alive (comentarios SSE ':') a cada 15s."
echo "Em 5s voce pode ver o content-type e o inicio da conexao."
echo ""

timeout 5 curl -s -N -X GET "$BASE" \
  -H "Accept: text/event-stream" \
  -H "Mcp-Session-Id: $SESSION_ID" 2>&1 || true

echo ""
echo "(Stream encerrado apos timeout)"
echo ""

# Limpar
curl -s -o /dev/null -X DELETE "$BASE" -H "Mcp-Session-Id: $SESSION_ID"
echo "Sessao encerrada."
