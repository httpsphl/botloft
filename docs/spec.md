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
| Binário do daemon | `%LOCALAPPDATA%\Botloft\bin\botloftd.exe` (seção 14) | `--home` |
| App instalado | `%LOCALAPPDATA%\Botloft\` (`Botloft.exe`, o sidecar `botloftd.exe`, `uninstall.exe`; seção 15.4) | |

O instalador por usuário do Tauri põe o app em `%LOCALAPPDATA%\<produto>`, a mesma pasta dos dados. Os nomes não se cruzam, e o desinstalador só apaga os arquivos que instalou e remove a pasta se ela ficar vazia: os dados ficam.

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
| erro `authentication_failed`, `oauth_org_not_allowed`, `billing_error` ou `account_on_hold` | `auth_error`; o processo é parado e o daemon confere o login na hora (`claude auth status`) |
| o login do Claude Code passa de desconectado para conectado | os bots em `auth_error` voltam a `offline` e sobem de novo |
| saída do processo | `backoff` (ou `offline`/`archived` se foi pedido) |

Eventos de uma generation antiga são ignorados. Todo processo novo emite `bot.state {botId, state, generation}`, mesmo que o nome do estado não mude, porque a generation mudou.

### 7.3 Regras

- Bots não pausados sempre rodam. O supervisor reconcilia no boot e a cada 5 s.
- Backoff exponencial com jitter entre `restart_backoff_initial_ms` e `restart_backoff_max_ms`; zera após 10 min de sessão estável.
- **Sessão:** o daemon guarda em `bots.session_id` o id da conversa. O primeiro start usa `--session-id <uuid novo>`; os seguintes, `--resume <session_id>`. Se a sessão retomada morrer em menos de `fresh_start_if_dies_within_s`, o próximo start vem com um id novo. `bots.restart {fresh: true}` também começa conversa nova. O histórico do chat fica no banco do daemon (seção 8) e não depende do transcript do Claude Code.
- Mudanças em nome, papel ou instruções regravam as regras na hora, mas o bot só as lê no próximo start; o daemon não reinicia o bot sozinho.
- `auth_error` não entra em loop de restart: fica parado até o owner pedir `bots.restart` ou até o daemon ver o Claude Code conectado de novo.
- **Login do Claude Code:** o daemon roda `claude auth status` (JSON com `loggedIn` e, conectado, `email`, `subscriptionType` e `orgName`; sai com 0 conectado e 1 desconectado, seção 19) no ambiente do usuário, sem janela: quando acha o Claude Code, a cada 30 s enquanto desconectado ou desconhecido, na hora quando um bot cai em `auth_error` e quando o app pede (`system.refresh`). Conectado, não confere de novo sozinho. Quando o resultado passa de desconectado para conectado, os bots em `auth_error` sobem de novo sem o dono pedir. Um erro de conta com o login válido (`billing_error`, `account_on_hold`, organização bloqueada) não muda o resultado e por isso não vira loop. O resultado sai em `system.status.claudeSignedIn`, e a conta (e-mail, plano, organização) em `system.status.account.claude`. O login em si é feito pelo app (15.2).
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
     --permission-mode <modo do bot>
     [--model <modelo do bot>]
     --permission-prompt-tool mcp__botloft__permission_prompt
     --allowedTools mcp__botloft
   ```

   - `--setting-sources project,local` e `--strict-mcp-config` deixam de fora hooks, skills, agents, modo de permissão e servidores MCP pessoais do dono: o bot vê o que o Botloft gera. O login da conta não é uma fonte de settings e continua valendo.
   - `--allowedTools mcp__botloft` libera as tools da crew sem aprovação. Uma regra `allow` no `settings.json` do projeto não bastaria: em `-p`, numa pasta que nunca passou pelo diálogo de confiança, o Claude Code não aplica as regras `allow` do projeto (documentado em `permissions`).
   - Qualquer outra ferramenta que peça permissão passa pela tool de aprovação (10.1) e vira um pedido no chat.
   - `--permission-mode` vem do modo do bot (`Bot.permissionMode`), que o dono escolhe no chat (15.1): `default` (Manual: pergunta antes de editar, rodar comandos e usar a rede), `accept_edits` (`acceptEdits`: edita arquivos sem perguntar), `plan` (planeja e pede para seguir, 10.1), `auto` (um classificador libera o que é seguro e o resto vira pedido) e `bypass_permissions` (`bypassPermissions`: faz tudo sem perguntar, ver 13). Bot novo começa em `default`. `dontAsk` não é oferecido: nega o que não estiver liberado, e o bot não teria como pedir. Regras `deny` valem em todos os modos.
   - O Claude Code só lê a flag ao iniciar e não entra em `bypassPermissions` no meio da sessão. Por isso mudar o modo reinicia o processo com `--resume`, e a conversa continua: na hora se o bot está parado, ou quando o turno em andamento e as aprovações dele terminam, sem cortar o trabalho.
   - `--model` vem do modelo do bot (`Bot.model`), que o dono escolhe no chat ou ao criar o bot (15.1): `fable`, `opus`, `sonnet` ou `haiku`, os apelidos do Claude Code, que apontam sempre para a versão mais nova de cada família. `default` deixa a flag de fora, e o Claude Code usa o padrão da conta, que depende do plano (19). Bot novo começa em `default`. O Claude Code não troca de modelo sozinho conforme a tarefa. Mudar o modelo reinicia o processo como a troca de modo, e `--resume` com `--model` passa a usar o modelo novo. O `system/init` de cada turno traz o modelo em uso (`claude-opus-5-5`); o daemon o guarda em `Bot.modelInUse` para o app dizer qual é o padrão do plano. Um modelo que a conta não pode usar falha o turno com o erro `model_not_found` (8.1), e o processo continua vivo.
   - O Claude Code sai do modo `plan` sozinho quando o dono aprova o plano. Cada turno começa com um `system/init` que traz `permissionMode` (19); se o processo começou em `plan`, o modo gravado ainda é `plan` e o init diz outro, o daemon grava o novo, sem reiniciar, para o app mostrar o que o bot faz e um reinício manter. Qualquer outra diferença vem de um processo que está para reiniciar no modo que o dono acabou de escolher, e é ignorada.
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
| `system/init` | guarda `session_id` em `bots.session_id` (vem no começo de cada turno); segue `permissionMode` e guarda `model` (7.4) |
| `user` com `isReplay: true` | a message com aquele `uuid` começou a ser processada: a delivery ganha `read_at` (9.1) |
| `stream_event` com `text_delta` | texto ao vivo da resposta (`chat.delta`, 8.3); não é gravado |
| `assistant`, bloco `text` | item `reply` com o texto (markdown) |
| `assistant`, bloco `tool_use` | item `tool` em `running`, com resumo da entrada |
| `user`, bloco `tool_result` | atualiza o item `tool` do mesmo `tool_use_id`: `done` ou `failed` e um trecho da saída |
| `assistant` com `error` | item `notice` e, conforme o erro, estado `rate_limited` ou `auth_error` (7.2); `model_not_found` vira o aviso `model_unavailable` |
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
| `notice` | `level` (`info`, `warning`, `error`), `code` (`signed_out`, `usage_limit`, `turn_failed`, `model_unavailable`; ausente em avisos antigos), `text` | avisos do daemon: limite de uso, login, turno com erro. O app escreve os avisos com `code` no idioma do dono; `text` fica em inglês para quem não conhece o código e, em `turn_failed`, traz o detalhe do erro |

