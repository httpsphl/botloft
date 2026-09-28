# Botloft: especificação v0.1

**TL;DR:** Botloft mantém "tripulações" de bots Claude Code sempre ligados no Windows. Um daemon Rust (`botloftd`) roda os bots em ConPTY, guarda tudo em SQLite e entrega mensagens entre bots pelo inbox nativo do Claude Code (named pipe). Um app Tauri + React mostra os terminais e controla tudo via JSON-RPC sobre WebSocket. O MVP cobre crews, bots, terminal e mensagens duráveis; rotinas, caixa de perguntas e acesso remoto vêm depois.

Status: rascunho para implementação. Este documento é a fonte de verdade do projeto. Quando código e spec divergirem, corrige-se um dos dois no mesmo PR.

---

## 1. Objetivo e princípios

Botloft é um workspace desktop, Windows-first e open-source, para rodar vários agentes Claude Code persistentes que colaboram entre si.

Princípios:

1. **Daemon independente do app.** Fechar o app não derruba bot nenhum. O daemon é a única fonte de verdade.
2. **Claude Code é o runtime.** Nada de SDK próprio de agente. O terminal nativo do Claude Code é preservado, com prompts de permissão e tudo.
3. **Terminal e mensagens em canais separados.** O que o usuário digita vai pela PTY. Mensagens de outros bots chegam pelo inbox do Claude Code, nunca injetadas no terminal.
4. **Durável antes de entregar.** Toda mensagem é gravada antes de sair. Entrega é pelo menos uma vez, com retry.
5. **Sem serviços externos.** SQLite local, sem Redis, sem nuvem, sem telemetria no MVP.
6. **Testável sem gastar token.** Um runtime falso implementa o mesmo contrato do runtime real.
7. **Windows de primeira classe.** Nada de camada de compatibilidade Unix. Linux/macOS podem vir depois atrás de `cfg`.

## 2. Glossário

| Termo | Significado |
|---|---|
| **Crew** | Grupo de bots que podem conversar entre si. Tem pasta compartilhada `shared/`. |
| **Bot** | Uma sessão Claude Code persistente com nome, papel e instruções. |
| **Workspace** | Pasta de trabalho do bot, onde o Claude Code roda. |
| **Message** | Texto enviado a um bot, pelo usuário ou por outro bot. |
| **Delivery** | Tentativa de entregar uma message a um bot. Tem estado, tentativas e prazo. |
| **Task** | Message que pede trabalho e espera um resultado de volta. |
| **Generation** | Contador que incrementa a cada vez que o processo de um bot é (re)iniciado. |
| **Owner** | O usuário humano dono da instalação. |

## 3. Arquitetura

```
 App (Tauri + React + xterm.js)
        │  JSON-RPC 2.0 sobre WebSocket  (127.0.0.1:45710/rpc)
        ▼
 botloftd ─────────────────────────────────────────────────────
   rpc/        sessões WS, autenticação, métodos e notificações
   supervisor/ ciclo de vida dos bots, estados, backoff
   runtime/    ConPTY (real) e FakeRuntime (testes)
   terminal/   ring buffer por bot, cursores de replay
   courier/    worker de entrega (lease, retry, dead)
   tools/      servidor MCP HTTP (/mcp) com tools dos bots
   hooks/      endpoint /hooks + subcomando `botloftd hook`
   store/      SQLite (WAL), migrations
   platform/   pipes, ACL, Job Objects, tarefa agendada, keep-awake
        │ ConPTY                         ▲ HTTP /mcp e /hooks (token do bot)
        ▼                                │
 claude.exe (um processo por bot) ───────┘
        ▲
        └─ named pipe do inbox (criado pelo Claude Code; botloftd é cliente)
```

### 3.1 Layout do repositório

```
botloft/
  crates/
    botloft-core/    tipos de domínio, IDs, erros, render de envelope, tipos do protocolo
    botloft-store/   SQLite: conexão, migrations, repositórios
    botloftd/        binário do daemon (rpc, supervisor, runtime, courier, tools, hooks, platform)
  app/
    src/             React
    src-tauri/       shell nativa
  docs/
    spec.md
    adr/
  .github/workflows/
```

