# apilab - MCP Gateway Server

Servidor MCP (Model Context Protocol) implementado em Rust com transporte
Streamable HTTP, seguindo a spec 2025-06-18.

## O que e o MCP?

O Model Context Protocol e um protocolo aberto que padroniza a comunicacao entre
aplicacoes de IA e ferramentas externas. Ele usa JSON-RPC 2.0 sobre HTTP,
permitindo que clientes descubram e invoquem tools expostas pelo servidor.

## O que o apilab faz

O apilab e um gateway MCP que:

1. **Gerencia sessoes** -- cada cliente recebe um ID unico via header
   `Mcp-Session-Id`, mantido durante toda a conexao.
2. **Expoe tools** -- ferramentas registradas no servidor que clientes podem
   descobrir (`tools/list`) e invocar (`tools/call`).
3. **Suporta o ciclo de vida completo** -- initialize, notifications/initialized,
   ping, tools/list, tools/call, e encerramento de sessao.
4. **SSE keep-alive** -- endpoint GET para Server-Sent Events (preparado para
   push de mensagens em fases futuras).

Na Fase 1, a unica tool disponivel e `ping`, que retorna `"pong"`.

## Arquitetura

```
apilab/
  dev.sh                 Script para modo desenvolvimento
  prod.sh                Script para modo producao (acesso remoto)
  src/
    main.rs              Ponto de entrada: Axum router, bind configuravel
    lib.rs               AppState (SessionManager + ToolRegistry)
    protocol/
      error.rs           Codigos de erro JSON-RPC 2.0
      jsonrpc.rs         Tipos JSON-RPC: Request, Response, ErrorResponse
      mcp.rs             Tipos MCP: Initialize, Tools, ServerCapabilities
    session/
      state.rs           Struct Session (id, timestamps, initialized)
      manager.rs         SessionManager com RwLock<HashMap>
    tools/
      mod.rs             build_registry() -- monta todas as tools
      registry.rs        ToolRegistry com handlers async
      ping.rs            Tool "ping" -> "pong"
    gateway/
      router.rs          Dispatcher: JSON-RPC method -> handler interno
    transport/
      handler.rs         Handlers Axum: POST/GET/DELETE /mcp
      sse.rs             Helper para eventos SSE
  doc/
    guia.md              Documentacao completa
    referencia-rapida.md Tabelas de consulta rapida
    examples/            Scripts de exemplo executaveis
```

### Fluxo de uma requisicao

```
Cliente                          Servidor
  |                                |
  |  POST /mcp (JSON-RPC)         |
  |------------------------------->|
  |                                |-- valida jsonrpc "2.0"
  |                                |-- extrai Mcp-Session-Id
  |                                |-- valida sessao (exceto initialize)
  |                                |-- dispatch() roteia pelo method
  |                                |-- handler processa
  |  <-----------------------------|
  |  JSON response / 202 Accepted  |
```

## Como compilar e executar

### Pre-requisitos

- Rust toolchain (rustc + cargo)

### Modo desenvolvimento

```bash
./dev.sh
```

- Build debug (compilacao rapida)
- Logs em nivel `debug`
- Bind em `127.0.0.1:3000` (somente local)

### Modo producao

```bash
./prod.sh
```

- Build release (otimizado)
- Logs em nivel `info`
- Bind em `0.0.0.0:3000` (acesso remoto habilitado)

### Comparacao dos modos

|                | `dev.sh`        | `prod.sh`          |
|----------------|-----------------|--------------------|
| Build          | debug           | release (otimizado)|
| Bind           | `127.0.0.1`     | `0.0.0.0`          |
| Logs           | debug           | info               |
| Acesso remoto  | nao             | sim                |

### Variaveis de ambiente

O servidor aceita configuracao via variaveis de ambiente:

| Variavel   | Default       | Descricao                    |
|------------|---------------|------------------------------|
| `MCP_HOST` | `127.0.0.1`   | Endereco de bind             |
| `MCP_PORT` | `3000`        | Porta do servidor            |
| `RUST_LOG` | (nenhum)       | Nivel de log (debug, info, warn, error) |

Os scripts `dev.sh` e `prod.sh` configuram essas variaveis automaticamente,
mas voce pode sobrescreve-las:

```bash
# Dev em porta customizada
MCP_PORT=8080 ./dev.sh

# Prod em porta customizada
MCP_PORT=9000 ./prod.sh

# Manual: acesso remoto com logs debug
MCP_HOST=0.0.0.0 MCP_PORT=4000 RUST_LOG=debug cargo run
```

### Acesso remoto

No modo producao (`prod.sh`), o servidor escuta em `0.0.0.0`, aceitando
conexoes de qualquer interface de rede. Clientes remotos acessam usando o IP
da maquina:

