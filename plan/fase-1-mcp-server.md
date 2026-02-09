# Plano: MCP Gateway Server em Rust - Fase 1

## Contexto

Criar um servidor MCP (Model Context Protocol) from scratch em Rust usando Axum, com transporte Streamable HTTP (spec 2025-06-18). O workspace `/home/neviim/developer/apilab` e um repositorio git vazio. A primeira funcionalidade e uma tool "ping" que retorna "pong", junto com o ciclo de vida completo do MCP (initialize, tools/list, tools/call, protocol ping).

Este e o alicerce para as fases futuras: proxy de roteamento (Fase 2) e cliente remoto (Fase 3).

---

## Estrutura do Projeto

```
apilab/
  Cargo.toml
  src/
    main.rs                    -- Bootstrap: Axum router, bind configuravel via env
    lib.rs                     -- AppState, re-exports dos modulos
    protocol/
      mod.rs
      error.rs                 -- Constantes de erro JSON-RPC (-32700, -32600, etc)
      jsonrpc.rs               -- Tipos JSON-RPC 2.0 (Request, Response, ErrorResponse)
      mcp.rs                   -- Tipos MCP (Initialize, Tools, ServerCapabilities)
    transport/
      mod.rs
      handler.rs               -- Handlers Axum: POST/GET/DELETE /mcp
      sse.rs                   -- Helper para criar eventos SSE
    session/
      mod.rs
      state.rs                 -- Struct Session (id, protocol_version, initialized, timestamps)
      manager.rs               -- SessionManager com RwLock<HashMap>
    tools/
      mod.rs                   -- build_registry() com todas as tools
      registry.rs              -- ToolRegistry com async handlers (boxed futures)
      ping.rs                  -- Tool "ping" -> "pong"
    gateway/
      mod.rs
      router.rs                -- Dispatcher: JSON-RPC method -> handler
```

## Dependencias (Cargo.toml)

| Crate | Versao | Motivo |
|-------|--------|--------|
| axum | 0.8 (features: json) | Framework HTTP + SSE nativo |
| tokio | 1 (features: full) | Runtime async |
| serde | 1 (features: derive) | Serialization JSON-RPC |
| serde_json | 1 | Manipulacao JSON |
| uuid | 1 (features: v4) | IDs de sessao |
| tokio-stream | 0.1 | Stream adapters para SSE |
| futures-util | 0.3 | Stream trait |
| tracing + tracing-subscriber | 0.1 / 0.3 | Logging estruturado |

## Sequencia de Implementacao

### Passo 1: Scaffold do projeto
- Criar `Cargo.toml` com dependencias
- Criar `src/main.rs` e `src/lib.rs` minimos
- Validar: `cargo check`

### Passo 2: Tipos do protocolo
- `src/protocol/mod.rs` - declaracoes de modulo
- `src/protocol/error.rs` - constantes PARSE_ERROR, INVALID_REQUEST, METHOD_NOT_FOUND, INVALID_PARAMS, INTERNAL_ERROR
- `src/protocol/jsonrpc.rs` - `JsonRpcRequest` (id como `Option<Value>`), `JsonRpcResponse`, `JsonRpcErrorResponse`
- `src/protocol/mcp.rs` - `InitializeParams/Result`, `ServerCapabilities`, `Tool`, `ToolCallParams/Result`, `ToolContent` (tagged enum)
- Ponto critico: `#[serde(rename_all = "camelCase")]` em todos os structs MCP
- Validar: `cargo check`

### Passo 3: Gerenciamento de sessao
- `src/session/state.rs` - struct `Session` com id, protocol_version, initialized, timestamps
- `src/session/manager.rs` - `SessionManager` com `RwLock<HashMap>`, metodos create/with/with_mut/destroy
- Decisao: `std::sync::RwLock` (nao tokio) pois o lock nunca cruza .await
- Validar: `cargo check`

### Passo 4: Registry de tools + ping
- `src/tools/registry.rs` - `ToolRegistry` com `HashMap<String, (Tool, ToolHandler)>`, metodos register/list/call
- `ToolHandler` = `Box<dyn Fn(Option<Value>) -> Pin<Box<dyn Future<Output=ToolCallResult> + Send>> + Send + Sync>`
- `src/tools/ping.rs` - definicao da tool (name: "ping", schema vazio) + handler async retornando `ToolContent::Text { text: "pong" }`
- `src/tools/mod.rs` - `build_registry()` registra a ping tool
- Validar: `cargo check`