Tipos do protocolo ficam em `botloft-core` e são exportados para TypeScript com `ts-rs`, gerando `app/src/lib/protocol.gen.ts`. O front nunca redeclara tipos do protocolo à mão.

## 4. Stack

| Camada | Escolha | Motivo |
|---|---|---|
| Daemon | Rust stable, Tokio, Axum | Async maduro, WebSocket e HTTP no mesmo servidor |
| Banco | `rusqlite` (bundled), WAL, FTS5 depois | Arquivo único, sem servidor |
| PTY | `portable-pty` (ConPTY) | Única opção madura em Rust para ConPTY |
| Win32 | crate `windows` | ACL, Job Objects, keep-awake, pipes |
| MCP | servidor Streamable HTTP próprio e mínimo (JSON-RPC) | Poucas tools, sem dependência pesada |
| Tempo | `time` + `croner` (rotinas, pós-MVP) | |
| IDs | ULID com prefixo (`bot_`, `crw_`, `msg_`, `dlv_`, `tsk_`) | Ordenável, legível em log |
| App | Tauri v2, React 19, TypeScript strict, Vite, Tailwind v4 | |
| Estado no app | Zustand | Leve, sem boilerplate |
| Terminal | `@xterm/xterm` + addons fit e webgl | |
| Qualidade | rustfmt, clippy `-D warnings`, Biome, Vitest | Biome faz lint e format do TS numa ferramenta só |

## 5. Estado em disco

| O quê | Onde (padrão) | Override |
|---|---|---|
| Dados do daemon | `%LOCALAPPDATA%\Botloft\` | `BOTLOFT_HOME` |
| Banco | `%LOCALAPPDATA%\Botloft\botloft.db` | |
| Segredos | `%LOCALAPPDATA%\Botloft\secrets\` (ACL só do usuário) | |
| Logs | `%LOCALAPPDATA%\Botloft\logs\` (rotação diária, 14 dias) | |
| Config | `%LOCALAPPDATA%\Botloft\config.toml` | `--config` |
| Workspaces | `%USERPROFILE%\Botloft\<crew>\<bot>\` | `workspaces_root` na config |
| Pasta compartilhada da crew | `%USERPROFILE%\Botloft\<crew>\shared\` | |

`%LOCALAPPDATA%` e não `%APPDATA%`: o perfil roaming sincroniza em rede e não deve carregar SQLite nem segredos.

Workspaces ficam num caminho curto e visível para o usuário abrir no Explorer e para reduzir estouro do limite de 260 caracteres (bots rodam `npm install`).

Nomes de pasta de crew e bot são slugs gerados na criação e **não mudam** quando o nome de exibição muda.

### 5.1 Arquivos gerados em cada workspace

```
<workspace>\
  CLAUDE.md                        memória viva do bot (o bot edita; o daemon só cria se não existir)
  .claude\
    settings.json                  hooks + permissões (gerado pelo daemon, sobrescrito a cada start)
    rules\botloft.md               identidade, papel, crew e guia de uso das tools (gerado pelo daemon)
  .botloft\
    mcp.json                       config MCP do bot (gerado pelo daemon)
```

Identidade e instruções vão em `.claude/rules/botloft.md` e não na linha de comando: o texto pode ser longo e a linha de comando do Windows é limitada. Regras sem frontmatter `paths` são carregadas no início de toda sessão, com a mesma prioridade de `.claude/CLAUDE.md` (confirmado na documentação oficial, ver seção 19). Plano B, se isso mudar: `--append-system-prompt` com texto curto apontando para o arquivo.

## 6. Configuração (`config.toml`)

```toml
port = 45710                  # só 127.0.0.1 no MVP
workspaces_root = ""          # vazio = %USERPROFILE%\Botloft
claude_path = ""              # vazio = resolver pelo PATH
keep_awake = true             # impede suspensão enquanto houver bot busy
log_level = "info"

