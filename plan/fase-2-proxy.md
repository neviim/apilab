# Plano: MCP Gateway Server em Rust - Fase 2 (Proxy Routing)

## Contexto

A Fase 1 entregou um servidor MCP funcional com tool local (ping) e ciclo de
vida completo. A Fase 2 transforma o apilab em um **gateway proxy**: ele se
conecta a servidores MCP upstream, descobre as tools deles, e as expoe aos
clientes como se fossem suas. O cliente fala com um unico endpoint e o gateway
roteia transparentemente.

### Pre-requisito

- Fase 1 concluida (servidor MCP com ping, sessoes, JSON-RPC)

---

## Visao Geral

```
Cliente MCP                   apilab (gateway)                Upstream A
    |                              |                              |
    |  tools/list  --------------->|                              |
    |  <--- ping + tools de A + B |                              |
    |                              |                              |
    |  tools/call "search"  ------>|  (search e de A)             |
    |                              |  tools/call "search" ------->|
    |                              |  <--- resultado -------------|
    |  <--- resultado              |                              |
    |                              |                         Upstream B
    |  tools/call "ping" -------->|  (ping e local)              |
    |  <--- "pong"                 |                              |
```

O gateway age como **multiplexador**: um cliente conecta uma vez e tem acesso
a tools de N servidores upstream + tools locais.

---

## Estrutura de Arquivos Novos

```
src/
  config/
    mod.rs                     -- Modulo de configuracao
    upstream.rs                -- Struct UpstreamConfig, parse de arquivo/env
  upstream/
    mod.rs                     -- Re-exports
    client.rs                  -- Cliente MCP HTTP (initialize, tools/list, tools/call)
    session.rs                 -- Gestao de sessoes com upstreams
    discovery.rs               -- Conecta nos upstreams, descobre tools, registra no registry
  tools/
    proxied.rs                 -- ProxiedTool handler (encaminha tools/call para upstream)
```

### Arquivos Modificados

```
src/
  lib.rs                       -- AppState ganha UpstreamManager
  main.rs                      -- Carrega config, inicializa upstreams antes de servir
  tools/
    mod.rs                     -- build_registry() registra tools locais + proxied
    registry.rs                -- Tool ganha campo opcional `origin` (local/upstream)
  gateway/
    router.rs                  -- tools/list merge local + upstream
```

---

## Dependencias Novas

| Crate | Versao | Motivo |
|-------|--------|--------|
| reqwest | 0.12 (features: json, rustls-tls) | Cliente HTTP para upstreams |
| toml | 0.8 | Parse de arquivo de configuracao |

---

## Configuracao de Upstreams

Arquivo `apilab.toml` na raiz do projeto:

```toml
[[upstream]]
name = "server-a"
url = "http://192.168.1.10:3000/mcp"

[[upstream]]
name = "server-b"
url = "http://192.168.1.20:3000/mcp"
```

Struct correspondente:

```rust
struct UpstreamConfig {
    name: String,
    url: String,
}

struct Config {
    upstreams: Vec<UpstreamConfig>,
}
```

Fallback: se `apilab.toml` nao existir, o servidor inicia sem upstreams
(somente tools locais, comportamento identico a Fase 1).

---

## Componentes

### 1. Cliente MCP Upstream (`upstream/client.rs`)

Cliente HTTP que fala o protocolo MCP com um servidor upstream:

- `connect(url)` -- faz `initialize` + `notifications/initialized`, armazena `Mcp-Session-Id`
- `list_tools()` -- chama `tools/list` no upstream
- `call_tool(name, arguments)` -- chama `tools/call` no upstream e retorna o resultado
- `disconnect()` -- envia `DELETE /mcp` para encerrar sessao no upstream
- Usa `reqwest::Client` com keep-alive

### 2. Sessao Upstream (`upstream/session.rs`)

Gerencia as sessoes que o gateway mantem com cada upstream:

```rust
struct UpstreamSession {
    name: String,
    url: String,
    session_id: String,          // Mcp-Session-Id do upstream
    client: reqwest::Client,
    tools: Vec<Tool>,            // Tools descobertas nesse upstream
}

struct UpstreamManager {
    sessions: RwLock<HashMap<String, UpstreamSession>>,  // key = upstream name
}
```

### 3. Descoberta de Tools (`upstream/discovery.rs`)

Na inicializacao do gateway:

1. Para cada upstream no config:
   - Conecta (initialize + initialized)
   - Lista tools (`tools/list`)
   - Registra cada tool como `ProxiedTool` no `ToolRegistry`
2. Se um upstream estiver offline, loga warning e continua (graceful degradation)
3. O registro inclui a origem: `Tool { name, description, input_schema, origin }`

### 4. ProxiedTool (`tools/proxied.rs`)

Handler que encaminha a chamada para o upstream:

```rust
// Pseudo-codigo
fn create_proxied_handler(upstream_manager: Arc<UpstreamManager>, upstream_name: String) -> ToolHandler {
    Box::new(move |args| {
        let mgr = upstream_manager.clone();
        let name = upstream_name.clone();
        Box::pin(async move {
            mgr.call_tool(&name, &tool_name, args).await
        })
    })
}
```