- `summary` é uma frase curta feita pelo daemon a partir da entrada: o comando do `Bash`, o arquivo do `Read`/`Edit`/`Write`, o padrão do `Grep`/`Glob`, a URL do `WebFetch`, a busca do `WebSearch`, o destinatário do `send_message`. Ferramenta desconhecida mostra só o nome.
- `input` guarda o JSON da entrada até 4 KB; `output`, até 8 KB de texto. O resto fica só no transcript do próprio Claude Code.
- A resposta do bot (`reply`) é guardada inteira.
- Uma message de um bot para outro aparece duas vezes: no chat de quem mandou, como o item `tool` do `send_message`; no chat de quem recebe, como `inbound`.

### 8.3 Ao vivo

- `chat.item {item, activity}`: item novo ou atualizado (tool que terminou, aprovação respondida). `activity` é a nova linha da conversa na barra lateral, quando mudou.
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

- `uuid` é um UUID v4 novo a cada envio, guardado na delivery (`turn_uuid`). O Claude Code o devolve no replay (visto com 2.1.284).
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
- Imagens aparecem como miniatura. O app lê o arquivo de volta com `attachments.read`, que devolve os bytes como estão agora na pasta do bot (o bot pode ter mudado ou apagado o arquivo; apagado dá `not_found`). Arquivo maior que `attachment_max_mb` não é lido.

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