[supervisor]
restart_backoff_initial_ms = 1000
restart_backoff_max_ms = 300000
fresh_start_if_dies_within_s = 15

[courier]
poll_interval_ms = 500
lease_ms = 15000
max_attempts = 8
retry_backoff_initial_ms = 2000
retry_backoff_max_ms = 120000

[terminal]
ring_buffer_bytes = 1048576

[tasks]
max_hops = 4
default_deadline_minutes = 120
```

## 7. Ciclo de vida do bot

### 7.1 Estados

| Estado | Quando |
|---|---|
| `offline` | Sem processo e sem restart agendado (crew ou bot pausado) |
| `launching` | Processo criado, aguardando hook `SessionStart` |
| `idle` | Sessão pronta, sem turno em andamento |
| `busy` | Turno em andamento |
| `needs_approval` | Claude Code mostrando prompt de permissão |
| `rate_limited` | Turno falhou por limite de uso |
| `auth_error` | Claude Code sem autenticação válida |
| `backoff` | Processo morreu; relançamento agendado |
| `archived` | Bot arquivado; processo parado, token revogado |

### 7.2 Fontes de transição

| Evento | Novo estado |
|---|---|
| spawn do processo | `launching` |
| hook `SessionStart` | `idle` (e registra o inbox, ver 9.2) |
| hook `UserPromptSubmit` | `busy` |
| hook `Stop` | `idle` |
| hook `StopFailure` com `rate_limit` | `rate_limited` |
| hook `StopFailure` com `authentication_failed` | `auth_error` |
| hook `Notification` com `permission_prompt` | `needs_approval` |
| input do usuário depois de `needs_approval` | `busy` |
| saída do processo | `backoff` (ou `offline`/`archived` se foi pedido) |

### 7.3 Regras

- Bots não pausados sempre rodam. O supervisor reconcilia no boot e a cada 5 s.
- Backoff exponencial com jitter entre `restart_backoff_initial_ms` e `restart_backoff_max_ms`; zera após 10 min de sessão estável.
- Relançamento usa `--continue` para retomar a conversa. Se a sessão retomada morrer em menos de `fresh_start_if_dies_within_s`, o próximo start vem sem `--continue`.
- `auth_error` não entra em loop de restart: fica parado até o owner pedir `bots.restart`.
- Cada processo de bot entra num **Job Object** com `KILL_ON_JOB_CLOSE`. Se o daemon morrer, a árvore de processos dos bots morre junto e não sobra `claude.exe` órfão.

### 7.4 Spawn

1. Resolver o binário: `claude_path` ou PATH (`claude.exe` do instalador nativo; `claude.cmd` do npm roda via `cmd.exe /d /s /c`).
2. `probe`: `claude --version` e exigir **>= 2.1.234** (inbox via named pipe no Windows nativo).
3. Gerar os arquivos da seção 5.1.
4. Comando (confirmar flags com `claude --help` na versão alvo):
   `claude [--continue] --settings <workspace>\.claude\settings.json --mcp-config <workspace>\.botloft\mcp.json`
5. Ambiente: `BOTLOFT_BOT_ID`, `BOTLOFT_BOT_TOKEN`, `BOTLOFT_PORT`, `BOTLOFT_BIN` (caminho absoluto do `botloftd.exe`).
6. PTY com cwd no workspace e tamanho vindo do último `terminal.resize` (padrão 120x32).

### 7.5 `settings.json` gerado

```json
{
  "crossSessionInbound": "accept",
  "hooks": {
    "SessionStart":     [{ "hooks": [{ "type": "command", "command": "<BOTLOFT_BIN>", "args": ["hook", "session-start"] }] }],
    "UserPromptSubmit": [{ "hooks": [{ "type": "command", "command": "<BOTLOFT_BIN>", "args": ["hook", "prompt-submit"] }] }],
    "Stop":             [{ "hooks": [{ "type": "command", "command": "<BOTLOFT_BIN>", "args": ["hook", "stop"] }] }],
    "StopFailure":      [{ "hooks": [{ "type": "command", "command": "<BOTLOFT_BIN>", "args": ["hook", "stop-failure"] }] }],
    "Notification":     [{ "hooks": [{ "type": "command", "command": "<BOTLOFT_BIN>", "args": ["hook", "notification"] }] }],
    "SessionEnd":       [{ "hooks": [{ "type": "command", "command": "<BOTLOFT_BIN>", "args": ["hook", "session-end"] }] }]
  },
  "permissions": {
    "deny": ["Read(//c/Users/<usuário>/AppData/Local/Botloft/secrets/**)"]
  }
}
```

- Hooks em **exec form** (`args` presente): o Claude Code executa o binário direto, sem Git Bash nem PowerShell. Some o problema de quoting, de `curl` e de path com barra invertida.
- O `command` do exec form precisa ser um `.exe` de verdade; shims `.cmd`/`.bat` exigem shell. `botloftd.exe` atende.
- `crossSessionInbound: accept` é obrigatório: o daemon não é processo filho da sessão, então sem isso a mensagem pode ficar retida esperando aprovação.
- Caminho absoluto em permission rule usa o prefixo `//` e a forma POSIX que o Claude Code aplica no Windows: `C:\Users\ana\...` vira `//c/Users/ana/...` (letra do drive em minúscula). Uma barra só (`/caminho`) é relativa à origem do settings, não à raiz, e não protegeria nada. O daemon converte `BOTLOFT_HOME` para essa forma ao gerar o arquivo.

