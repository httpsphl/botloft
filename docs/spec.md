# Botloft: especificação v0.2

**TL;DR:** Botloft mantém "tripulações" de bots Claude Code sempre ligados no Windows. Um daemon Rust (`botloftd`) roda cada bot como Claude Code headless (`stream-json` pelo stdin e pelo stdout), guarda tudo em SQLite e entrega mensagens entre bots de forma durável. Um app Tauri + React mostra cada bot como um chat: o dono conversa, manda imagens e arquivos e aprova o que o bot pede, tudo via JSON-RPC sobre WebSocket. O MVP cobre crews, bots, chat e mensagens duráveis; rotinas, caixa de perguntas e acesso remoto vêm depois.

Status: rascunho para implementação. Este documento é a fonte de verdade do projeto. Quando código e spec divergirem, corrige-se um dos dois no mesmo PR. Decisões de arquitetura ficam em `docs/adr/` (a ADR 0001 trocou o terminal pelo chat).

---

## 1. Objetivo e princípios

Botloft é um workspace desktop, Windows-first e open-source, para rodar vários agentes Claude Code persistentes que colaboram entre si.

Princípios:

1. **Daemon independente do app.** Fechar o app não derruba bot nenhum. O daemon é a única fonte de verdade.
2. **Claude Code é o runtime.** Cada bot é uma sessão do Claude Code em modo headless (`claude -p` com `stream-json` na entrada e na saída). Nada de SDK próprio de agente nem de loop reimplementado: o daemon só conversa com o processo pelo protocolo de linhas JSON.
3. **Uma conversa por bot.** Tudo que o bot recebe (dono, outros bots, avisos do daemon) entra pelo stdin como mensagem, em ordem. Tudo que ele faz sai pelo stdout como eventos, que o daemon grava e o app mostra como chat.
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
| **Message** | Texto enviado a um bot, pelo dono, por outro bot ou pelo daemon. Pode trazer anexos (só do dono). |
| **Delivery** | Entrega de uma message ao processo do bot. Tem estado, tentativas e prazo. |
| **Task** | Message que pede trabalho e espera um resultado de volta. |
| **Turn** | Um processamento do Claude Code: começa numa message e termina no evento `result`. |
| **Chat** | Histórico estruturado de um bot: messages recebidas, respostas, ferramentas usadas, aprovações e avisos. |
| **Approval** | Pedido de permissão de uma ferramenta, esperando o dono no chat. |
| **Generation** | Contador que muda a cada vez que o processo de um bot é (re)iniciado. |
| **Owner** | O usuário humano dono da instalação. |

## 3. Arquitetura

```
 App (Tauri + React)
        │  JSON-RPC 2.0 sobre WebSocket  (127.0.0.1:45710/rpc)
        ▼
 botloftd ─────────────────────────────────────────────────────
   rpc/        sessões WS, autenticação, métodos e notificações
   supervisor/ ciclo de vida dos bots, estados, backoff
   runtime/    processo claude com pipes (real) e FakeRuntime (testes)
   chat/       leitura do stream-json, itens do chat, texto ao vivo
   courier/    worker de entrega (escreve no stdin, retry, dead)
   tools/      servidor MCP HTTP (/mcp): tools da crew e aprovações
   store/      SQLite (WAL), migrations
   platform/   ACL, Job Objects, ambiente do usuário, tarefa agendada, keep-awake
        │ stdin (mensagens)   ▲ stdout (eventos)   ▲ HTTP /mcp (token do bot)
        ▼                     │                    │
 claude.exe -p (um processo por bot) ──────────────┘
```

### 3.1 Layout do repositório

```
botloft/
  crates/
    botloft-core/    tipos de domínio, IDs, erros, render de envelope, tipos do protocolo
    botloft-store/   SQLite: conexão, migrations, repositórios
    botloftd/        binário do daemon (rpc, supervisor, runtime, chat, courier, tools, platform)
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
| Processo do bot | `tokio::process` com pipes | stdin/stdout de linhas JSON, sem PTY |
| Win32 | crate `windows` | ACL, Job Objects, ambiente do usuário, keep-awake |
| MCP | servidor Streamable HTTP próprio e mínimo (JSON-RPC) | Poucas tools, sem dependência pesada |
| Tempo | `time` + `croner` (rotinas, pós-MVP) | |
| IDs | ULID com prefixo (`bot_`, `crw_`, `msg_`, `dlv_`, `tsk_`, `cht_`, `apr_`, `att_`) | Ordenável, legível em log |
| App | Tauri v2, React 19, TypeScript strict, Vite, Tailwind v4 | |
| Estado no app | Zustand | Leve, sem boilerplate |
| Markdown no chat | `react-markdown` + `remark-gfm`, sem HTML cru | Resposta do bot é markdown; nada de `dangerouslySetInnerHTML` |
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
  attachments\<aaaa-mm-dd>\        arquivos que o dono mandou no chat (9.5)
  .claude\
    settings.json                  regra de negação para os segredos (gerado pelo daemon, sobrescrito a cada start)
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

[bots]
approval_timeout_minutes = 60 # sem resposta do dono, a ferramenta é negada
attachment_max_mb = 20        # por arquivo; no máximo 10 arquivos por message

[tasks]
max_hops = 4
default_deadline_minutes = 120
```

## 7. Ciclo de vida do bot

### 7.1 Estados

| Estado | Quando |
|---|---|
| `offline` | Sem processo e sem restart agendado (crew ou bot pausado) |
| `launching` | Processo criado, nos primeiros 1,5 s |
| `idle` | Processo vivo, sem turno em andamento |
| `busy` | Turno em andamento ou na fila do Claude Code |
| `needs_approval` | Uma ferramenta espera o dono aprovar (10.1) |
| `rate_limited` | O limite de uso da conta foi atingido; espera `resetsAt` |
| `auth_error` | Claude Code sem autenticação válida |
| `backoff` | Processo morreu; relançamento agendado |
| `archived` | Bot arquivado; processo parado, token revogado |