### Passo 5: Gateway dispatcher
- `src/gateway/router.rs` - enum `DispatchResult` (Response/Accepted/Error) + funcao `dispatch()` que roteia:
  - `"initialize"` -> cria InitializeResult com capabilities (tools)
  - `"notifications/initialized"` -> marca sessao como initialized, retorna Accepted
  - `"ping"` -> protocolo ping, retorna `{}` vazio
  - `"tools/list"` -> lista do registry
  - `"tools/call"` -> chama tool pelo nome via registry
  - `_` -> METHOD_NOT_FOUND
- Validar: `cargo check`

### Passo 6: Transport HTTP/SSE
- `src/transport/sse.rs` - helper `json_rpc_event()` para wrapping SSE
- `src/transport/handler.rs` - tres handlers Axum:
  - `post_mcp`: valida jsonrpc "2.0", extrai Mcp-Session-Id, valida sessao (exceto initialize), despacha, retorna JSON ou 202
  - `get_mcp`: valida sessao, retorna SSE stream com keep-alive (stream::pending para Fase 1)
  - `delete_mcp`: destroi sessao, retorna 200 ou 404
- Na resposta de initialize: cria sessao e insere header `Mcp-Session-Id`
- Validar: `cargo check`

### Passo 7: Conectar tudo no main.rs
- `src/lib.rs` - struct `AppState { session_manager: Arc<SessionManager>, tool_registry: Arc<ToolRegistry> }`
- `src/main.rs` - init tracing, build state, Router com `.route("/mcp", post().get().delete())`, bind configuravel via MCP_HOST/MCP_PORT
- Validar: `cargo run` (servidor sobe)

### Passo 8: Testes manuais com curl
- Testar fluxo completo: initialize -> initialized -> tools/list -> tools/call ping -> protocol ping -> delete session

## Verificacao (Testes com curl)

```bash
# 1. Initialize
curl -s -D - -X POST http://127.0.0.1:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"test"}}}'
# Esperado: 200 + header Mcp-Session-Id + result com serverInfo

# 2. Initialized (notificacao, sem id)
curl -s -D - -X POST http://127.0.0.1:3000/mcp \
  -H "Content-Type: application/json" \
  -H "Mcp-Session-Id: <SESSION_ID>" \
  -d '{"jsonrpc":"2.0","method":"notifications/initialized"}'
# Esperado: 202 Accepted

# 3. List tools
curl -s -X POST http://127.0.0.1:3000/mcp \
  -H "Content-Type: application/json" \
  -H "Mcp-Session-Id: <SESSION_ID>" \
  -d '{"jsonrpc":"2.0","id":2,"method":"tools/list"}'
# Esperado: tools array com "ping"

# 4. Call ping tool
curl -s -X POST http://127.0.0.1:3000/mcp \
  -H "Content-Type: application/json" \
  -H "Mcp-Session-Id: <SESSION_ID>" \
  -d '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"ping"}}'
# Esperado: {"content":[{"type":"text","text":"pong"}]}

# 5. Protocol ping
curl -s -X POST http://127.0.0.1:3000/mcp \
  -H "Content-Type: application/json" \
  -H "Mcp-Session-Id: <SESSION_ID>" \
  -d '{"jsonrpc":"2.0","id":4,"method":"ping"}'
# Esperado: {"result":{}}
```

## Decisoes Tecnicas

| Decisao | Motivo |
|---------|--------|
| `std::sync::RwLock` (nao tokio) | Lock nunca cruza .await, evita overhead |
| `ToolHandler` como boxed closure | Permite handlers async heterogeneos no HashMap |
| `ToolContent` com `#[serde(tag = "type")]` | Serializa como `{"type":"text","text":"..."}` conforme spec MCP |
| GET /mcp com `stream::pending` | Placeholder para SSE; fases futuras adicionam channels |
| Variaveis de ambiente para config | `MCP_HOST`, `MCP_PORT`, `RUST_LOG` -- simples, sem arquivo de config |

## Pontos de Extensao para Fases Futuras

| Componente | Fase 2 (Proxy) | Fase 3 (Cliente) |
|-----------|----------------|-------------------|
| `ToolRegistry` | Adicionar `ProxiedTool` que encaminha para upstream | Abstracoes unificadas local + remoto |
| `SessionManager` | Tabela de roteamento por sessao | Tracking de sessao no cliente |
| `gateway/router.rs` | Se tool nao e local, rotear para upstream | Roteamento bidirecional |
| GET `/mcp` | Push de mensagens de upstreams via channels | Parse de SSE no cliente |

## Status

- [x] Passo 1: Scaffold do projeto
- [x] Passo 2: Tipos do protocolo
- [x] Passo 3: Gerenciamento de sessao
- [x] Passo 4: Registry de tools + ping
- [x] Passo 5: Gateway dispatcher
- [x] Passo 6: Transport HTTP/SSE
- [x] Passo 7: Conectar tudo no main.rs
- [x] Passo 8: Testes manuais com curl

**Fase 1 concluida.**
