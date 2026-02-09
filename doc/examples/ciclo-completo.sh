#!/usr/bin/env bash
#
# Exemplo: Ciclo de vida completo do MCP
#
# Demonstra o fluxo obrigatorio desde initialize ate encerramento da sessao.
# Prerequisito: o servidor deve estar rodando (cargo run)
#
set -euo pipefail

BASE="http://127.0.0.1:3000/mcp"

echo "=== 1. Initialize ==="
RESPONSE=$(curl -s -D /tmp/mcp_headers -X POST "$BASE" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "initialize",
    "params": {
      "protocolVersion": "2025-06-18",
      "capabilities": {},
      "clientInfo": { "name": "exemplo-shell", "version": "1.0" }
    }
  }')

echo "$RESPONSE" | python3 -m json.tool 2>/dev/null || echo "$RESPONSE"

# Extrair session ID do header da resposta
SESSION_ID=$(grep -i 'mcp-session-id' /tmp/mcp_headers | tr -d '\r' | awk '{print $2}')
echo ""
echo "Session ID: $SESSION_ID"
echo ""

echo "=== 2. Notifications/Initialized ==="
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" -X POST "$BASE" \
  -H "Content-Type: application/json" \
  -H "Mcp-Session-Id: $SESSION_ID" \
  -d '{
    "jsonrpc": "2.0",
    "method": "notifications/initialized"
  }')
echo "HTTP Status: $HTTP_CODE (esperado: 202)"
echo ""

echo "=== 3. Tools/List ==="
curl -s -X POST "$BASE" \
  -H "Content-Type: application/json" \
  -H "Mcp-Session-Id: $SESSION_ID" \
  -d '{
    "jsonrpc": "2.0",
    "id": 2,
    "method": "tools/list"
  }' | python3 -m json.tool 2>/dev/null
echo ""

echo "=== 4. Tools/Call (ping) ==="
curl -s -X POST "$BASE" \
  -H "Content-Type: application/json" \
  -H "Mcp-Session-Id: $SESSION_ID" \
  -d '{
    "jsonrpc": "2.0",
    "id": 3,
    "method": "tools/call",
    "params": { "name": "ping" }
  }' | python3 -m json.tool 2>/dev/null
echo ""

echo "=== 5. Protocol Ping ==="
curl -s -X POST "$BASE" \
  -H "Content-Type: application/json" \
  -H "Mcp-Session-Id: $SESSION_ID" \
  -d '{
    "jsonrpc": "2.0",
    "id": 4,
    "method": "ping"
  }' | python3 -m json.tool 2>/dev/null
echo ""

echo "=== 6. Encerrar Sessao ==="
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" -X DELETE "$BASE" \
  -H "Mcp-Session-Id: $SESSION_ID")
echo "HTTP Status: $HTTP_CODE (esperado: 200)"
echo ""

echo "=== 7. Verificar que sessao foi destruida ==="
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" -X DELETE "$BASE" \
  -H "Mcp-Session-Id: $SESSION_ID")
echo "HTTP Status: $HTTP_CODE (esperado: 404)"

rm -f /tmp/mcp_headers
echo ""
echo "Ciclo completo executado com sucesso."