### 7.6 Subcomando `botloftd hook <evento>`

- Lê o JSON do stdin, lê `BOTLOFT_BOT_TOKEN` e `BOTLOFT_PORT` do ambiente.
- `session-start` também lê `CLAUDE_CODE_MESSAGING_SOCKET` e `CLAUDE_CODE_MESSAGING_TOKEN`.
- Faz `POST http://127.0.0.1:<port>/hooks/<evento>` com `Authorization: Bearer <token>`, timeout de 3 s.
- **Sempre** sai com código 0 e **nunca** escreve no stdout (no `SessionStart`, stdout vira contexto do Claude). Erros vão para `logs\hook.log`.

## 8. Terminal

- Leitura da PTY numa thread dedicada por bot, enviando blocos por canal para a task async.
- Coalescência: junta leituras por até 8 ms ou 32 KiB antes de emitir.
- Cada bot tem um ring buffer de `ring_buffer_bytes` com cursor `offset` (bytes desde o início da generation).
- `terminal.attach {botId, generation?, offset?}`:
  - mesma generation e offset ainda no buffer: responde `{generation, offset, reset: false}` e envia só o que falta;
  - senão: `{reset: true}` e envia o buffer inteiro, cortado no primeiro `\n` para não começar no meio de uma sequência de escape.
- Dados vão em `terminal.data {botId, generation, offset, data}` com `data` em base64 (bytes crus; o xterm.js recebe `Uint8Array` e resolve UTF-8 quebrado entre blocos).
- Vários clientes podem assistir ao mesmo bot. Qualquer cliente autenticado pode escrever (MVP só tem o owner).

## 9. Mensagens e entrega

### 9.1 Fluxo

1. `messages.send` (owner) ou tool `send_message` (bot) grava `message` + `delivery` (`pending`) numa transação.
2. O **courier** acorda a cada `poll_interval_ms` (e na hora, via notify, quando entra delivery nova).
3. Pega deliveries `pending` com `next_attempt_at <= agora`, marca `sending` com `lease_until`.
4. Se o bot não está em `idle`/`busy`/`needs_approval` ou o inbox não foi registrado: volta para `pending` com `next_attempt_at` em 5 s, **sem** contar tentativa.
5. Renderiza o envelope (9.3) e escreve no pipe (9.2).
6. Sucesso: `sent`. Falha: `attempts += 1`, backoff exponencial; ao chegar em `max_attempts`, `dead`.
7. Lease vencido (daemon caiu no meio): volta a `pending` no boot e a cada ciclo.