### 7.2 Fontes de transição

Os estados saem do próprio fluxo de eventos (8.1); não há hooks.

| Evento | Novo estado |
|---|---|
| spawn do processo | `launching` |
| processo vivo há 1,5 s | `idle` (o Claude Code fica calado até a primeira mensagem) |
| delivery escrita no stdin | `busy` (conta um turno pendente) |
| `result` | `idle` se não sobrou turno pendente, senão continua `busy` |
| a tool de aprovação é chamada | `needs_approval` |
| aprovação respondida ou vencida | `busy` |
| erro `rate_limit` num turno, ou `rate_limit_event` com status diferente de `allowed` | `rate_limited` até `resetsAt` (5 min se não vier), depois `idle` |
| erro `authentication_failed`, `oauth_org_not_allowed`, `billing_error` ou `account_on_hold` | `auth_error`; o processo é parado |
| saída do processo | `backoff` (ou `offline`/`archived` se foi pedido) |

Eventos de uma generation antiga são ignorados. Todo processo novo emite `bot.state {botId, state, generation}`, mesmo que o nome do estado não mude, porque a generation mudou.

### 7.3 Regras

- Bots não pausados sempre rodam. O supervisor reconcilia no boot e a cada 5 s.
- Backoff exponencial com jitter entre `restart_backoff_initial_ms` e `restart_backoff_max_ms`; zera após 10 min de sessão estável.
- **Sessão:** o daemon guarda em `bots.session_id` o id da conversa. O primeiro start usa `--session-id <uuid novo>`; os seguintes, `--resume <session_id>`. Se a sessão retomada morrer em menos de `fresh_start_if_dies_within_s`, o próximo start vem com um id novo. `bots.restart {fresh: true}` também começa conversa nova. O histórico do chat fica no banco do daemon (seção 8) e não depende do transcript do Claude Code.
- Mudanças em nome, papel ou instruções regravam as regras na hora, mas o bot só as lê no próximo start; o daemon não reinicia o bot sozinho.
- `auth_error` não entra em loop de restart: fica parado até o owner pedir `bots.restart`. O login é feito fora do bot (`claude auth login` num terminal), e o app diz isso.
- Cada processo de bot entra num **Job Object** com `KILL_ON_JOB_CLOSE`. Se o daemon morrer, a árvore de processos dos bots morre junto e não sobra `claude.exe` órfão.

### 7.4 Spawn

1. Resolver o binário: `claude_path` ou PATH. Só o `claude.exe` nativo roda; o `claude.cmd` do npm é recusado com erro claro (passar por `cmd.exe` estraga o quoting dos argumentos). O PATH é separado só por `;`, como o Windows faz; aspas soltas numa entrada (comum em PATHs reais) não escondem as entradas seguintes.
2. `probe`: `claude --version` e exigir **>= 2.1.234**. Sem Claude utilizável, os bots ficam `offline`, `system.status.runtimeError` diz o motivo e o daemon tenta de novo a cada 30 s.
3. Gerar os arquivos da seção 5.1.
4. Comando, com cwd no workspace:

   ```
   claude -p --input-format stream-json --output-format stream-json --verbose
     --include-partial-messages --replay-user-messages
     (--session-id <uuid> | --resume <session_id>)
     --setting-sources project,local
     --mcp-config <workspace>\.botloft\mcp.json --strict-mcp-config
     --permission-mode default
     --permission-prompt-tool mcp__botloft__permission_prompt
     --allowedTools mcp__botloft
   ```

   - `--setting-sources project,local` e `--strict-mcp-config` deixam de fora hooks, skills, agents, modo de permissão e servidores MCP pessoais do dono: o bot vê o que o Botloft gera. O login da conta não é uma fonte de settings e continua valendo.
   - `--allowedTools mcp__botloft` libera as tools da crew sem aprovação. Uma regra `allow` no `settings.json` do projeto não bastaria: em `-p`, numa pasta que nunca passou pelo diálogo de confiança, o Claude Code não aplica as regras `allow` do projeto (documentado em `permissions`).
   - Qualquer outra ferramenta que peça permissão passa pela tool de aprovação (10.1) e vira um pedido no chat.
5. Ambiente: o bloco padrão do usuário (`CreateEnvironmentBlock`, o mesmo de um logon novo), **não** o ambiente do daemon. Um daemon iniciado de dentro de uma sessão do Claude Code herda `CLAUDECODE`, `CLAUDE_CODE_MESSAGING_SOCKET`, `ANTHROPIC_BASE_URL` e outras variáveis da sessão, que fariam o bot se achar filho dela. Por cima vão `BOTLOFT_BOT_ID`, `BOTLOFT_BOT_TOKEN` e `BOTLOFT_PORT`.
6. stdin, stdout e stderr em pipes, sem console (`CREATE_NO_WINDOW`). O stdin fica aberto enquanto o processo vive; fechá-lo encerra o Claude Code com código 0. O stderr vai para o log em nível `debug`, sem conteúdo de mensagem.

### 7.5 `settings.json` gerado

```json
{
  "permissions": {
    "deny": ["Read(//c/Users/<usuário>/AppData/Local/Botloft/secrets/**)"]
  }
}
```

- Regras `deny` valem mesmo sem confiança na pasta: elas só restringem.
- Caminho absoluto em permission rule usa o prefixo `//` e a forma POSIX que o Claude Code aplica no Windows: `C:\Users\ana\...` vira `//c/Users/ana/...` (letra do drive em minúscula). Uma barra só (`/caminho`) é relativa à origem do settings, não à raiz, e não protegeria nada. O daemon converte `BOTLOFT_HOME` para essa forma ao gerar o arquivo.
- Em `-p` o diálogo de confiança da pasta nunca aparece (documentado), então bot novo começa a trabalhar sem passo manual.

