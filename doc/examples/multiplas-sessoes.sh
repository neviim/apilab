#!/usr/bin/env bash
#
# Exemplo: Multiplas sessoes simultaneas
#
# Demonstra que o servidor suporta varios clientes conectados ao mesmo tempo,
# cada um com sua propria sessao independente.
# Prerequisito: o servidor deve estar rodando (cargo run)
#
set -euo pipefail

BASE="http://127.0.0.1:3000/mcp"

create_session() {
  local client_name="$1"
  local sid
  sid=$(curl -s -D - -X POST "$BASE" \
    -H "Content-Type: application/json" \
    -d "{
      \"jsonrpc\": \"2.0\",
      \"id\": 1,
      \"method\": \"initialize\",
      \"params\": {
        \"protocolVersion\": \"2025-06-18\",
        \"capabilities\": {},
        \"clientInfo\": { \"name\": \"$client_name\" }
      }
    }" | grep -i 'mcp-session-id' | tr -d '\r' | awk '{print $2}')

  # Enviar initialized
  curl -s -o /dev/null -X POST "$BASE" \
    -H "Content-Type: application/json" \
    -H "Mcp-Session-Id: $sid" \
    -d '{"jsonrpc":"2.0","method":"notifications/initialized"}'

  echo "$sid"
}

echo "=== Criando 3 sessoes simultaneas ==="
echo ""

S1=$(create_session "cliente-alpha")
echo "Sessao 1 (alpha): $S1"

S2=$(create_session "cliente-beta")
echo "Sessao 2 (beta):  $S2"

S3=$(create_session "cliente-gamma")
echo "Sessao 3 (gamma): $S3"
echo ""

echo "=== Cada sessao chama ping independentemente ==="
echo ""

for i in 1 2 3; do
  eval SID=\$S$i
  echo "Sessao $i chamando tools/call ping..."
  curl -s -X POST "$BASE" \
    -H "Content-Type: application/json" \
    -H "Mcp-Session-Id: $SID" \
    -d "{\"jsonrpc\":\"2.0\",\"id\":$i,\"method\":\"tools/call\",\"params\":{\"name\":\"ping\"}}"
  echo ""
done
echo ""

echo "=== Destruindo sessao 2, as outras continuam ativas ==="
echo ""

curl -s -o /dev/null -w "DELETE sessao 2: HTTP %{http_code}\n" -X DELETE "$BASE" \
  -H "Mcp-Session-Id: $S2"

echo "Sessao 1 (ping):"
curl -s -X POST "$BASE" \
  -H "Content-Type: application/json" \
  -H "Mcp-Session-Id: $S1" \
  -d '{"jsonrpc":"2.0","id":10,"method":"ping"}'
echo ""

echo "Sessao 2 (deve falhar):"
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" -X POST "$BASE" \
  -H "Content-Type: application/json" \
  -H "Mcp-Session-Id: $S2" \
  -d '{"jsonrpc":"2.0","id":11,"method":"ping"}')
echo "HTTP Status: $HTTP_CODE (esperado: 404)"

echo "Sessao 3 (ping):"
curl -s -X POST "$BASE" \
  -H "Content-Type: application/json" \
  -H "Mcp-Session-Id: $S3" \
  -d '{"jsonrpc":"2.0","id":12,"method":"ping"}'
echo ""
echo ""

# Limpar
curl -s -o /dev/null -X DELETE "$BASE" -H "Mcp-Session-Id: $S1"
curl -s -o /dev/null -X DELETE "$BASE" -H "Mcp-Session-Id: $S3"

echo "Todas as sessoes encerradas."