`sent` quer dizer que o Claude Code aceitou a conexão. Não prova que o bot fez o trabalho; para isso existem tasks.

### 9.2 Escrita no inbox (Windows)

- Endereço e token chegam pelo hook `session-start` e ficam só em memória, por generation.
- Abrir o pipe como cliente **somente com a mensagem pronta** (o Claude Code derruba conexão sem linha completa em 30 s).
- Se `ERROR_PIPE_BUSY`, esperar até 2 s (`WaitNamedPipeW`) e tentar de novo dentro da mesma tentativa.
- Conteúdo, uma linha JSON por item, terminado em `\n`:
  1. `{"type":"auth","token":"<CLAUDE_CODE_MESSAGING_TOKEN>"}` (**obrigatório** no Windows)
  2. `{"type":"user","message":{"role":"user","content":"<envelope>"}}`
- Fechar a conexão depois do flush.
- A documentação oficial confirma a linha de auth, a obrigatoriedade dela no Windows e o limite de 30 s, mas **não documenta** a linha de mensagem (item 2). Ela precisa ser confirmada com teste real no M3 antes de o courier depender dela.
- Limites documentados do lado do Claude Code: mensagem de até ~1 milhão de caracteres, no máximo 50 mensagens aceitas na fila, rajadas recusadas e repetições idênticas descartadas. O courier deve respeitar isso (uma entrega por vez por bot).

### 9.3 Envelope

```
[botloft] de @revisor · crew exemplo · tarefa tsk_01J9Z... · prazo 14:30
Para responder: send_message(to: "revisor"). Para concluir: complete_task(task_id: "tsk_01J9Z...").

<corpo da mensagem>
```

Mensagem do owner usa `de @owner` e não tem linha de instrução de tarefa, a menos que seja uma task.

### 9.4 Tasks entre bots

- `send_message` com `kind: "task"` cria uma `task` ligada à message.
- Campos: `requester_bot_id`, `assignee_bot_id`, `status` (`open`, `done`, `failed`, `cancelled`, `expired`), `deadline_at`, `hops`, `origin` (id da task que originou a cadeia).
- Uma task criada por um bot enquanto trabalha em outra herda `origin` e `hops + 1`. Acima de `max_hops`, a tool recusa com erro explicativo.
- `complete_task` grava o resultado e envia automaticamente uma message de volta ao solicitante.
- Task vencida vira `expired` e o solicitante recebe aviso.

## 10. Tools MCP dos bots (`POST /mcp`)

Autenticação: `Authorization: Bearer <token do bot>`. `mcp.json` gerado:

```json
{ "mcpServers": { "botloft": { "type": "http", "url": "http://127.0.0.1:45710/mcp",
  "headers": { "Authorization": "Bearer ${BOTLOFT_BOT_TOKEN}" } } } }
```

**Verificar** que a expansão `${VAR}` em headers do `mcp.json` funciona na versão alvo; plano B é escrever o token direto no arquivo (com ACL só do usuário).

| Tool | Entrada | Saída |
|---|---|---|
| `crew_roster` | nenhuma | bots da mesma crew: handle, papel, estado |
| `send_message` | `to` (handle), `body`, `kind?` (`note` padrão, `task`), `deadline_minutes?` | ids da message e da task |
| `complete_task` | `task_id`, `result`, `status?` (`done` padrão, `failed`) | ok |
| `my_tasks` | `role?` (`assigned`, `requested`) | lista de tasks abertas |

Endereçamento só dentro da crew. Bot não enxerga bots de outras crews.

## 11. Protocolo do app (JSON-RPC 2.0 sobre WebSocket)

Endpoint: `ws://127.0.0.1:45710/rpc`. Mensagens seguem JSON-RPC 2.0: requests com `id`, respostas com `result` ou `error`, notificações do servidor sem `id`.

### 11.1 Sessão