## 8. Chat

O chat de um bot é a sequência de itens que o daemon monta a partir do que entra no stdin e do que sai no stdout. Ele fica no banco (`chat_items`), é paginado pelo app e é a única visão da conversa: não há terminal.

### 8.1 Leitura do stdout

Uma linha JSON por evento, em UTF-8. Linha que não é JSON, ou maior que 8 MiB, é descartada com um aviso em `debug`. Tipos desconhecidos são ignorados (o protocolo cresce entre versões do Claude Code). O que o daemon usa:

| Evento | O que vira |
|---|---|
| `system/init` | guarda `session_id` em `bots.session_id` (vem no começo de cada turno) |
| `user` com `isReplay: true` | a message com aquele `uuid` começou a ser processada: a delivery ganha `read_at` (9.1) |
| `stream_event` com `text_delta` | texto ao vivo da resposta (`chat.delta`, 8.3); não é gravado |
| `assistant`, bloco `text` | item `reply` com o texto (markdown) |
| `assistant`, bloco `tool_use` | item `tool` em `running`, com resumo da entrada |
| `user`, bloco `tool_result` | atualiza o item `tool` do mesmo `tool_use_id`: `done` ou `failed` e um trecho da saída |
| `assistant` com `error` | item `notice` e, conforme o erro, estado `rate_limited` ou `auth_error` (7.2) |
| `rate_limit_event` | uso da conta (janelas de 5 h e 7 dias) em `system.status.usage`; status diferente de `allowed` leva a `rate_limited` |
| `result` | item `turn` com duração, custo e erro (se houve); fecha um turno pendente |

Blocos `thinking` não são gravados nem mostrados.

### 8.2 Itens

Todo item tem `id` (`cht_`), `botId`, `kind`, `createdAt` e `updatedAt`.

| `kind` | Campos | Origem |
|---|---|---|
| `inbound` | `message` (a `Message`, com anexos) | criado junto com a message para o bot: dono, outro bot ou daemon |
| `reply` | `text` | texto do bot |
| `tool` | `toolUseId`, `name`, `summary`, `input`, `status` (`running`, `done`, `failed`), `output` | ferramenta usada pelo bot |
| `approval` | `approvalId`, `toolName`, `summary`, `input`, `status` (`pending`, `allowed`, `denied`, `expired`), `note` | pedido de permissão (10.1) |
| `turn` | `durationMs`, `costUsd`, `error` | fim de um turno |
| `notice` | `level` (`info`, `warning`, `error`), `text` | avisos do daemon: limite de uso, login, sessão reiniciada |

- `summary` é uma frase curta feita pelo daemon a partir da entrada: o comando do `Bash`, o arquivo do `Read`/`Edit`/`Write`, o padrão do `Grep`/`Glob`, a URL do `WebFetch`, a busca do `WebSearch`, o destinatário do `send_message`. Ferramenta desconhecida mostra só o nome.
- `input` guarda o JSON da entrada até 4 KB; `output`, até 8 KB de texto. O resto fica só no transcript do próprio Claude Code.
- A resposta do bot (`reply`) é guardada inteira.
- Uma message de um bot para outro aparece duas vezes: no chat de quem mandou, como o item `tool` do `send_message`; no chat de quem recebe, como `inbound`.

### 8.3 Ao vivo

- `chat.item {item}`: item novo ou atualizado (tool que terminou, aprovação respondida).
- `chat.delta {botId, text}`: pedaço do texto que o bot está escrevendo, na ordem. O app junta os pedaços num balão provisório, trocado pelo `reply` quando ele chega. Um app que conecta no meio de um turno não vê o texto parcial já passado, só o que vier depois e o `reply` final.

### 8.4 Privacidade

Itens do chat são dado pessoal como o corpo das messages: nunca vão para o log em nível `info` ou acima. O log de `debug` registra só tipos de evento e ids.

## 9. Mensagens e entrega

### 9.1 Fluxo

1. `messages.send` (owner) ou tool `send_message` (bot) grava `message`, `delivery` (`pending`) e o item `inbound` no chat do destinatário, numa transação. Anexos são gravados antes (9.5).
2. O **courier** acorda a cada `poll_interval_ms` (e na hora, via notify, quando entra delivery nova ou termina um envio).
3. Entrega em ordem, uma por vez por bot: de cada bot, só a delivery `pending` mais antiga pode sair, quando `next_attempt_at <= agora` e o bot não tem outra em `sending`. Ela vira `sending` com `lease_until`.
4. Se o bot não está em `idle`/`busy`/`needs_approval`: volta para `pending` com `next_attempt_at` em 5 s, **sem** contar tentativa. Bot ou crew arquivados: a delivery vira `dead` na hora.
5. Monta a mensagem (9.2) na hora do envio e escreve uma linha no stdin do processo, com timeout de 10 s.
6. Escrita aceita: `sent`, com a generation do processo. Falha de escrita (processo saindo): volta para `pending` sem contar tentativa.
7. Quando o Claude Code começa o turno daquela mensagem, ele a devolve no stdout com o mesmo `uuid` (`--replay-user-messages`), e a delivery ganha `read_at`. O app mostra isso como "lida".
8. **O processo morreu antes de ler:** a delivery ainda está `sent` sem `read_at` e com a generation que acabou. Ela volta para `pending` com `attempts += 1` e o backoff de `retry_backoff_initial_ms * 2^(attempts-1)`, limitado a `retry_backoff_max_ms`. Ao chegar em `max_attempts` vira `dead`: uma mensagem que derruba o processo toda vez não fica em laço.
9. Lease vencido (daemon caiu no meio): volta a `pending` no boot e a cada ciclo.
10. `deliveries.retry` devolve uma delivery `dead` para `pending`, com as tentativas zeradas.

