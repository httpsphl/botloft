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

Nomes de pasta de crew e bot são slugs gerados na criação e **não mudam** quando o nome de exibição muda. Slug: ASCII minúsculo com hífens, acentos transliterados (`Revisão` -> `revisao`), no máximo 32 caracteres, nunca um nome de dispositivo do Windows (`con`, `lpt1`...) e nunca `shared` para bot. Colisão ganha sufixo (`docs-2`), inclusive com slug de item arquivado ou pasta que já exista no disco.

O **handle** do bot (`@revisao`) é derivado do nome pela mesma regra, acompanha renomeações e é único entre os bots ativos da crew; um nome que gere handle já usado é recusado com erro de validação.

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
| hook `StopFailure` com `authentication_failed` ou `oauth_org_not_allowed` | `auth_error` |
| hook `StopFailure` com outro `error` | `idle` (o turno acabou, a sessão segue) |
| hook `Notification` com `permission_prompt` | `needs_approval` |
| input do usuário depois de `needs_approval` | `busy` |
| saída do processo | `backoff` (ou `offline`/`archived` se foi pedido) |

Campos lidos do JSON do stdin (documentados): `error` no `StopFailure`, `notification_type` no `Notification`. Hooks de uma generation antiga são ignorados. Todo processo novo emite `bot.state {botId, state, generation}`, mesmo que o nome do estado não mude, porque a generation mudou.

### 7.3 Regras

- Bots não pausados sempre rodam. O supervisor reconcilia no boot e a cada 5 s.
- Backoff exponencial com jitter entre `restart_backoff_initial_ms` e `restart_backoff_max_ms`; zera após 10 min de sessão estável.
- Relançamento usa `--continue` para retomar a conversa. Se a sessão retomada morrer em menos de `fresh_start_if_dies_within_s`, o próximo start vem sem `--continue`. O primeiro start de um bot não usa `--continue`: o daemon grava `.botloft/started` no workspace no primeiro `SessionStart` e só a partir daí retoma. `bots.restart {fresh: true}` também começa conversa nova.
- Mudanças em nome, papel ou instruções regravam as regras na hora, mas o bot só as lê no próximo start; o daemon não reinicia o bot sozinho.
- `auth_error` não entra em loop de restart: fica parado até o owner pedir `bots.restart`.
- Cada processo de bot entra num **Job Object** com `KILL_ON_JOB_CLOSE`. Se o daemon morrer, a árvore de processos dos bots morre junto e não sobra `claude.exe` órfão.

### 7.4 Spawn

1. Resolver o binário: `claude_path` ou PATH. Só o `claude.exe` nativo roda. O `claude.cmd` do npm é recusado com erro claro: passar por `cmd.exe /d /s /c` estraga o quoting dos argumentos na PTY, e a documentação recomenda o instalador nativo. O PATH é separado só por `;`, como o Windows faz; aspas soltas numa entrada (comum em PATHs reais) não escondem as entradas seguintes.
2. `probe`: `claude --version` e exigir **>= 2.1.234** (inbox via named pipe no Windows nativo). Sem Claude utilizável, os bots ficam `offline`, `system.status.runtimeError` diz o motivo e o daemon tenta de novo a cada 30 s.
3. Gerar os arquivos da seção 5.1.
4. Comando: `claude [--continue] --settings <workspace>\.claude\settings.json --mcp-config <workspace>\.botloft\mcp.json`. O `--settings` aponta para o mesmo arquivo que o Claude Code já lê como settings do projeto; a documentação garante que um hook definido em dois arquivos roda uma vez só, e o `--settings` dá precedência ao `crossSessionInbound` sobre as settings do usuário.
5. Ambiente: o bloco padrão do usuário (`CreateEnvironmentBlock`, o mesmo de um logon novo), **não** o ambiente do daemon. Um daemon iniciado de dentro de uma sessão do Claude Code herda `CLAUDECODE`, `CLAUDE_CODE_MESSAGING_SOCKET`, `ANTHROPIC_BASE_URL` e outras variáveis da sessão, que fariam o bot se achar filho dela. Por cima vão `BOTLOFT_BOT_ID`, `BOTLOFT_BOT_TOKEN`, `BOTLOFT_PORT`, `BOTLOFT_BIN` (caminho absoluto do `botloftd.exe`) e `BOTLOFT_HOME` (onde o hook grava `logs\hook.log`).
6. PTY com cwd no workspace e tamanho vindo do último `terminal.resize` (padrão 120x32).

