#!/usr/bin/env bash
#
# Exemplo: Cenarios de erro
#
# Demonstra como o servidor responde a requests invalidas.
# Prerequisito: o servidor deve estar rodando (cargo run)
#
set -uo pipefail

BASE="http://127.0.0.1:3000/mcp"

echo "=== 1. Metodo desconhecido ==="
echo "Enviando method 'foo/bar' para provocar METHOD_NOT_FOUND (-32601)"
echo ""

# Primeiro precisamos de uma sessao valida
SESSION_ID=$(curl -s -D - -X POST "$BASE" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "initialize",
    "params": {
      "protocolVersion": "2025-06-18",
      "capabilities": {},
      "clientInfo": { "name": "teste-erros" }
    }
  }' | grep -i 'mcp-session-id' | tr -d '\r' | awk '{print $2}')

curl -s -X POST "$BASE" \
  -H "Content-Type: application/json" \
  -H "Mcp-Session-Id: $SESSION_ID" \
  -d '{
    "jsonrpc": "2.0",
    "id": 10,
    "method": "foo/bar"
  }' | python3 -m json.tool 2>/dev/null
echo ""

echo "=== 2. Tool inexistente ==="
echo "Chamando tools/call com name 'nao_existe'"
echo ""
curl -s -X POST "$BASE" \
  -H "Content-Type: application/json" \
  -H "Mcp-Session-Id: $SESSION_ID" \
  -d '{
    "jsonrpc": "2.0",
    "id": 11,
    "method": "tools/call",
    "params": { "name": "nao_existe" }
  }' | python3 -m json.tool 2>/dev/null
echo ""

echo "=== 3. Request sem sessao ==="
echo "Enviando tools/list sem header Mcp-Session-Id"
echo ""
HTTP_CODE=$(curl -s -o /tmp/mcp_err -w "%{http_code}" -X POST "$BASE" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 12,
    "method": "tools/list"
  }')
echo "HTTP Status: $HTTP_CODE (esperado: 400)"
cat /tmp/mcp_err | python3 -m json.tool 2>/dev/null
echo ""

echo "=== 4. Sessao invalida ==="
echo "Enviando request com Mcp-Session-Id falso"
echo ""
HTTP_CODE=$(curl -s -o /tmp/mcp_err -w "%{http_code}" -X POST "$BASE" \
  -H "Content-Type: application/json" \
  -H "Mcp-Session-Id: sessao-que-nao-existe" \
  -d '{
    "jsonrpc": "2.0",
    "id": 13,
    "method": "ping"
  }')
echo "HTTP Status: $HTTP_CODE (esperado: 404)"
cat /tmp/mcp_err | python3 -m json.tool 2>/dev/null
echo ""

echo "=== 5. Initialize sem params ==="
echo "Enviando initialize sem campo params"
echo ""
curl -s -X POST "$BASE" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 14,
    "method": "initialize"
  }' | python3 -m json.tool 2>/dev/null
echo ""

# Limpar sessao
curl -s -o /dev/null -X DELETE "$BASE" -H "Mcp-Session-Id: $SESSION_ID"
rm -f /tmp/mcp_err

echo "Todos os cenarios de erro demonstrados."