`sent` quer dizer que o processo do bot recebeu a mensagem na fila dele; `read_at`, que o bot começou a trabalhar nela. Nenhum dos dois prova que o trabalho foi feito; para isso existem tasks.

Os horários de `next_attempt_at`, `lease_until` e prazos vêm de um relógio injetável do daemon, para os testes controlarem o tempo sem esperar.

### 9.2 Entrada pelo stdin

Uma linha JSON por mensagem, terminada em `\n`, em UTF-8:

```json
{"type":"user","uuid":"<uuid da delivery>","message":{"role":"user","content":[<blocos>]}}
```

- `uuid` é o ULID da delivery no formato de UUID. O Claude Code o devolve no replay (visto com 2.1.284).
- Blocos: um `text` com o texto (9.3) e, para imagens anexadas, blocos `image` com `source: {type: "base64", media_type, data}`.
- Mensagem escrita durante um turno entra na fila do Claude Code e vira o turno seguinte (visto com 2.1.284). O courier não precisa esperar o bot ficar parado.
- O formato de entrada do `stream-json` **não é documentado**; foi verificado com teste real (seção 19).

### 9.3 Texto que o bot recebe

A mensagem do **dono** vai como ele escreveu, sem envelope: é o usuário da sessão falando, com a autoridade de quem digita.

Mensagens de **outros bots** e **avisos do daemon** levam um envelope em inglês, como as regras geradas (5.1):

```
[botloft] from @revisor · crew Exemplo · task tsk_01J9Z... · due in 2 h
Reply with send_message(to: "revisor"). When the task is done, call complete_task(task_id: "tsk_01J9Z...").

<corpo da mensagem>
```

- Nota de outro bot: primeira linha `[botloft] from @revisor · crew Exemplo` e só a instrução de resposta.
- Resultado de task: `· result of task tsk_... · done` (ou `failed`) e a instrução de resposta.
- Aviso do daemon (task vencida): `from Botloft` e o texto do aviso, sem instrução.
- O prazo é relativo (`due in 45 min`, `due in 2 h`, `overdue`) e calculado na hora do envio: o bot não sabe a hora atual, e o app mostra o horário absoluto a partir de `deadline_at`.

### 9.4 Tasks entre bots

- `send_message` com `kind: "task"` cria uma `task` ligada à message, na mesma transação.
- Campos: `requester_bot_id`, `assignee_bot_id`, `status` (`open`, `done`, `failed`, `cancelled`, `expired`), `deadline_at`, `hops`, `origin` (id da task que originou a cadeia).
- Prazo: `deadline_minutes` de 1 a 10 080 (uma semana); sem ele, `default_deadline_minutes`.
- Cadeia: a task que o bot está fazendo é a task `open` atribuída a ele com mais `hops` (a mais nova, no empate). Uma task criada por ele herda `origin` dessa task (ou o id dela, se ela for a primeira) e `hops + 1`. Uma task sem cadeia tem `hops = 1`. Acima de `max_hops`, a tool recusa com um erro que diz o passo, o limite e onde a cadeia começou. Notas não contam hops.
- `complete_task` só vale para quem recebeu a task, com status `open` ou `expired`. Grava o resultado e, na mesma transação, cria a message `result` para o solicitante.
- Task vencida vira `expired` no ciclo do courier, e o solicitante recebe um aviso do daemon (`from Botloft`). Ela continua aceitando resultado atrasado.
- `cancelled` fica reservado: nenhuma tool cancela task no MVP.

### 9.5 Anexos

- Só o dono manda anexos, pelo `messages.send` (até 10 arquivos, cada um até `attachment_max_mb`). O app manda os bytes em base64; nomes são reduzidos ao nome do arquivo, sem pasta.
- O daemon grava cada arquivo em `<workspace>\attachments\<aaaa-mm-dd>\<nome>` (nome repetido ganha sufixo `-2`) e registra na tabela `attachments`.
- Imagens PNG, JPEG, GIF e WebP de até 5 MB também seguem inline como blocos `image`, para o bot vê-las sem abrir arquivo.
- O texto da mensagem ganha, no fim, a lista do que foi salvo (`Attached files, saved in your folder: attachments/2026-09-28/relatorio.pdf`), para o bot abrir o resto com as ferramentas dele (PDF, planilha, código...).
- Anexos ficam na pasta do bot até alguém apagar; o chat mostra nome, tipo e tamanho, e o app abre a pasta.

## 10. Tools MCP dos bots (`POST /mcp`)

Autenticação: `Authorization: Bearer <token do bot>`, só de uma generation viva; outro token dá HTTP 401. `mcp.json` gerado:

```json
{ "mcpServers": { "botloft": { "type": "http", "url": "http://127.0.0.1:45710/mcp",
  "headers": { "Authorization": "Bearer ${BOTLOFT_BOT_TOKEN}" },
  "timeout": <(approval_timeout_minutes + 2) em ms> } } }
```

- A expansão `${BOTLOFT_BOT_TOKEN}` foi confirmada com o Claude Code real (seção 19), então o token não vai para o disco.
- `timeout` existe por causa da aprovação (10.1). Um servidor MCP por HTTP tem, por padrão, 60 s por request e 5 min sem resposta antes de o Claude Code abortar a chamada. O `timeout` por servidor (>= 1000) sobe os dois limites (documentado em `env-vars`, `MCP_TOOL_TIMEOUT` e `CLAUDE_CODE_MCP_TOOL_IDLE_TIMEOUT`).