### 7.4.1 Primeira execução de cada bot

Numa sessão interativa, o Claude Code segura **todos** os hooks até o dono aceitar o diálogo de confiança da pasta (documentado; confirmado com 2.1.283). Na primeira execução o bot fica em `launching`, com o diálogo no terminal, e só vai para `idle` quando alguém escolhe "Yes, I trust this folder" (a opção pré-selecionada é "No, exit"). Depois disso a confiança fica gravada para aquela pasta. Avisos de primeira execução vindos da configuração global do usuário também aparecem no terminal (ex.: extensão do Chrome detectada, novo renderizador).

A confiança de uma pasta pai cobre as subpastas (fora de repositório git). O onboarding do app (M4) deve pedir ao dono para confiar uma vez na raiz dos workspaces, para que bots novos não parem no diálogo. Workspaces dentro de um repositório git herdam a confiança da raiz do repositório.

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

- Cada processo tem três threads: leitura da PTY, escrita (a escrita nunca bloqueia quem chama) e espera pelo fim do processo. A leitura manda blocos por canal para a task async. Quando o processo termina, o daemon fecha o pseudoconsole; sem isso o ConPTY nunca entrega EOF.
- Coalescência: junta leituras por até 8 ms ou 32 KiB antes de emitir.
- **Handshake do ConPTY:** o `portable-pty` cria o pseudoconsole com `PSEUDOCONSOLE_INHERIT_CURSOR`, e com isso o ConPTY pergunta a posição do cursor (`ESC[6n`) ao iniciar e segura o processo filho até ouvir a resposta. O daemon responde `ESC[1;1R` à primeira consulta de cada generation; as seguintes ficam com o terminal do app (confirmado com Claude Code real: sem a resposta, a tela fica vazia para sempre).
- Cada bot tem um ring buffer de `ring_buffer_bytes` com cursor `offset` (bytes desde o início da generation). O buffer fica depois que o processo morre, para o dono ver a última tela.
- Generations são únicas entre bots e entre reinícios do daemon (o contador começa no horário de boot em ms), para um cliente nunca confundir um processo novo com um que já viu.
- `terminal.attach {botId, generation?, offset?}`:
  - mesma generation e offset ainda no buffer: responde `{generation, offset, reset: false}` e envia só o que falta;
  - senão: `{reset: true}` e envia o buffer inteiro; se o buffer já descartou o começo da generation, corta no primeiro `\n` para não começar no meio de uma sequência de escape.
- A resposta do `terminal.attach` sai antes do primeiro `terminal.data`. `terminal.detach {botId}` para o envio. `terminal.write {botId, data}` e `terminal.resize {botId, cols, rows}` (1 a 1000) respondem `null`; escrever num bot sem processo dá `-32003`.
- Dados vão em `terminal.data {botId, generation, offset, data}` com `data` em base64 (bytes crus; o xterm.js recebe `Uint8Array` e resolve UTF-8 quebrado entre blocos). Um `terminal.data` com generation nova é uma tela nova: o cliente limpa antes de escrever. Um cliente que atrasa pode ver o offset pular; ele percebe (offset diferente do esperado) e refaz o attach com o que tem.
- Vários clientes podem assistir ao mesmo bot. Qualquer cliente autenticado pode escrever (MVP só tem o owner).

## 9. Mensagens e entrega

### 9.1 Fluxo