**Plano.** No modo `plan`, o bot pede para seguir com a ferramenta `ExitPlanMode`, cuja entrada traz o plano em markdown (`{plan}`). O pedido passa pela mesma tool e vira no chat um cartão com o plano inteiro: a entrada dessa ferramenta é guardada até 32 KB (as outras, até 4 KB) e o resumo é a primeira linha do plano. "Aprovar plano" permite; "Pedir mudanças" nega com a nota do dono, e o bot continua planejando. Depois de aprovado, o Claude Code troca de modo sozinho (7.4). Que o pedido passa pela tool em `-p` e para qual modo o bot vai ainda precisam de teste real (19).

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
| `system.status` | | versão, uptime, versão e caminho do claude (`claudeVersion`, `claudePath`), `runtimeError` (por que os bots não sobem), `claudeSignedIn` (7.3; `null` antes de conferir), `account` (o dono para a área da conta do app: `name`, o nome de exibição da conta do Windows, `GetUserNameExW(NameDisplay)`, ou o nome de usuário sem ele, lido uma vez; e `claude`, a conta do Claude com `email`, `plan` e `organization`, ou `null` desconectado), backlog de entrega, `usage` (uso da conta, 8.1) |
| `system.refresh` | | pede uma nova conferência do Claude Code (login e, se falhou, o executável) e responde na hora com o `system.status` atual; o app relê o status até ver o resultado |
| `crews.list` | | `Crew[]` |
| `crews.create` | `name` | `Crew` |
| `crews.rename` | `crewId, name` | `Crew` |
| `crews.setPaused` | `crewId, paused` | `Crew` |
| `crews.archive` | `crewId` | `Crew` |
| `bots.list` | `crewId?` | `Bot[]` |
| `bots.create` | `crewId, name, role, instructions, color?, model?` | `Bot` |
| `bots.update` | `botId, name?, role?, instructions?, color?` | `Bot` |
| `bots.setPaused` | `botId, paused` | `Bot` |
| `bots.setPermissionMode` | `botId, mode` (`default`, `accept_edits`, `plan`, `auto`, `bypass_permissions`) | `Bot`; o bot reinicia no novo modo quando nada estiver em andamento (7.4) |
| `bots.setModel` | `botId, model` (`default`, `fable`, `opus`, `sonnet`, `haiku`) | `Bot`; o bot reinicia no novo modelo quando nada estiver em andamento (7.4) |
| `bots.restart` | `botId, fresh?` | `Bot` |
| `bots.archive` | `botId` | `Bot` |
| `chat.history` | `botId, before?, limit?` | `ChatItem[]`, mais novo primeiro; `limit` de 1 a 200, 50 se ausente |
| `approvals.answer` | `approvalId, allow, note?` | `Approval` |
| `messages.send` | `botId, body, attachments?` (`[{name, mediaType, data}]`, data em base64) | `Message` |
| `messages.list` | `crewId?, botId?, before?, limit?` | `Message[]` |
| `attachments.read` | `attachmentId` | `{mediaType, data}`, data em base64 (9.5) |
| `deliveries.list` | `state?, botId?` | `Delivery[]` |
| `deliveries.retry` | `deliveryId` | `Delivery` |
| `tasks.list` | `crewId?, status?` | `Task[]` |

`Bot` traz também `permissionMode`, `model` e `modelInUse` (7.4) e `lastActivity`: o último item do chat resumido em uma linha, para a lista de conversas: `kind` (`owner`, `message`, `reply`, `tool`, `approval`, `notice`), `text` e `at`. O `text` não tem palavras do daemon: a mensagem do dono vem sem "You:" e a aprovação só com o nome da ferramenta, e o app completa no idioma do dono.

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
| `bots` | `id, crew_id, name, handle, slug, role, instructions, color, paused, permission_mode, model, model_in_use, token_hash, session_id, created_at, archived_at` |
| `messages` | `id, crew_id, from_kind (owner/bot/system), from_bot_id, to_bot_id, kind (note/task/result/system), body, task_id, created_at` |
| `attachments` | `id, message_id, name, media_type, size, path, created_at` |
| `deliveries` | `id, message_id, bot_id, state, attempts, next_attempt_at, lease_until, last_error, sent_generation, turn_uuid, read_at, updated_at` |
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
- Modo `bypass_permissions` (7.4): o bot faz tudo sem perguntar. As regras `deny` de leitura só cobrem as ferramentas de arquivo do Claude Code, alguns comandos do Bash (`cat`, `head`, `tail`, `sed`, `tee`) e redirecionamentos, não um script em Python ou Node, e o Windows nativo não tem sandbox. Um bot nesse modo pode ler `secrets\owner.token`, o banco e as pastas de outros bots, e uma mensagem de outra pessoa pode levá-lo a isso. O app só liga o modo depois de uma confirmação que diz isso, e o bot fica marcado em vermelho (15.1).

## 14. Integração com o Windows