Transporte: Streamable HTTP, só POST e resposta `application/json`, sem sessão e sem stream. O servidor fala duas eras do MCP, porque o Claude Code 2.1.284 tenta a nova e cai para a antiga:

- **2026-07-28** (sem handshake): cada request traz `_meta` com `io.modelcontextprotocol/protocolVersion` e `clientCapabilities`, e os headers `MCP-Protocol-Version`, `Mcp-Method` e, em `tools/call`, `Mcp-Name` (com a forma `=?base64?...?=`). Header que não bate com o corpo dá 400 com `-32020`; `_meta` incompleto dá 400 com `-32602`; método desconhecido dá 404 com `-32601`. Resultados levam `resultType: "complete"` e `_meta.serverInfo`. `server/discover` responde versões, `capabilities: {tools: {}}` e instruções.
- **2025-11-25, 2025-06-18 e 2025-03-26**: `initialize` (sem header de versão) negocia a versão pedida, ou 2025-11-25 se não conhecer. Notificações recebem 202.
- Qualquer outra versão no header dá 400 com `-32022` e a lista de versões aceitas. `GET` e `DELETE` dão 405. Request com `Origin` dá 403 (bots não mandam `Origin`; navegador sempre manda).

| Tool | Entrada | Saída (JSON em texto) |
|---|---|---|
| `crew_roster` | nenhuma | `crew`, `you` e os outros bots da crew: handle, nome, papel, estado |
| `send_message` | `to` (handle, com ou sem `@`), `body`, `kind?` (`note` padrão, `task`), `deadline_minutes?` (só task) | `message_id`, `task_id` e `due` (task), e um lembrete de que a resposta chega depois |
| `complete_task` | `task_id`, `result`, `status?` (`done` padrão, `failed`) | `task_id`, `status` e quem recebe o resultado |
| `my_tasks` | `role?` (`assigned`, `requested`) | tasks `open` e `expired`: id, de, para, status, prazo relativo, hops e o pedido original |
| `permission_prompt` | `tool_name`, `input`, `tool_use_id` | decisão do dono (10.1). Chamada pelo Claude Code, não pelo modelo |

- Erro que o modelo pode corrigir (handle desconhecido, argumento inválido, limite de hops, task de outro bot) volta como resultado com `isError: true` e uma frase explicando. Só tool desconhecida ou chamada malformada vira erro JSON-RPC (`-32602`).
- Erro interno não expõe detalhes ao bot; vai para o log.

Endereçamento só dentro da crew. Bot não enxerga bots nem tasks de outras crews.

### 10.1 Aprovações

Com `--permission-prompt-tool mcp__botloft__permission_prompt`, toda ferramenta que precisaria de permissão chama a tool `permission_prompt`. A entrada é `{tool_name, input, tool_use_id}` (documentado; visto com 2.1.284).

1. O daemon grava uma `approval` (`pending`) e o item `approval` no chat, e passa o bot para `needs_approval`.
2. A chamada HTTP fica aberta até o dono responder (`approvals.answer {approvalId, allow, note?}`) ou até `approval_timeout_minutes`.
3. Resposta ao Claude Code, em texto JSON:
   - Permitir: `{"behavior":"allow","updatedInput":<input original>}`.
   - Negar: `{"behavior":"deny","message":"The owner denied this."}`, com a nota do dono se houver.
   - Sem resposta no prazo: nega com `"The owner did not answer in time."` e a aprovação vira `expired`.
4. Se o processo do bot morrer ou o daemon reiniciar com a aprovação aberta, ela vira `expired`.
5. O bot volta a `busy` e o item do chat é atualizado.

"Permitir sempre" (gravar uma regra para o bot) fica para depois do MVP.

A aprovação só abre se `tool_use_id` for de uma ferramenta em `running` no chat daquele bot. O daemon espera até 2 s pelo evento, que às vezes chega depois da chamada. Senão nega na hora, sem incomodar o dono. Assim um bot que chame `permission_prompt` por conta própria não consegue pôr um pedido inventado na frente do dono. A descrição da tool também diz para não chamá-la.

## 11. Protocolo do app (JSON-RPC 2.0 sobre WebSocket)

Endpoint: `ws://127.0.0.1:45710/rpc`. Mensagens seguem JSON-RPC 2.0: requests com `id`, respostas com `result` ou `error`, notificações do servidor sem `id`. O maior frame aceito do app é de 32 MiB, por causa dos anexos.

### 11.1 Sessão

- Primeiro request obrigatório: `session.hello {token, client: {name, version}, protocol: 2}` -> `{daemonVersion, protocol}`.
- Qualquer outro método antes do hello: erro `-32001` e a conexão fecha. Token errado também dá `-32001`; `protocol` diferente dá `-32004`; nos dois casos a conexão fecha. O hello tem que chegar em até 10 s.
- `Origin` aceito: `http://tauri.localhost`, `tauri://localhost` e `http://localhost:1420` (dev). Qualquer outro `Origin` recebe HTTP 403 antes do upgrade. Sem `Origin` (cliente nativo, testes) é aceito: navegadores sempre mandam o header, e o token continua obrigatório.
- Um cliente lento que deixa acumular mais de 1024 notificações é desconectado e recarrega o estado ao reconectar.
- A versão 2 do protocolo troca `terminal.*` pelo chat (ADR 0001).

### 11.2 Métodos (MVP)

| Método | Params | Result |
|---|---|---|
| `system.status` | | versão, uptime, versão do claude, `runtimeError` (por que os bots não sobem), backlog de entrega, `usage` (uso da conta, 8.1) |
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
| `chat.history` | `botId, before?, limit?` | `ChatItem[]`, mais novo primeiro; `limit` de 1 a 200, 50 se ausente |
| `approvals.answer` | `approvalId, allow, note?` | `Approval` |
| `messages.send` | `botId, body, attachments?` (`[{name, mediaType, data}]`, data em base64) | `Message` |
| `messages.list` | `crewId?, botId?, before?, limit?` | `Message[]` |
| `deliveries.list` | `state?, botId?` | `Delivery[]` |
| `deliveries.retry` | `deliveryId` | `Delivery` |
| `tasks.list` | `crewId?, status?` | `Task[]` |