1. `messages.send` (owner) ou tool `send_message` (bot) grava `message` + `delivery` (`pending`) numa transação.
2. O **courier** acorda a cada `poll_interval_ms` (e na hora, via notify, quando entra delivery nova ou termina um envio).
3. Entrega em ordem, uma por vez por bot: de cada bot, só a delivery `pending` mais antiga pode sair, quando `next_attempt_at <= agora` e o bot não tem outra em `sending`. Ela vira `sending` com `lease_until`.
4. Se o bot não está em `idle`/`busy`/`needs_approval` ou o inbox não foi registrado: volta para `pending` com `next_attempt_at` em 5 s, **sem** contar tentativa. Bot ou crew arquivados: a delivery vira `dead` na hora.
5. Renderiza o envelope (9.3) na hora do envio e escreve no pipe (9.2), com timeout de 10 s.
6. Sucesso: `sent`. Falha: `attempts += 1` e nova tentativa em `retry_backoff_initial_ms * 2^(attempts-1)`, limitado a `retry_backoff_max_ms`; ao chegar em `max_attempts`, `dead`. Se o bot reiniciou durante o envio (inbox trocou), a falha não conta tentativa.
7. Lease vencido (daemon caiu no meio): volta a `pending` no boot e a cada ciclo.
8. `deliveries.retry` devolve uma delivery `dead` para `pending`, com as tentativas zeradas.

`sent` quer dizer que o Claude Code aceitou a conexão. Não prova que o bot fez o trabalho; para isso existem tasks.

Os horários de `next_attempt_at`, `lease_until` e prazos vêm de um relógio injetável do daemon, para os testes controlarem o tempo sem esperar.

### 9.2 Escrita no inbox (Windows)

- Endereço e token chegam pelo hook `session-start` e ficam só em memória, por generation. O endereço tem a forma `\\.\pipe\LOCAL\cc-msg-<32 hex>` e o token tem 32 caracteres (visto com 2.1.283).
- Abrir o pipe como cliente **somente com a mensagem pronta** (o Claude Code derruba conexão sem linha completa em 30 s).
- Se `ERROR_PIPE_BUSY`, tentar abrir de novo a cada 50 ms por até 2 s, dentro da mesma tentativa.
- Conteúdo, uma linha JSON por item, terminado em `\n`, em UTF-8:
  1. `{"type":"auth","token":"<CLAUDE_CODE_MESSAGING_TOKEN>"}` (**obrigatório** no Windows)
  2. `{"type":"user","message":{"role":"user","content":"<envelope>"}}`
- Fechar a conexão logo depois de escrever. O Claude Code não responde nada; o que já foi escrito continua legível para ele depois do fechamento.
- A documentação oficial confirma a linha de auth, a obrigatoriedade dela no Windows e o limite de 30 s, mas **não documenta** a linha de mensagem (item 2). Ela foi confirmada com teste real (2.1.283, ver seção 19):
  - a mensagem chega como `Another Claude session sent a message:` seguida do texto e de um aviso de que mensagem de outra sessão não aprova prompts nem muda configuração; conteúdo com várias linhas e acentos chega intacto e abre um turno se o bot estiver parado;
  - sem a linha de auth, ou com token errado, o Claude Code fecha a conexão e não entrega nada;
  - com auth válido, uma linha que não é JSON é ignorada em silêncio e a conexão continua aberta;
  - três conexões seguidas, uma mensagem cada, chegaram todas.
- Mensagens do owner chegam com o mesmo aviso de "outra sessão": o envelope (9.3) diz que vieram do owner, mas elas não valem como aprovação de prompt de permissão.
- Limites documentados do lado do Claude Code: mensagem de até ~1 milhão de caracteres, no máximo 50 mensagens aceitas na fila, rajadas recusadas e repetições idênticas descartadas. O courier respeita isso com uma entrega por vez por bot e corpo de no máximo 100 000 caracteres.

### 9.3 Envelope

O texto que o bot lê é em inglês, como as regras geradas (5.1):

```
[botloft] from @revisor · crew Exemplo · task tsk_01J9Z... · due in 2 h
Reply with send_message(to: "revisor"). When the task is done, call complete_task(task_id: "tsk_01J9Z...").

<corpo da mensagem>
```