```bash
# Na maquina remota:
curl -s -X POST http://<IP_DO_SERVIDOR>:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"remote-client"}}}'
```

## Endpoint: `/mcp`

O servidor expoe um unico endpoint que aceita tres metodos HTTP:

| Metodo   | Funcao                                      |
|----------|---------------------------------------------|
| `POST`   | Enviar mensagens JSON-RPC (requests e notificacoes) |
| `GET`    | Abrir stream SSE para mensagens do servidor |
| `DELETE` | Encerrar uma sessao                         |

### Headers

| Header            | Quando usar                        |
|-------------------|------------------------------------|
| `Content-Type`    | `application/json` (sempre no POST)|
| `Mcp-Session-Id`  | Em todas as requests apos initialize |

## Metodos JSON-RPC

### `initialize`

Inicia o handshake MCP. Retorna as capabilities do servidor e cria uma sessao.

- **Requer sessao**: Nao
- **Tipo**: Request (tem `id`)
- **Resposta**: `Mcp-Session-Id` no header + `InitializeResult` no body

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "initialize",
  "params": {
    "protocolVersion": "2025-06-18",
    "capabilities": {},
    "clientInfo": { "name": "meu-cliente", "version": "1.0" }
  }
}
```

Resposta:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "protocolVersion": "2025-06-18",
    "capabilities": { "tools": {} },
    "serverInfo": { "name": "apilab", "version": "0.1.0" }
  }
}
```

### `notifications/initialized`

Notificacao que confirma ao servidor que o cliente concluiu o handshake.

- **Requer sessao**: Sim
- **Tipo**: Notificacao (sem `id`)
- **Resposta**: HTTP 202 Accepted (sem body)

```json
{
  "jsonrpc": "2.0",
  "method": "notifications/initialized"
}
```

### `ping`

Ping do protocolo MCP. Verifica se o servidor esta ativo.

- **Requer sessao**: Sim
- **Tipo**: Request (tem `id`)
- **Resposta**: Objeto vazio `{}`

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "ping"
}
```

Resposta:

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {}
}
```

### `tools/list`

Lista todas as tools disponives no servidor.

- **Requer sessao**: Sim
- **Tipo**: Request (tem `id`)
- **Resposta**: Array de tools com nome, descricao e schema

```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/list"
}
```

Resposta:

```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "tools": [
      {
        "name": "ping",
        "description": "Returns pong",
        "inputSchema": { "type": "object", "properties": {} }
      }
    ]
  }
}
```

### `tools/call`

Invoca uma tool pelo nome, passando argumentos opcionais.

- **Requer sessao**: Sim
- **Tipo**: Request (tem `id`)
- **Resposta**: Array de conteudo (text, image, etc.)

```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "tools/call",
  "params": {
    "name": "ping",
    "arguments": {}
  }
}
```

Resposta:

```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "result": {
    "content": [
      { "type": "text", "text": "pong" }
    ]
  }
}
```

### `DELETE /mcp`

Encerra a sessao. Nao envia body, apenas o header `Mcp-Session-Id`.

- **Resposta**: HTTP 200 (sessao destruida) ou HTTP 404 (sessao nao encontrada)

## Codigos de erro JSON-RPC

| Codigo   | Constante          | Significado                |
|----------|--------------------|----------------------------|
| `-32700` | `PARSE_ERROR`      | JSON invalido              |
| `-32600` | `INVALID_REQUEST`  | Request JSON-RPC invalida  |
| `-32601` | `METHOD_NOT_FOUND` | Metodo nao existe          |
| `-32602` | `INVALID_PARAMS`   | Parametros invalidos       |
| `-32603` | `INTERNAL_ERROR`   | Erro interno do servidor   |

Exemplo de resposta de erro:

```json
{
  "jsonrpc": "2.0",
  "id": 99,
  "error": {
    "code": -32601,
    "message": "Method not found"
  }
}
```

## Dependencias

| Crate              | Versao | Funcao                          |
|--------------------|--------|---------------------------------|
| axum               | 0.8    | Framework HTTP + SSE            |
| tokio              | 1      | Runtime async                   |
| serde              | 1      | Serializacao/deserializacao     |
| serde_json         | 1      | Manipulacao JSON                |
| uuid               | 1      | Geracao de IDs de sessao (v4)   |
| tokio-stream       | 0.1    | Adaptadores de stream para SSE  |
| futures-util       | 0.3    | Trait Stream                    |
| tracing            | 0.1    | Logging estruturado             |
| tracing-subscriber | 0.3    | Formatacao de logs              |

## Proximas fases

| Fase | Descricao                                   |
|------|---------------------------------------------|
| 2    | Proxy routing -- encaminhar tools para upstreams |
| 3    | Cliente remoto -- parse de SSE, roteamento bidirecional |