- Primeiro request obrigatório: `session.hello {token, client: {name, version}, protocol: 1}` -> `{daemonVersion, protocol}`.
- Qualquer outro método antes do hello: erro `-32001` e a conexão fecha.
- `Origin` aceito: `http://tauri.localhost`, `tauri://localhost` e `http://localhost:1420` (dev).

### 11.2 Métodos (MVP)

| Método | Params | Result |
|---|---|---|
| `system.status` | | versão, uptime, versão do claude, backlog de entrega |
| `crews.list` | | `Crew[]` |
| `crews.create` | `name` | `Crew` |
| `crews.rename` | `crewId, name` | `Crew` |
| `crews.setPaused` | `crewId, paused` | `Crew` |
| `crews.archive` | `crewId` | `Crew` |
| `bots.list` | `crewId?` | `Bot[]` |
| `bots.create` | `crewId, name, role, instructions` | `Bot` |
| `bots.update` | `botId, name?, role?, instructions?` | `Bot` |
| `bots.setPaused` | `botId, paused` | `Bot` |
| `bots.restart` | `botId, fresh?` | `Bot` |
| `bots.archive` | `botId` | `Bot` |
| `terminal.attach` | `botId, generation?, offset?` | `{generation, offset, reset}` |
| `terminal.detach` | `botId` | |
| `terminal.write` | `botId, data` (base64) | |
| `terminal.resize` | `botId, cols, rows` | |
| `messages.send` | `botId, body` | `Message` |
| `messages.list` | `crewId?, botId?, before?, limit?` | `Message[]` |
| `deliveries.list` | `state?, botId?` | `Delivery[]` |
| `deliveries.retry` | `deliveryId` | `Delivery` |
| `tasks.list` | `crewId?, status?` | `Task[]` |

### 11.3 Notificações do servidor

`bot.state`, `bot.changed`, `crew.changed`, `terminal.data`, `message.created`, `delivery.changed`, `task.changed`.

### 11.4 Erros

| Código | Significado |
|---|---|
| `-32001` | não autenticado |
| `-32002` | não encontrado |
| `-32003` | conflito de estado (ex.: bot arquivado) |
| `-32004` | validação |
| `-32005` | runtime indisponível (claude ausente ou versão antiga) |

## 12. Dados (SQLite)

Pragmas: `journal_mode=WAL`, `foreign_keys=ON`, `busy_timeout=5000`. Migrations numeradas em `botloft-store/migrations/NNNN_nome.sql`, versão em `PRAGMA user_version`. Tempo em milissegundos Unix (`INTEGER`). Arquivamento é lógico (`archived_at`).

| Tabela | Colunas principais |
|---|---|
| `crews` | `id, name, slug, paused, created_at, archived_at` |
| `bots` | `id, crew_id, name, handle, slug, role, instructions, paused, token_hash, created_at, archived_at` |
| `messages` | `id, crew_id, from_kind (owner/bot/system), from_bot_id, to_bot_id, kind (note/task/result/system), body, task_id, created_at` |
| `deliveries` | `id, message_id, bot_id, state, attempts, next_attempt_at, lease_until, last_error, updated_at` |
| `tasks` | `id, crew_id, requester_bot_id, assignee_bot_id, status, deadline_at, hops, origin_task_id, result, created_at, updated_at` |
| `settings` | `key, value` |

Índices mínimos: `deliveries(state, next_attempt_at)`, `messages(crew_id, created_at)`, `tasks(assignee_bot_id, status)`, `bots(crew_id)`.

## 13. Segurança e privacidade

- Daemon escuta **só em 127.0.0.1** no MVP.
- Token do owner: 32 bytes aleatórios em `secrets\owner.token`, ACL com acesso só para o SID do usuário atual (DACL protegida, sem herança).
- Token de bot: 32 bytes aleatórios. No banco fica só o **SHA-256** (`token_hash`); o valor cru existe apenas no ambiente do processo do bot. Novo token a cada generation.
- Logs nunca registram tokens, conteúdo de mensagens nem saída de terminal em nível `info`. Bots podem manipular dados sensíveis (inclusive de saúde); o daemon trata corpo de mensagem como dado pessoal.
- Isolamento entre bots é cooperativo (mesmo usuário do Windows). Documentar isso no README sem prometer sandbox.