| Tema | Solução |
|---|---|
| Iniciar com o Windows | `botloftd service install` registra uma **Tarefa Agendada por usuário**, pela API COM do Agendador (as mensagens do `schtasks.exe` são traduzidas e não dá para lê-las). Dois gatilhos: "ao fazer logon" do usuário, que sobe o daemon na hora, e um gatilho de horário com início no passado repetido **a cada 1 min**, que o traz de volta se ele morrer: com `MultipleInstancesPolicy = IgnoreNew`, a repetição não faz nada enquanto o daemon roda. Token interativo e privilégio mínimo (só com o usuário logado, na sessão dele), sem limite de execução, roda na bateria, prioridade 5 (a padrão, 7, passaria "abaixo do normal" para todos os bots). A ação é `<home>\bin\botloftd.exe serve --home <home>`. Não usar Windows Service: roda em outra sessão e sem acesso à autenticação do Claude Code do usuário. Subcomandos `service status`, `service restart`, `service uninstall` (os dados ficam). |
| Reinício da tarefa (testado no Windows 11 25H2) | `RestartOnFailure` **não** reinicia a tarefa quando o processo sai com código de erro, nem numa execução por gatilho de horário nem numa sob demanda. A repetição de um gatilho de logon só começa no próximo logon, não quando a tarefa é registrada. Por isso o gatilho de horário faz o papel de vigia: morto o daemon, ele voltou em 43 s |
| Uma tarefa por pasta de dados | `Botloft` para `%LOCALAPPDATA%\Botloft`; `Botloft-<8 hex do SHA-256 do caminho>` para outra pasta (`--home` ou `BOTLOFT_HOME` de dev), para instalar um daemon de dev sem tocar no real. `--home` vale para todos os subcomandos e vem antes de `BOTLOFT_HOME` |
| Binário instalado | `service install` copia o próprio executável para `<home>\bin\botloftd.exe` e a tarefa roda essa cópia, nunca a do app: assim o instalador do app troca os arquivos dele com o daemon rodando. Um exe em uso não pode ser sobrescrito mas pode ser renomeado, então o antigo vai para `botloftd.<n>.old` e é apagado numa instalação seguinte. Se o binário não mudou e o daemon dessa versão já roda pela tarefa, `install` não o reinicia; senão para a tarefa, espera o `/health` sumir, inicia de novo e espera o `/health` com a nova versão (20 s) |
| Janela de console | O manifesto do daemon pede `consoleAllocationPolicy = detached` (Windows 11 24H2 e depois): iniciado pela tarefa, ele não ganha console nem janela; num terminal, continua usando o console do terminal. Em Windows mais antigo, `serve` larga o console se for o único processo nele (`FreeConsole`). O manifesto entra como recurso (`embed-resource`), porque a ferramenta de manifesto do linker não conhece o elemento e avisa a cada build |
| Não suspender | Um power request (`PowerCreateRequest` + `PowerSetRequest(PowerRequestSystemRequired)`, motivo "Botloft bots are working") enquanto houver bot `busy`, se `keep_awake = true`. Bot esperando aprovação não conta. Diferente de `SetThreadExecutionState`, o pedido é de um handle e não de uma thread, então qualquer worker do Tokio o liga e desliga, e `powercfg /requests` o mostra. Não impede a suspensão pedida pelo usuário (tampa, menu Iniciar) |
| Processos órfãos | Job Object por bot (seção 7.3) |
| Encerramento | `ctrl_c`, `ctrl_close`, `ctrl_shutdown`, `ctrl_logoff` do Tokio: parar courier, sinalizar bots, flush do banco. Esses sinais são de console: iniciado pela tarefa, sem console, o daemon não tem garantia de recebê-los, e o Windows encerra o processo no logoff e no desligamento. Isso equivale a um crash, e o sistema já é feito para ele (mensagens gravadas antes de sair, bots mortos pelo Job Object, aprovações abertas expiram no próximo start). Um erro fatal na subida vai para o log, porque ninguém vê o stderr da tarefa |
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

- **Barra lateral:** crews como seções, com os bots como conversas. Cada conversa mostra avatar, nome, estado (cor, ícone e texto) e a prévia da última atividade (`lastActivity`) com a hora. Ela aparece desde a conexão, também na tela de boas-vindas antes da primeira crew.
- **Área da conta**, no pé da barra lateral, como nos apps de chat: um círculo com a inicial, o nome do dono (do Windows) e o plano do Claude ("Plano Max"; sem plano, a organização ou o e-mail; desconectado, "Sem conta do Claude conectada"). Um clique abre um menu para cima com o e-mail e: **Uso** (as janelas de uso do plano, 8.1, com a parte usada e quando renovam; antes da primeira resposta de um bot, um aviso de que ainda não há dados), **Configurações** (tema, tamanho, idioma e as versões do Botloft e do Claude Code), **Idioma** (submenu ao lado, com a escolha na hora), **Novidades** (a página de releases no navegador) e **Ajuda** (o README no navegador). Tema, tamanho e idioma saíram da barra de título; ela só mostra o idioma nas telas de preparo, que ainda não têm barra lateral.
- **Área principal com um bot:** cabeçalho com nome, estado e ações; o chat; o compositor embaixo. O compositor aceita texto, colar imagem e arrastar ou escolher arquivos. Enter envia e Shift+Enter quebra linha (a dica aparece enquanto o dono escreve).
- **Modo do bot**, no compositor, ao lado do clipe, como no Claude Code: um botão com o modo atual abre um menu para cima, "Modo", com Automático, Manual, Aceitar edições e Plano, cada um com uma linha que fala do bot pelo nome ("Scout decide o que precisa do seu OK") e a marca no atual. Separado, "Ignorar permissões" com o botão Ativar, que abre uma confirmação dizendo que o bot não fica preso à pasta dele (13). Nesse modo o botão fica vermelho e o cabeçalho mostra "Não pergunta nada" em vermelho. Com o bot ocupado, uma linha acima do compositor avisa que ele muda de modo quando terminar o que está fazendo.
- **Modelo do bot**, no compositor, ao lado do botão de enviar: mostra o modelo em uso ("Opus 5.5"; antes do primeiro turno, o nome escolhido ou "Padrão") e abre um menu para cima, "Modelo", com Padrão do plano, Fable, Opus, Sonnet e Haiku. Cada um tem uma linha que fala do bot pelo nome ("Scout fica rápido e capaz, bom para quase tudo"; o padrão diz qual modelo é hoje), e o pé do menu lembra que modelos mais capazes gastam o limite do plano mais rápido. A janela de criar e editar bot tem a mesma escolha. Com o bot ocupado, a mesma linha acima do compositor avisa que ele troca quando terminar.
- **Área principal com uma crew:** a timeline (messages entre os bots e do dono) e as tasks.
- **Detalhes do bot** (pasta, instruções, sessão) ficam num painel, fora do caminho da conversa.