`Bot` traz também `lastActivity`: o último item do chat resumido em uma linha (`text`, `at`), para a lista de conversas.

### 11.3 Notificações do servidor

`bot.state`, `bot.changed`, `crew.changed`, `chat.item`, `chat.delta`, `message.created`, `delivery.changed`, `task.changed`.

### 11.4 Erros

| Código | Significado |
|---|---|
| `-32001` | não autenticado |
| `-32002` | não encontrado |
| `-32003` | conflito de estado (ex.: bot arquivado, aprovação já respondida) |
| `-32004` | validação |
| `-32005` | runtime indisponível (claude ausente ou versão antiga) |

Além desses, os códigos padrão do JSON-RPC: `-32700` (JSON inválido), `-32600` (request inválido), `-32601` (método desconhecido), `-32602` (params inválidos) e `-32603` (erro interno; a mensagem não traz corpo de mensagem nem token). `crews.archive` e `bots.archive` são idempotentes.

## 12. Dados (SQLite)

Pragmas: `journal_mode=WAL`, `foreign_keys=ON`, `busy_timeout=5000`. Migrations numeradas em `botloft-store/migrations/NNNN_nome.sql`, versão em `PRAGMA user_version`. Tempo em milissegundos Unix (`INTEGER`). Arquivamento é lógico (`archived_at`).

| Tabela | Colunas principais |
|---|---|
| `crews` | `id, name, slug, paused, created_at, archived_at` |
| `bots` | `id, crew_id, name, handle, slug, role, instructions, color, paused, token_hash, session_id, created_at, archived_at` |
| `messages` | `id, crew_id, from_kind (owner/bot/system), from_bot_id, to_bot_id, kind (note/task/result/system), body, task_id, created_at` |
| `attachments` | `id, message_id, name, media_type, size, path, created_at` |
| `deliveries` | `id, message_id, bot_id, state, attempts, next_attempt_at, lease_until, last_error, sent_generation, read_at, updated_at` |
| `tasks` | `id, crew_id, requester_bot_id, assignee_bot_id, status, deadline_at, hops, origin_task_id, result, created_at, updated_at` |
| `chat_items` | `id, bot_id, kind, data (JSON), created_at, updated_at` |
| `approvals` | `id, bot_id, tool_use_id, tool_name, input, status, note, created_at, answered_at` |
| `settings` | `key, value` |

Índices mínimos: `deliveries(state, next_attempt_at)`, `messages(crew_id, created_at)`, `tasks(assignee_bot_id, status)`, `bots(crew_id)`, `chat_items(bot_id, id)`, `attachments(message_id)`.

## 13. Segurança e privacidade

- Daemon escuta **só em 127.0.0.1** no MVP.
- Token do owner: 32 bytes aleatórios em `secrets\owner.token`, ACL com acesso só para o SID do usuário atual (DACL protegida, sem herança).
- Token de bot: 32 bytes aleatórios, novo a cada generation. O valor cru existe apenas no ambiente do processo do bot; o daemon guarda só o **SHA-256**, e só em memória: todo processo de bot morre junto com o daemon (Job Object), então nenhum token sobrevive a um reinício e não há motivo para gravá-lo. A coluna `bots.token_hash` fica sem uso.
- Logs nunca registram tokens, conteúdo de mensagens, anexos nem itens do chat em nível `info`. Bots podem manipular dados sensíveis (inclusive de saúde); o daemon trata corpo de mensagem, anexo e saída de ferramenta como dado pessoal.
- `log_level` vale só para os crates do Botloft; dependências ficam em `warn`, porque em `debug`/`trace` a pilha de WebSocket registra frames, que podem conter mensagens. `RUST_LOG` sobrepõe tudo e é só para depuração local.
- Nome de anexo vira só o nome do arquivo (sem `..`, sem pasta, sem caracteres proibidos no Windows) antes de ir para o disco.
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
    crews/        lista, criar, renomear, pausar; página da crew com timeline e tasks
    bots/         conversas na barra lateral, criar, editar, estado, detalhes
    chat/         conversa com o bot: itens, texto ao vivo, aprovações, compositor com anexos
    messages/     timeline da crew, estado de entrega, deliveries com falha (botão na barra de título, retry)
    tasks/        tarefas da crew (abertas por padrão, todas sob demanda)
    settings/
  lib/
    rpc.ts        cliente JSON-RPC com reconexão e fila de requests
    api.ts        interface BotloftApi (contrato usado pela UI)
    client.ts     implementação real sobre rpc.ts
    fake.ts       FakeBotloft em memória para testes
    host.ts       interface Host: comandos Tauri e janela (15.2)
    tauriHost.ts  implementação real; fakeHost.ts para testes
    protocol.gen.ts   gerado por ts-rs, não editar
  store/          stores Zustand alimentados por notificações
  ui/             componentes base
  dev/            prévia: `pnpm dev` num navegador comum usa FakeBotloft, ou um daemon de dev real com `?live=<porta>` (só em dev)