O cliente MCP nao sabe se a tool e local ou proxied -- a resposta tem o mesmo
formato.

### 5. Merge no tools/list

Quando o cliente chama `tools/list`, o gateway retorna:

- Tools locais (ping, etc.)
- Tools de cada upstream (com prefixo opcional para evitar colisao de nomes)

Estrategia de colisao de nomes:

| Estrategia | Exemplo | Complexidade |
|-----------|---------|--------------|
| Sem prefixo (first wins) | `search` | Simples, risco de colisao |
| Prefixo por upstream | `server-a/search` | Seguro, mas muda nomes |
| Namespace no metadata | `search` + campo origin | Transparente, cliente decide |

Decisao a tomar: qual estrategia usar.

---

## Sequencia de Implementacao

### Passo 1: Configuracao
- Criar `src/config/mod.rs` e `src/config/upstream.rs`
- Parse de `apilab.toml` com fallback (sem upstreams = Fase 1)
- Adicionar `toml` ao Cargo.toml
- Validar: `cargo check`

### Passo 2: Cliente MCP upstream
- Criar `src/upstream/client.rs` com reqwest
- Implementar connect, list_tools, call_tool, disconnect
- Adicionar `reqwest` ao Cargo.toml
- Validar: `cargo check`

### Passo 3: UpstreamManager
- Criar `src/upstream/session.rs` com UpstreamSession e UpstreamManager
- Metodos: add, get, remove, list
- Validar: `cargo check`

### Passo 4: Descoberta de tools
- Criar `src/upstream/discovery.rs`
- Na inicializacao: conecta em cada upstream, descobre tools
- Registra ProxiedTool handlers no ToolRegistry
- Validar: `cargo check`

### Passo 5: ProxiedTool handler
- Criar `src/tools/proxied.rs`
- Handler async que encaminha tools/call via UpstreamManager
- Validar: `cargo check`

### Passo 6: Integrar no main.rs
- Carregar config
- Inicializar UpstreamManager
- Descobrir tools dos upstreams
- Adicionar UpstreamManager ao AppState
- Validar: `cargo run`

### Passo 7: Testes
- Subir uma instancia apilab como upstream (porta 3001)
- Subir o gateway apontando para ela (porta 3000)
- `tools/list` no gateway deve mostrar tools locais + do upstream
- `tools/call` no gateway deve encaminhar para o upstream
- Testar upstream offline (graceful degradation)

---

## Decisoes Tecnicas

| Decisao | Opcoes | Recomendacao |
|---------|--------|--------------|
| Cliente HTTP | reqwest vs hyper direto | reqwest (mais ergonomico, connection pooling built-in) |
| Config format | TOML vs JSON vs env vars | TOML (legivel, padrao Rust) |
| Colisao de nomes | Prefixo vs first-wins vs metadata | Definir antes do passo 5 |
| Upstream offline | Falhar tudo vs graceful degradation | Graceful (loga warning, ignora) |
| Reconexao | Manual vs automatica | Manual na Fase 2, automatica na Fase 3 |

---

## Verificacao (Testes com curl)

```bash
# Terminal 1: upstream na porta 3001
MCP_PORT=3001 ./dev.sh

# Terminal 2: gateway na porta 3000 (com apilab.toml apontando para localhost:3001)
./dev.sh

# Terminal 3: testes

# 1. Initialize no gateway
curl -s -D - -X POST http://127.0.0.1:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"test"}}}'

# 2. tools/list -- deve incluir tools do upstream
curl -s -X POST http://127.0.0.1:3000/mcp \
  -H "Content-Type: application/json" \
  -H "Mcp-Session-Id: <SESSION_ID>" \
  -d '{"jsonrpc":"2.0","id":2,"method":"tools/list"}'

# 3. tools/call de tool do upstream -- gateway encaminha transparentemente
curl -s -X POST http://127.0.0.1:3000/mcp \
  -H "Content-Type: application/json" \
  -H "Mcp-Session-Id: <SESSION_ID>" \
  -d '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"ping"}}'
```

---

## Pontos de Extensao para Fase 3

| Componente | Fase 3 (Cliente Remoto) |
|-----------|-------------------------|
| `upstream/client.rs` | Adicionar parse de SSE para receber push do upstream |
| `UpstreamManager` | Bidirecional: receber notificacoes dos upstreams |
| `GET /mcp` | Encaminhar eventos dos upstreams para o cliente via SSE |
| Config | Suportar upstreams dinamicos (adicionar/remover via API) |

---

## Status

- [ ] Passo 1: Configuracao (TOML + structs)
- [ ] Passo 2: Cliente MCP upstream (reqwest)
- [ ] Passo 3: UpstreamManager
- [ ] Passo 4: Descoberta de tools
- [ ] Passo 5: ProxiedTool handler
- [ ] Passo 6: Integrar no main.rs
- [ ] Passo 7: Testes