## 14. Integração com o Windows

| Tema | Solução |
|---|---|
| Iniciar com o Windows | `botloftd service install` cria **Tarefa Agendada por usuário** ("ao fazer logon", reiniciar em falha a cada 1 min, sem limite de execução, só com o usuário logado). Não usar Windows Service: roda em outra sessão e sem acesso à autenticação do Claude Code do usuário. Subcomandos `service status`, `service restart`, `service uninstall`. |
| Binário instalado | `%LOCALAPPDATA%\Botloft\bin\botloftd.exe` (copiado do sidecar do app) |
| Não suspender | `SetThreadExecutionState(ES_CONTINUOUS \| ES_SYSTEM_REQUIRED)` enquanto houver bot `busy`, se `keep_awake = true` |
| Processos órfãos | Job Object por bot (seção 7.3) |
| Encerramento | `ctrl_c`, `ctrl_close`, `ctrl_shutdown`, `ctrl_logoff` do Tokio: parar courier, sinalizar bots, flush do banco |
| Caminhos longos | manifesto `longPathAware` no daemon e no app; usar APIs com `\\?\` ao apagar árvores de workspace |
| Arquivo em uso | arquivar bot só apaga workspace depois do processo encerrado; retentar exclusão se bloqueado |
| Instância única | mutex nomeado `Local\Botloft.Daemon`; se já existir, sair com erro claro |
| Porta ocupada | se 45710 estiver em uso por outro processo, sair com erro (sem porta alternativa no MVP) |

## 15. App desktop

### 15.1 Estrutura

```
app/src/
  main.tsx
  shell/          janela, barra de título, layout, atalhos
  features/
    onboarding/   checagens (claude instalado e versão, daemon rodando) e instalação do serviço
    crews/        lista, criar, renomear, pausar
    bots/         lista, criar, editar, estado
    terminal/     view xterm.js, attach/replay, input, resize
    messages/     timeline da crew, enviar mensagem, deliveries com falha
    tasks/        tarefas abertas por crew
    settings/
  lib/
    rpc.ts        cliente JSON-RPC com reconexão e fila de requests
    api.ts        interface BotloftApi (contrato usado pela UI)
    client.ts     implementação real sobre rpc.ts
    fake.ts       FakeBotloft em memória para testes
    protocol.gen.ts   gerado por ts-rs, não editar
  store/          stores Zustand alimentados por notificações
  ui/             componentes base
```

Componentes dependem só de `BotloftApi`, nunca do cliente concreto.

### 15.2 Comandos Tauri

| Comando | Função |
|---|---|
| `daemon_status` | GET `/health` local |
| `daemon_install` | copia sidecar e roda `botloftd service install` |
| `daemon_restart` | `botloftd service restart` |
| `read_owner_token` | lê `secrets\owner.token` (só no app local) |
| `open_path` | abre pasta no Explorer (`explorer /select,`) |
| `set_unread_badge` | overlay icon na taskbar |

### 15.3 Direção visual

Ferramenta de trabalho densa, estilo painel de operação: tipografia forte, grid firme, estados dos bots legíveis de longe (cor + ícone + texto, nunca só cor). Sem gradiente, sem sombra pesada, sem visual de template. Tema escuro e claro. Barra de título própria (`decorations: false`) com controles de janela do Windows.

Identidade: o mascote do Botloft é uma chama com olhos, desenhada em vetor em `app/app-icon.svg`. O ícone do app é o mascote branco sobre fundo preto. Cada bot usa o mesmo personagem como avatar, com uma cor própria escolhida na criação. A cor do avatar identifica o bot e não comunica estado: estado continua sendo cor + ícone + texto, como descrito acima.

## 16. Qualidade

- Rust: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`.
- TS: `tsc --noEmit`, `biome check`, `vitest run`.
- Limite **flexível de 300 linhas** por arquivo; passou disso, dividir por responsabilidade.
- Toda lógica de supervisor, courier e terminal testada com `FakeRuntime` (sem Claude real).
- CI: GitHub Actions em `windows-latest` (principal) e `ubuntu-latest` para `botloft-core` e `botloft-store`.