```

Componentes dependem só de `BotloftApi` e `Host`, nunca do cliente concreto nem do Tauri.

O store recarrega crews, bots e `system.status` a cada (re)conexão e depois segue as notificações. `system.status` não tem notificação e é relido a cada 15 s.

- **Deliveries:** entram no store pela message (cada message tem uma). Vêm as 500 atualizadas mais recentemente e todas as mortas, depois cada `delivery.changed`.
- **Tasks:** todas, depois cada `task.changed`.
- **Chat e timeline** não ficam no store. Cada um carrega uma página (50) do que mostra, pede as anteriores sob demanda com `before` e acrescenta o que chega por notificação (`chat.item`, `chat.delta`, `message.created`).

Layout, como um app de mensagens:

- **Barra lateral:** crews como seções, com os bots como conversas. Cada conversa mostra avatar, nome, estado (cor, ícone e texto) e a prévia da última atividade (`lastActivity`) com a hora.
- **Área principal com um bot:** cabeçalho com nome, estado e ações; o chat; o compositor embaixo. O compositor aceita texto, colar imagem e arrastar ou escolher arquivos. Enter envia e Shift+Enter quebra linha.
- **Área principal com uma crew:** a timeline (messages entre os bots e do dono) e as tasks.
- **Detalhes do bot** (pasta, instruções, sessão) ficam num painel, fora do caminho da conversa.

### 15.2 Comandos Tauri

| Comando | Função |
|---|---|
| `daemon_status` | GET `/health` local; diz se o daemon roda, está parado ou se outro programa ocupa a porta |
| `daemon_start` | (M4) inicia o `botloftd.exe` ao lado do executável do app, destacado (sem console, fora do job do app) para seguir vivo quando o app fecha, e espera o `/health` por até 15 s. No M5 o `daemon_install` o substitui |
| `daemon_install` | copia sidecar e roda `botloftd service install` |
| `daemon_restart` | `botloftd service restart` |
| `read_owner_token` | lê `secrets\owner.token` (só no app local) |
| `open_path` | abre uma pasta no Explorer; recusa arquivos, que o Explorer executaria |
| overlay na taskbar | não é comando próprio: o app usa `setOverlayIcon` da janela (permissão `core:window:allow-set-overlay-icon`) e marca o ícone com um ponto enquanto algo espera o dono: aprovação pendente, bot em `auth_error`, ou mensagem não entregue a um bot ativo. Entregas mortas para bot arquivado não contam: foram abandonadas de propósito |

O app acha o daemon como o daemon acha a si mesmo (seção 5): `BOTLOFT_HOME` ou `%LOCALAPPDATA%\Botloft`, com a porta lida do `config.toml` dessa pasta (45710 se ausente). Um daemon de dev com seu próprio `BOTLOFT_HOME` é encontrado sem configuração extra. A CSP libera `ws://127.0.0.1:*` pelo mesmo motivo.

### 15.3 Direção visual

Ferramenta de trabalho densa e calma: tipografia forte, grid firme, estados dos bots legíveis de longe (cor + ícone + texto, nunca só cor). Sem gradiente, sem sombra pesada, sem visual de template. Tema escuro e claro. Barra de título própria (`decorations: false`) com controles de janela do Windows.

No chat:

- O dono fala em balões à direita; o bot, à esquerda, com markdown.
- Mensagens de outros bots aparecem à esquerda, com o avatar e o nome de quem mandou.
- O que o bot faz com as ferramentas aparece em linhas compactas (ícone, ferramenta, resumo e estado), agrupadas por turno, que abrem para mostrar entrada e saída.
- Pedido de aprovação é um cartão com o que o bot quer fazer e os botões Permitir e Negar.
- Anexos aparecem como miniatura (imagem) ou cartão com nome, tipo e tamanho.

Identidade: o mascote do Botloft é uma chama com olhos, desenhada em vetor em `app/app-icon.svg`. O ícone do app é o mascote branco sobre fundo preto. Cada bot usa o mesmo personagem como avatar, com uma cor própria escolhida na criação. A cor do avatar identifica o bot e não comunica estado: estado continua sendo cor + ícone + texto, como descrito acima.

## 16. Qualidade