### 15.2 Comandos Tauri

| Comando | Função |
|---|---|
| `daemon_status` | GET `/health` local; diz se o daemon roda, está parado ou se outro programa ocupa a porta. Quando o daemon roda, diz também se ele é `outdated`: versão menor que a do app, que é a do sidecar (um workspace Cargo só). Pré-release é ignorado; versão ilegível nunca é antiga |
| `daemon_install` | roda `botloftd --home <home> service install` a partir do sidecar (o `botloftd.exe` ao lado do app), sem janela, e espera: ele se copia para `<home>\bin`, registra e inicia a tarefa e espera o `/health` (seção 14). Um erro volta na mensagem de uma linha que o daemon escreve no stderr. Substitui o `daemon_start` do M4, que iniciava o daemon destacado ao lado do app |
| `daemon_restart` | `botloftd --home <home> service restart`, do mesmo jeito |
| `claude_sign_in` | abre `claude auth login` numa janela de console própria (`CREATE_NEW_CONSOLE`) com o `claudePath` do `system.status` e espera; responde se saiu com 0. Só roda um caminho absoluto de arquivo existente chamado `claude.exe`. Tira do ambiente `CLAUDECODE` e `CLAUDE_CODE_*`, que um app aberto de dentro de uma sessão do Claude Code herdaria. Quem abre é o app, e não o daemon, porque a janela aberta pelo app em primeiro plano vem para a frente. Na janela o dono vê o que acontece e pode colar um código se o navegador pedir |
| `read_owner_token` | lê `secrets\owner.token` (só no app local) |
| `open_path` | abre uma pasta no Explorer; recusa arquivos, que o Explorer executaria |
| `open_url` | abre no navegador padrão um link de uma resposta do bot; só `http` e `https`, porque qualquer outro esquema pode iniciar um programa. Seguir o link dentro do app trocaria a janela pela página |
| overlay na taskbar | não é comando próprio: o app usa `setOverlayIcon` da janela (permissão `core:window:allow-set-overlay-icon`) e marca o ícone com um ponto enquanto algo espera o dono: aprovação pendente, bot em `auth_error`, ou mensagem não entregue a um bot ativo. Entregas mortas para bot arquivado não contam: foram abandonadas de propósito |

O app acha o daemon como o daemon acha a si mesmo (seção 5): `BOTLOFT_HOME` ou `%LOCALAPPDATA%\Botloft`, com a porta lida do `config.toml` dessa pasta (45710 se ausente). Um daemon de dev com seu próprio `BOTLOFT_HOME` é encontrado sem configuração extra. A CSP libera `ws://127.0.0.1:*` pelo mesmo motivo.

Onboarding (M5): **configuração sem perguntas e sem jargão.** O dono não precisa saber que existe um daemon, uma porta ou uma tarefa agendada; a interface nunca usa essas palavras. Fala de "Botloft" e de "rodar em segundo plano", e o texto técnico (erro do daemon, caminho, porta) fica dobrado sob "Details".

- Nada rodando: o app chama `daemon_install` sozinho ("Getting Botloft ready…", com uma linha dizendo que o Botloft segue rodando em segundo plano depois que a janela fecha e inicia com o Windows). Se falhar, "Botloft couldn't start" com "Try again".
- Daemon `outdated`: o app o atualiza sozinho ("Updating Botloft…"), porque app e daemon são distribuídos juntos e quem atualizou o app espera o daemon novo. Os bots em turno são interrompidos e retomam a sessão (7.3). Se falhar ou a versão não mudar, "Botloft couldn't finish updating" com "Try again".
- Cada verificação instala ou atualiza no máximo uma vez: uma falha aparece em vez de virar loop.
- Daemon mais novo que o app com outro protocolo: pede para instalar a versão mais recente do Botloft.
- Porta ocupada por outro programa: diz que outro programa está no caminho; porta e `config.toml` ficam em "Details".
- Conectando sem sucesso: "Try again" e "Restart Botloft in the background" (`daemon_restart`).
- Claude Code ausente ou inutilizável (`runtimeError`), na tela de boas-vindas e no aviso "Bots can't start": explica que os bots rodam no Claude Code, oferece "How to install Claude Code" (abre `https://code.claude.com/docs/en/setup`) e diz que o Botloft percebe sozinho em até 30 s; o erro vai em "Details".

- Login do Claude (`claudeSignedIn`): a tela de boas-vindas mostra "Claude account" (conectado, desconectado com o botão, ou conferindo). Com crews, desconectado vira o aviso "Sign in to Claude" no topo; um bot em `auth_error` também mostra o botão. "Sign in to Claude" chama `claude_sign_in`; se terminou bem, chama `system.refresh` e relê o status a cada 1 s por até 20 s. Os bots parados voltam sozinhos (7.3). Se a janela fechar antes, "The sign-in didn't finish" com o botão de novo.