- Nota de outro bot: primeira linha `[botloft] from @revisor · crew Exemplo` e só a instrução de resposta.
- Mensagem do owner: `from the owner`, sem `@` (um bot pode se chamar "Owner") e sem linha de instrução.
- Resultado de task: `· result of task tsk_... · done` (ou `failed`) e a instrução de resposta.
- Aviso do daemon (task vencida): `from Botloft` e o texto do aviso, sem instrução.
- O prazo é relativo (`due in 45 min`, `due in 2 h`, `overdue`) e calculado na hora do envio: o bot não sabe a hora atual, e o app mostra o horário absoluto a partir de `deadline_at`.

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
- Qualquer outro método antes do hello: erro `-32001` e a conexão fecha. Token errado também dá `-32001`; `protocol` diferente dá `-32004`; nos dois casos a conexão fecha. O hello tem que chegar em até 10 s.
- `Origin` aceito: `http://tauri.localhost`, `tauri://localhost` e `http://localhost:1420` (dev). Qualquer outro `Origin` recebe HTTP 403 antes do upgrade. Sem `Origin` (cliente nativo, testes) é aceito: navegadores sempre mandam o header, e o token continua obrigatório.
- Um cliente lento que deixa acumular mais de 1024 notificações é desconectado e recarrega o estado ao reconectar.

### 11.2 Métodos (MVP)

| Método | Params | Result |
|---|---|---|
| `system.status` | | versão, uptime, versão do claude, `runtimeError` (por que os bots não sobem), backlog de entrega (M3) |
| `crews.list` | | `Crew[]` |
| `crews.create` | `name` | `Crew` |
| `crews.rename` | `crewId, name` | `Crew` |
| `crews.setPaused` | `crewId, paused` | `Crew` |
| `crews.archive` | `crewId` | `Crew` |
| `bots.list` | `crewId?` | `Bot[]` |
| `bots.create` | `crewId, name, role, instructions, color?` | `Bot` |
| `bots.update` | `botId, name?, role?, instructions?, color?` | `Bot` |
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

Além desses, os códigos padrão do JSON-RPC: `-32700` (JSON inválido), `-32600` (request inválido), `-32601` (método desconhecido), `-32602` (params inválidos) e `-32603` (erro interno; a mensagem não traz corpo de mensagem nem token). `crews.archive` e `bots.archive` são idempotentes.

## 12. Dados (SQLite)

Pragmas: `journal_mode=WAL`, `foreign_keys=ON`, `busy_timeout=5000`. Migrations numeradas em `botloft-store/migrations/NNNN_nome.sql`, versão em `PRAGMA user_version`. Tempo em milissegundos Unix (`INTEGER`). Arquivamento é lógico (`archived_at`).

| Tabela | Colunas principais |
|---|---|
| `crews` | `id, name, slug, paused, created_at, archived_at` |
| `bots` | `id, crew_id, name, handle, slug, role, instructions, color, paused, token_hash, created_at, archived_at` |
| `messages` | `id, crew_id, from_kind (owner/bot/system), from_bot_id, to_bot_id, kind (note/task/result/system), body, task_id, created_at` |
| `deliveries` | `id, message_id, bot_id, state, attempts, next_attempt_at, lease_until, last_error, updated_at` |
| `tasks` | `id, crew_id, requester_bot_id, assignee_bot_id, status, deadline_at, hops, origin_task_id, result, created_at, updated_at` |
| `settings` | `key, value` |

Índices mínimos: `deliveries(state, next_attempt_at)`, `messages(crew_id, created_at)`, `tasks(assignee_bot_id, status)`, `bots(crew_id)`.

## 13. Segurança e privacidade