- Rust: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`.
- TS: `tsc --noEmit`, `biome check`, `vitest run`.
- Limite **flexível de 300 linhas** por arquivo; passou disso, dividir por responsabilidade.
- Toda lógica de supervisor, courier e chat testada com `FakeRuntime` (sem Claude real). O `FakeRuntime` fala `stream-json` de verdade: recebe as linhas do stdin e o teste escreve os eventos do stdout.
- CI: GitHub Actions em `windows-latest` (principal) e `ubuntu-latest` para `botloft-core` e `botloft-store`.

## 17. Marcos do MVP

| Marco | Entrega | Pronto quando |
|---|---|---|
| **M0** Fundação | Workspace Cargo, app Tauri vazio, CI, LICENSE, README | CI verde nos dois runners |
| **M1** Daemon base | config, store + migrations, `/health`, `/rpc` com hello, CRUD de crews e bots, geração de workspace | teste de integração cria crew e bot via RPC |
| **M2** Runtime | supervisor com estados e backoff, Job Objects, `FakeRuntime` | bot real sobe no Windows e os estados mudam |
| **M3** Mensagens | courier, tools MCP, tasks com hops e prazo | dois bots trocam task e resultado sem intervenção |
| **M4** App | onboarding, crews, bots, timeline, deliveries com falha | fluxo completo pelo app sem abrir terminal |
| **M4.1** Chat | runtime headless (ADR 0001), chat com texto ao vivo, aprovações pelo chat, anexos, lista de conversas | conversar com um bot pelo app, mandar uma imagem, aprovar uma ferramenta e ver o resultado; dois bots trocam task e resultado pelo stdin |
| **M5** Distribuição | tarefa agendada, keep-awake, instalador NSIS com sidecar, updater | instalar em máquina limpa e bots voltarem sozinhos após reboot |

M2 a M4 foram entregues com ConPTY, terminal com replay, inbox por named pipe e hooks. A ADR 0001 troca tudo isso pelo runtime headless no M4.1, antes do M5.

## 18. Fora do MVP (ordem sugerida)

1. Rotinas (cron com timezone, intervalo) com política de sobreposição.
2. Caixa de perguntas ao owner (bot pergunta, owner responde, resposta volta como mensagem).
3. "Permitir sempre" nas aprovações, gravado como regra do bot.
4. Busca FTS5 em mensagens e no chat.
5. Histórico de versões das instruções do bot.
6. Acesso remoto com token por dispositivo (Tailscale).
7. Sinais entre bots disparando rotinas.
8. Suporte Linux/macOS.

## 19. Pontos a verificar na versão alvo do Claude Code

Conferência na documentação oficial (code.claude.com/docs) em 2026-09-28. "Confirmado" quer dizer documentado; o teste real na versão alvo continua no checklist do marco indicado.

| Item | Seção | Resultado | Teste real |
|---|---|---|---|
| Formato de entrada `--input-format stream-json` | 9.2 | **Não documentado**. **Testado com 2.1.284**: `{"type":"user","uuid":...,"message":{"role":"user","content":[blocos]}}` abre um turno; bloco `image` em base64 é aceito | feito (M4.1) |
| Vários turnos num processo; mensagem durante um turno | 9.2 | **Testado com 2.1.284**: dois turnos seguidos no mesmo processo; a mensagem escrita durante um turno virou o turno seguinte. Cada turno começa com `system/init` e termina com `result` | feito (M4.1) |
| Processo sem entrada | 7.2 | **Testado com 2.1.284**: fica calado e vivo até a primeira mensagem; sai com 0 quando o stdin fecha | feito (M4.1) |
| `--replay-user-messages` | 9.1 | Flag no `--help` sem detalhes na documentação. **Testado com 2.1.284**: devolve a mensagem com `isReplay: true` e o mesmo `uuid` quando o turno dela começa, não quando é lida | feito (M4.1) |
| Eventos do `--output-format stream-json` | 8.1 | Parcialmente documentado (`headless`): `system/init`, `stream_event` com `text_delta` (exige `--verbose` e `--include-partial-messages`), `system/api_retry` com `error` (`rate_limit`, `authentication_failed`...), `result`. **Visto com 2.1.284**: também `rate_limit_event` (status, `resetsAt`, uso das janelas de 5 h e 7 dias), `system/post_turn_summary`, `system/task_summary`, `system/thinking_tokens`; erro de API vem em `assistant.error` e `result.terminal_reason` | feito (M4.1) |
| `--permission-prompt-tool` | 10.1 | Confirmado (`cli-reference`, `headless`). **Testado com 2.1.284**: chama a tool com `{tool_name, input, tool_use_id}`, esperou 20 s pela resposta e seguiu `{"behavior":"allow","updatedInput":...}` | tempo longo (minutos) pelo servidor HTTP do daemon: M4.1 |
| `--permission-prompts host` | 10.1 | Documentado só para o SDK. **Visto com 2.1.284**: sem o aperto de mão do SDK, nega tudo (`system/permission_denied`). Não usado | não se aplica |
| `timeout` por servidor MCP | 10 | Confirmado (`env-vars`): HTTP tem 60 s por request e 5 min sem resposta por padrão; `timeout` >= 1000 no servidor sobe os dois | M4.1 |
| `--setting-sources project,local` | 7.4 | Confirmado (`cli-reference`). **Testado com 2.1.284**: sem os hooks, skills e agents do usuário; modo `default`; login da assinatura continua valendo | feito (M4.1) |
| Regras `allow` do projeto em `-p` sem confiança | 7.4 | Confirmado (`permissions`): não são aplicadas numa pasta nunca confiada; `deny` vale sempre. Por isso `--allowedTools mcp__botloft` | M4.1 |
| `--session-id`, `--resume` em `-p` | 7.3 | Confirmado (`cli-reference`, `sessions`): a sessão retoma histórico e modelo; flags como `--mcp-config` têm de ser passadas de novo | retomar depois de reiniciar o daemon: M4.1 |
| Transcript `.jsonl` | 7.3 | Confirmado (`sessions`): formato interno, muda entre versões. O chat do Botloft não depende dele | não se aplica |
| Inbox entre sessões em `-p` | 9 | Documentado como indisponível em `-p`; o courier escreve no stdin | não se aplica |
| Carregamento de `.claude/rules/*.md` sem frontmatter | 5.1 | Confirmado (`memory`) e **testado com 2.1.283** (modo interativo): o bot respondeu nome, handle e crew tirados das regras | repetir em `-p`: M4.1 |
| Expansão `${VAR}` em headers do `mcp.json` | 10 | Confirmado (`mcp`); alguns nomes de credencial conhecidos são lidos vazios, `BOTLOFT_BOT_TOKEN` não é um deles. **Testado com 2.1.284**: o header chegou com o valor da variável de ambiente | feito (M3) |
| Revisão do MCP que o Claude Code usa | 10 | Especificação MCP 2026-07-28 (sem `initialize`) e versões antigas. **Visto com 2.1.284**: manda `server/discover` com os headers de 2026-07-28 e, se falhar, `initialize` com 2025-11-25; o mesmo num servidor stdio | feito (M3) |
| Sintaxe de caminho Windows em permission rules | 7.5 | Confirmado (`permissions`) e **testado com 2.1.283**: ler `secrets\owner.token` deu "File is in a directory that is denied by your permission settings" | repetir em `-p`: M4.1 |

Itens do runtime anterior (ConPTY, hooks em exec form, `crossSessionInbound`, linha de auth do inbox, diálogo de confiança, `ESC[6n` do ConPTY, consultas do terminal no replay) foram verificados no M2–M4 e deixaram de se aplicar com a ADR 0001; o histórico está no git e na ADR.