Em dev, `pnpm tauri dev` usa o `target\debug\botloftd.exe` como sidecar e instala uma tarefa própria da pasta de dev (seção 14). Quem prefere um daemon em primeiro plano roda `cargo run -p botloftd -- serve` antes de abrir o app.

### 15.3 Direção visual

Ferramenta de trabalho densa e calma: tipografia forte, grid firme, estados dos bots legíveis de longe (cor + ícone + texto, nunca só cor). Sem gradiente, sem sombra pesada, sem visual de template. Cantos arredondados, como nos apps de chat: botões, campos e linhas de menu com 8 px; cartões, avisos e menus com 12 px; compositor, diálogos e cartões do chat com 16 px. Menus e diálogos flutuam com uma sombra leve (`shadow-lift`, mais forte no tema escuro). Na barra lateral, a conversa aberta é um bloco arredondado recuado das bordas. Tema escuro e claro. Barra de título própria (`decorations: false`) com controles de janela do Windows.

Tamanho: a janela inteira é desenhada numa escala (zoom do webview, `setZoom`), então texto, espaçamento, ícones e avatares crescem juntos. Níveis 100%, 110%, 125% e 150%; o padrão é **125%**, porque o desenho a 100% ficava miúdo num monitor sem ampliação do Windows. Quem já usa a ampliação alta volta para 100% em Configurações (área da conta) ou com Ctrl+-; Ctrl+= aumenta e Ctrl+0 volta ao padrão. A escolha fica no `localStorage` (`botloft.zoom`) e é aplicada antes da primeira pintura, para a janela não piscar pequena. Acima de 150% a janela mínima (900 px) não cabe o layout.

No chat:

- O chat é uma coluna centralizada (48rem), com o compositor na mesma largura; o dia é uma pílula no meio.
- O dono fala em balões à direita; o bot, à esquerda, com markdown.
- Mensagens de outros bots aparecem à esquerda, com o avatar e o nome de quem mandou.
- O que o bot faz com as ferramentas aparece em linhas compactas (ícone, ferramenta, resumo e estado), agrupadas por turno, que abrem para mostrar entrada e saída.
- Pedido de aprovação é um cartão com o que o bot quer fazer e os botões Permitir e Negar.
- Pedido para seguir com um plano (10.1) é um cartão com o plano em markdown, um campo para o que deve mudar e os botões Aprovar plano e Pedir mudanças. Respondido, vira uma linha que abre o plano de novo.
- Anexos aparecem como miniatura (imagem) ou cartão com nome, tipo e tamanho.

Identidade: o mascote do Botloft é uma chama com olhos, desenhada em vetor em `app/app-icon.svg`. O ícone do app é o mascote branco sobre fundo preto. Cada bot usa o mesmo personagem como avatar, com uma cor própria escolhida na criação, sem fundo e com um contorno fino e discreto (escuro no tema claro, claro no escuro) para as cores claras não sumirem. O mascote branco do próprio Botloft (barra de título, mensagens do daemon) fica sobre o quadrado preto do ícone do app, que é o que o torna visível no tema claro. A cor do avatar identifica o bot e não comunica estado: estado continua sendo cor + ícone + texto, como descrito acima.

### 15.4 Instalador e sidecar

- `pnpm bundle` (em `app/`) roda `scripts/sidecar.mjs`, que compila o `botloftd` em release e o copia para `src-tauri/binaries/botloftd-<target triple>.exe`, e depois `tauri build --config src-tauri/tauri.bundle.conf.json`. Esse arquivo liga o `externalBin` e o NSIS; ele fica fora do `tauri.conf.json` porque o `tauri-build` copia o `externalBin` também em dev e no `cargo clippy`, o que exigiria o sidecar em todo build e sobrescreveria o `target\debug\botloftd.exe` com a cópia de release.
- NSIS por usuário (`installMode = currentUser`), sem pedir administrador, em `%LOCALAPPDATA%\Botloft` (seção 5). O instalador não mexe no daemon: ele roda da própria cópia em `<home>\bin`, então o arquivo do sidecar nunca está em uso.
- Hook `NSIS_HOOK_PREUNINSTALL` (`src-tauri/windows/hooks.nsh`): numa desinstalação de verdade, `botloftd service uninstall` para o daemon e apaga a tarefa e o binário; bots e dados ficam. Uma atualização também roda o desinstalador antigo, com `/UPDATE`: aí o hook não faz nada e o daemon continua rodando até o app novo abrir e atualizá-lo (15.2).
- O manifesto do app (`src-tauri/windows/app.manifest`) é o padrão do Tauri (controles comuns v6) mais `longPathAware`.

### 15.5 Atualizações

