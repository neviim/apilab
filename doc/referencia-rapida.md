# Referencia Rapida de Comandos

## Compilar e Executar

```bash
# Desenvolvimento (debug, localhost, logs verbose)
./dev.sh

# Producao (release, 0.0.0.0, acesso remoto)
./prod.sh

# Porta customizada
MCP_PORT=8080 ./dev.sh
MCP_PORT=9000 ./prod.sh

# Manual
MCP_HOST=0.0.0.0 MCP_PORT=4000 RUST_LOG=debug cargo run
```

## Modos

|                | `dev.sh`        | `prod.sh`          |
|----------------|-----------------|--------------------|
| Build          | debug           | release (otimizado)|
| Bind           | `127.0.0.1`     | `0.0.0.0`          |
| Logs           | debug           | info               |
| Acesso remoto  | nao             | sim                |

## Variaveis de Ambiente

| Variavel   | Default     | Descricao          |
|------------|-------------|--------------------|
| `MCP_HOST` | `127.0.0.1` | Endereco de bind   |
| `MCP_PORT` | `3000`      | Porta              |
| `RUST_LOG` | (nenhum)    | Nivel de log       |

## Ciclo de Vida MCP (ordem obrigatoria)

```
1. initialize           Cria sessao, retorna Mcp-Session-Id
2. notifications/initialized   Confirma handshake (202)
3. tools/list           Lista tools disponiveis
4. tools/call           Invoca uma tool
5. ping                 Heartbeat do protocolo
6. DELETE /mcp          Encerra sessao
```

## Resumo dos Metodos

| # | Metodo                     | id? | Sessao? | HTTP Response       |
|---|----------------------------|-----|---------|---------------------|
| 1 | `initialize`               | Sim | Nao     | 200 + JSON + header |
| 2 | `notifications/initialized`| Nao | Sim     | 202 Accepted        |
| 3 | `tools/list`               | Sim | Sim     | 200 + JSON          |
| 4 | `tools/call`               | Sim | Sim     | 200 + JSON          |
| 5 | `ping`                     | Sim | Sim     | 200 + JSON          |
| - | `DELETE /mcp`              | -   | Sim     | 200 ou 404          |
| - | `GET /mcp`                 | -   | Sim     | SSE stream          |

## Template de Request

Todas as requests POST seguem esta estrutura:

```bash
curl -s -X POST http://127.0.0.1:3000/mcp \
  -H "Content-Type: application/json" \
  -H "Mcp-Session-Id: <SESSION_ID>" \
  -d '{"jsonrpc":"2.0","id":N,"method":"<METODO>","params":{...}}'
```

- Sem `id` = notificacao (servidor responde 202, sem body)
- Com `id` = request (servidor responde com `result` ou `error`)
- Sem `Mcp-Session-Id` = so funciona para `initialize`

## Tools Disponiveis (Fase 1)

| Tool   | Descricao      | Argumentos | Retorno         |
|--------|----------------|------------|-----------------|
| `ping` | Retorna "pong" | Nenhum     | `{"type":"text","text":"pong"}` |

## Codigos de Erro

| Codigo   | Significado             |
|----------|-------------------------|
| `-32700` | JSON invalido (parse)   |
| `-32600` | Request invalida        |
| `-32601` | Metodo nao encontrado   |
| `-32602` | Parametros invalidos    |
| `-32603` | Erro interno            |

## HTTP Status Codes

| Status | Quando                                     |
|--------|--------------------------------------------|
| `200`  | Request processada com sucesso             |
| `202`  | Notificacao aceita (sem body)              |
| `400`  | Falta header Mcp-Session-Id                |
| `404`  | Sessao nao encontrada                      |