## 17. Marcos do MVP

| Marco | Entrega | Pronto quando |
|---|---|---|
| **M0** Fundação | Workspace Cargo, app Tauri vazio, CI, LICENSE, README | CI verde nos dois runners |
| **M1** Daemon base | config, store + migrations, `/health`, `/rpc` com hello, CRUD de crews e bots, geração de workspace | teste de integração cria crew e bot via RPC |
| **M2** Runtime | ConPTY, supervisor com estados e backoff, Job Objects, hooks, terminal com replay, `FakeRuntime` | bot real abre no Windows, estados mudam pelos hooks, reattach sem perder tela |
| **M3** Mensagens | courier, inbox via pipe, tools MCP, tasks com hops e prazo | dois bots trocam task e resultado sem intervenção |
| **M4** App | onboarding, crews, bots, terminal, timeline, deliveries com falha | fluxo completo pelo app sem abrir terminal |
| **M5** Distribuição | tarefa agendada, keep-awake, instalador NSIS com sidecar, updater | instalar em máquina limpa e bots voltarem sozinhos após reboot |

## 18. Fora do MVP (ordem sugerida)

1. Rotinas (cron com timezone, intervalo) com política de sobreposição.
2. Caixa de perguntas ao owner (bot pergunta, owner responde, resposta volta como mensagem).
3. Busca FTS5 em mensagens.
4. Histórico de versões das instruções do bot.
5. Acesso remoto com token por dispositivo (Tailscale).
6. Sinais entre bots disparando rotinas.
7. Suporte Linux/macOS.

## 19. Pontos a verificar na versão alvo do Claude Code

Conferência na documentação oficial (code.claude.com/docs) em 2026-09-28. "Confirmado" quer dizer documentado; o teste real na versão alvo continua no checklist do marco indicado.

| Item | Seção | Resultado | Teste real |
|---|---|---|---|
| Exec form de hooks (`args`) no Windows | 7.5 | Confirmado (`hooks`): com `args`, o binário roda direto, sem shell; `command` precisa ser `.exe` | M2 |
| Linha de auth, 30 s e variáveis do inbox | 9.2 | Confirmado (`cross-session-messaging`): `CLAUDE_CODE_MESSAGING_SOCKET`, `CLAUDE_CODE_MESSAGING_TOKEN`, auth obrigatória no Windows, >= 2.1.234 | M3 |
| Formato da linha de mensagem no inbox | 9.2 | **Não documentado** | M3, obrigatório |
| `crossSessionInbound` | 7.5 | Confirmado: valores `accept`, `hold`, `refuse`; `refuse` em settings de projeto vence tudo | M3 |
| Carregamento de `.claude/rules/*.md` sem frontmatter | 5.1 | Confirmado (`memory`): carregadas sempre | M2 |
| Expansão `${VAR}` em headers do `mcp.json` | 10 | Confirmado (`mcp`); alguns nomes de credencial conhecidos são lidos vazios, `BOTLOFT_BOT_TOKEN` não é um deles | M3 |
| Sintaxe de caminho Windows em permission rules | 7.5 | Confirmado (`permissions`): `//c/...` em forma POSIX; spec corrigida | M2 |
| Flags `--continue`, `--mcp-config` | 7.4 | Confirmado (`cli-reference`) | M2 |
| Flag `--settings <arquivo>` | 7.4 | Citada em outras páginas (ex.: `cross-session-messaging`), ausente da tabela de CLI; confirmar com `claude --help` | M2 |
| stdout do `SessionStart` vira contexto | 7.6 | Confirmado (`hooks`) | M2 |
| `StopFailure` (`rate_limit`, `authentication_failed`) e `Notification` (`permission_prompt`) | 7.2 | Eventos e valores de matcher confirmados; nome do campo no JSON do stdin a confirmar | M2 |