- O app usa o `tauri-plugin-updater`. O feed é `latest.json` do último release publicado no GitHub (`releases/latest/download/latest.json`). O app o consulta ao abrir e a cada 6 h; sem rede ou sem release, fica quieto. Build de dev não consulta, para não se trocar pelo app publicado.
- Com versão nova, aparece "Update available" na barra de título. Um clique abre o diálogo: a versão, "What's new" (se o release tiver notas) e o aviso de que o Botloft fecha, instala e abre de novo, e de que os bots pausam por um instante e continuam de onde pararam. "Update now" baixa com progresso e roda o instalador; se falhar, o erro fica em "Details" e dá para tentar de novo.
- O instalador roda como `/P /UPDATE /R`: passivo, sem perguntas, e reabre o app. O hook de desinstalação não faz nada com `/UPDATE` (15.4), o daemon segue rodando, e o app reaberto o atualiza porque ele ficou `outdated` (15.2).
- Os artefatos são assinados com a chave do updater do Tauri (`createUpdaterArtifacts`), e a chave pública fica no `tauri.conf.json`. `requireSignedVersion` exige que a assinatura traga a versão, para um feed adulterado não empurrar uma versão antiga de volta. O instalador não tem assinatura Authenticode, então o SmartScreen avisa na primeira execução.
- Release: a versão fica só no `[workspace.package]` do `Cargo.toml` (o `tauri.conf.json` não repete a versão e usa a do crate). Um push de tag `vX.Y.Z` roda `.github/workflows/release.yml`, que confere tag e versão, roda `pnpm bundle` com `TAURI_SIGNING_PRIVATE_KEY` (segredo do repositório) e abre um release **rascunho** com o instalador, o `.sig` e o `latest.json` (`app/scripts/release.mjs`). O updater só enxerga o release depois que o dono o publica.

### 15.6 Idiomas