- Daemon escuta **só em 127.0.0.1** no MVP.
- Token do owner: 32 bytes aleatórios em `secrets\owner.token`, ACL com acesso só para o SID do usuário atual (DACL protegida, sem herança).
- Token de bot: 32 bytes aleatórios, novo a cada generation. O valor cru existe apenas no ambiente do processo do bot; o daemon guarda só o **SHA-256**, e só em memória: todo processo de bot morre junto com o daemon (Job Object), então nenhum token sobrevive a um reinício e não há motivo para gravá-lo. A coluna `bots.token_hash` fica sem uso.
- Logs nunca registram tokens, conteúdo de mensagens nem saída de terminal em nível `info`. Bots podem manipular dados sensíveis (inclusive de saúde); o daemon trata corpo de mensagem como dado pessoal.
- `log_level` vale só para os crates do Botloft; dependências ficam em `warn`, porque em `debug`/`trace` a pilha de WebSocket registra frames, que podem conter mensagens. `RUST_LOG` sobrepõe tudo e é só para depuração local.
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
| Instância única | lock exclusivo em `<BOTLOFT_HOME>\botloftd.lock` (`File::try_lock`), solto pelo sistema quando o processo termina, mesmo em crash; se já estiver preso, sair com erro claro. É um lock por pasta de dados, e não um mutex de nome fixo, para o daemon de dev (`BOTLOFT_HOME`) rodar ao lado do instalado |
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
| Exec form de hooks (`args`) no Windows | 7.5 | Confirmado (`hooks`) e **testado com 2.1.283**: `botloftd.exe hook session-start` rodou direto, sem shell | feito (M2) |
| Linha de auth, 30 s e variáveis do inbox | 9.2 | Confirmado (`cross-session-messaging`): `CLAUDE_CODE_MESSAGING_SOCKET`, `CLAUDE_CODE_MESSAGING_TOKEN`, auth obrigatória no Windows, >= 2.1.234. **Testado com 2.1.283**: as duas variáveis chegam ao hook `SessionStart`; sem auth ou com token errado a conexão é fechada | feito (M3) |
| Formato da linha de mensagem no inbox | 9.2 | **Não documentado**; **testado com 2.1.283**: `{"type":"user","message":{"role":"user","content":...}}` é entregue e abre um turno (detalhes em 9.2) | feito (M3) |
| `crossSessionInbound` | 7.5 | Confirmado: valores `accept`, `hold`, `refuse`; `refuse` em settings de projeto vence tudo. **Testado com 2.1.283**: com `accept` a mensagem entrou direto, sem diálogo | feito (M3) |
| Carregamento de `.claude/rules/*.md` sem frontmatter | 5.1 | Confirmado (`memory`) e **testado com 2.1.283**: o bot respondeu nome, handle e crew tirados das regras | feito (M2) |
| Expansão `${VAR}` em headers do `mcp.json` | 10 | Confirmado (`mcp`); alguns nomes de credencial conhecidos são lidos vazios, `BOTLOFT_BOT_TOKEN` não é um deles | M3 |
| Sintaxe de caminho Windows em permission rules | 7.5 | Confirmado (`permissions`) e **testado com 2.1.283**: ler `secrets\owner.token` deu "File is in a directory that is denied by your permission settings" | feito (M2) |
| Flags `--continue`, `--mcp-config` | 7.4 | Confirmado (`cli-reference`); aceitas pelo 2.1.283. **Testado**: depois de reiniciar o daemon, o bot voltou com `--continue` e a conversa anterior na tela | feito (M3) |
| Flag `--settings <arquivo>` | 7.4 | Ausente da tabela de CLI, mas aceita pelo 2.1.283 e os hooks do arquivo rodaram | feito (M2) |
| stdout do `SessionStart` vira contexto | 7.6 | Confirmado (`hooks`); o subcomando nunca escreve no stdout | feito (M2) |
| `StopFailure` (`rate_limit`, `authentication_failed`) e `Notification` (`permission_prompt`) | 7.2 | Campos documentados: `error` e `notification_type`. `SessionStart`, `UserPromptSubmit` e `Stop` **testados com 2.1.283** (`launching` -> `idle` -> `busy` -> `idle`) | disparar `StopFailure` e `permission_prompt` reais: pendente |
| Confiança da pasta segura os hooks | 7.4.1 | Confirmado (`hooks`, `permissions`) e **visto com 2.1.283**: diálogo na primeira execução, bot em `launching` até aceitar | feito (M2) |