- O app fala **inglês, português (Brasil) e espanhol**. Todo texto que o dono lê fica em `app/src/i18n/<idioma>/`, um arquivo por área (`common`, `shell`, `onboarding`, `updates`, `bots`, `chat`, `crews`, `messages`). O inglês é a referência: o formato dele é o tipo `Messages`, e um texto que falte ou sobre em outro idioma não compila. Texto com valores é função (`ready(version)`), com o plural escrito para cada idioma.
- Componentes leem com `useT()`; código fora do React (toasts, formatação, erros da conexão) com `t()` na hora do uso.
- Escolha na área da conta (Idioma, ou Configurações), e no botão de idioma da barra de título só nas telas de preparo: "Idioma do sistema" segue o Windows (o primeiro idioma suportado entre os preferidos; `pt-PT` vira `pt-BR`; nenhum, inglês) ou um idioma fixo. A escolha fica no `localStorage` do app (`botloft.locale`) e marca `<html lang>`.
- Datas e horas (`lib/format.ts`) usam o idioma escolhido.
- O daemon não escreve texto para o dono: avisos vêm com `code` e a linha da conversa com `kind` (8.2, 11.2), e o app escreve. Continuam como vêm: nomes, mensagens, respostas e saídas de ferramenta; o resumo de ferramenta que o daemon faz a partir da entrada (o comando, o arquivo); e as mensagens de erro do daemon (validação, falhas), mostradas como texto técnico.
- O instalador NSIS também traz os três idiomas e escolhe pelo idioma do Windows.
- Tom: palavras simples, sem jargão; "você" em português, "tú" em espanhol. Glossário: crew = equipe / equipo; task = tarefa / tarea; Allow / Deny = Permitir / Negar / Denegar; role = função / rol.

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
| `--permission-prompt-tool` | 10.1 | Confirmado (`cli-reference`, `headless`). **Testado com 2.1.284**: chama a tool com `{tool_name, input, tool_use_id}` e segue `{"behavior":"allow","updatedInput":...}`. Pelo daemon e pelo app: permitir depois de 95 s funcionou; negar com nota fez o bot citar a nota e não usar a ferramenta | feito (M4.1) |
| `--permission-prompts host` | 10.1 | Documentado só para o SDK. **Visto com 2.1.284**: sem o aperto de mão do SDK, nega tudo (`system/permission_denied`). Não usado | não se aplica |
| `timeout` por servidor MCP | 10 | Confirmado (`env-vars`): HTTP tem 60 s por request e 5 min sem resposta por padrão; `timeout` >= 1000 no servidor sobe os dois. **Testado com 2.1.284**: a aprovação respondida depois de 95 s chegou ao bot | feito (M4.1) |
| `--setting-sources project,local` | 7.4 | Confirmado (`cli-reference`). **Testado com 2.1.284**: sem os hooks, skills e agents do usuário; modo `default`; login da assinatura continua valendo | feito (M4.1) |
| `claude auth status` | 7.3 | Confirmado (`cli-reference`): JSON por padrão, sai com 0 conectado e 1 desconectado. **Testado com 2.1.284**: conectado traz `"loggedIn": true`; com `CLAUDE_CONFIG_DIR` vazio, `"loggedIn": false`, `"authMethod": "none"` e saída 1. As credenciais ficam em `%USERPROFILE%\.claude\.credentials.json` (`authentication`) | feito |
| `claude auth login` | 15.2 | Confirmado (`cli-reference`, `authentication`): abre o navegador e volta por um servidor local; se o navegador não alcançar esse servidor, mostra um código para colar no terminal. Se o subcomando sai sozinho depois do login ainda não foi visto: o app só depende do código de saída | manual (PR) |
| `permissionMode` no `system/init` | 7.4 | **Visto com 2.1.284**: o `system/init` de cada turno traz `permissionMode` com o valor da CLI (`default`, `acceptEdits`, `plan`, `auto`, `bypassPermissions`) | feito |
| Modos de permissão em `-p` | 7.4 | Confirmado (`permission-modes`): `deny` vale em todos os modos, inclusive `bypassPermissions`, que não pode ser ligado no meio da sessão; em `-p`, `auto` manda para a tool de aprovação o que o classificador não libera e `plan` continua bloqueando edições. O `auto` depende do plano e do modelo; o que `--permission-mode auto` faz sem ele ainda não foi visto | manual (PR) |
| `ExitPlanMode` em `-p` | 10.1 | A lista de ferramentas diz que ele pede permissão. Não visto: se o pedido chega à tool de aprovação com `{plan}` na entrada, e para qual modo o bot vai depois de aprovado (o daemon segue o `system/init` do turno seguinte) | manual (PR) |
| Modelo em `-p` | 7.4 | Confirmado (`model-config`, `sessions`): apelidos `fable`, `opus`, `sonnet`, `haiku` (e `best`, `opusplan`, `[1m]`, não usados); sem `model` nas settings, vale o padrão da conta; `--resume` mantém o modelo da sessão, a menos que `--model` escolha outro. **Visto com 2.1.284** (plano Max): sem `--model`, o `system/init` traz `"model": "claude-opus-5-5"`; um modelo inexistente sobe o processo, o init o repete, e cada turno termina com `assistant.error: "model_not_found"` e `result.is_error`; `/model haiku` mandado como mensagem troca o modelo no meio da sessão, com uma resposta sintética e sem custo, mas o daemon reinicia o processo em vez disso, como na troca de modo. Qual é o padrão em cada plano, e quais modelos cada plano tem, não está documentado | feito |
| Regras `allow` do projeto em `-p` sem confiança | 7.4 | Confirmado (`permissions`): não são aplicadas numa pasta nunca confiada; `deny` vale sempre. Por isso `--allowedTools mcp__botloft` | M4.1 |
| `--session-id`, `--resume` em `-p` | 7.3 | Confirmado (`cli-reference`, `sessions`): a sessão retoma histórico e modelo; flags como `--mcp-config` têm de ser passadas de novo. **Testado com 2.1.284**: depois de reiniciar o bot e depois de reiniciar o daemon, o bot lembrou arquivos, a imagem e a mensagem de outro bot | feito (M4.1) |
| Tools MCP adiadas | 10 | **Visto com 2.1.284**: as tools do `botloft` chegam adiadas; antes da primeira `send_message` o bot chama `ToolSearch` com `select:mcp__botloft__send_message`. O chat mostra isso como "load send_message" | feito (M4.1) |
| Imagem inline na entrada | 9.5 | **Visto com 2.1.284**: o Claude Code guarda cada bloco `image` recebido em `%TEMP%\claude\<projeto>\<sessão>\images\<n>.png`, e o bot pode abrir essa cópia com `Read` sem pedir aprovação. O anexo original continua na pasta do bot | feito (M4.1) |
| Transcript `.jsonl` | 7.3 | Confirmado (`sessions`): formato interno, muda entre versões. O chat do Botloft não depende dele | não se aplica |
| Inbox entre sessões em `-p` | 9 | Documentado como indisponível em `-p`; o courier escreve no stdin | não se aplica |
| Carregamento de `.claude/rules/*.md` sem frontmatter | 5.1 | Confirmado (`memory`) e **testado com 2.1.283** (modo interativo) e **2.1.284** (`-p`): o bot respondeu nome, handle e crew tirados das regras | feito (M4.1) |
| Expansão `${VAR}` em headers do `mcp.json` | 10 | Confirmado (`mcp`); alguns nomes de credencial conhecidos são lidos vazios, `BOTLOFT_BOT_TOKEN` não é um deles. **Testado com 2.1.284**: o header chegou com o valor da variável de ambiente | feito (M3) |
| Revisão do MCP que o Claude Code usa | 10 | Especificação MCP 2026-07-28 (sem `initialize`) e versões antigas. **Visto com 2.1.284**: manda `server/discover` com os headers de 2026-07-28 e, se falhar, `initialize` com 2025-11-25; o mesmo num servidor stdio | feito (M3) |
| Sintaxe de caminho Windows em permission rules | 7.5 | Confirmado (`permissions`) e **testado com 2.1.283** (interativo) e **2.1.284** (`-p --setting-sources project,local`): `Read(//c/.../**)` em `deny` bloqueou a leitura com "File is in a directory that is denied by your permission settings", sem perguntar, e o `result` listou a negação em `permission_denials` | feito (M4.1) |

Itens do runtime anterior (ConPTY, hooks em exec form, `crossSessionInbound`, linha de auth do inbox, diálogo de confiança, `ESC[6n` do ConPTY, consultas do terminal no replay) foram verificados no M2–M4 e deixaram de se aplicar com a ADR 0001; o histórico está no git e na ADR.
