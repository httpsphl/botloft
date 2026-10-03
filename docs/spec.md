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
| **Crew** | Grupo de bots que podem conversar entre si. Trabalha numa pasta: a `shared/` dela ou uma que o dono escolhe. |
| **Bot** | Uma sessão Claude Code persistente com nome, papel e instruções. |
| **Chefe** | O bot que lidera a crew (`Crew.leadBotId`): nasce com ela, planeja o trabalho, distribui tarefas e é o único que sugere bots novos (10.2). |
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
   browser/    navegador de cada bot: Edge sem janela controlado por CDP (seção 21)
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
    src/             React (o app e, em src/setup/, a tela de instalação)
    src-tauri/       shell nativa
    src-setup/       tela de instalação: crate botloft-setup, com o instalador NSIS dentro (15.7)
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
| Tempo | `time`; `jiff` para os fusos das rotinas (seção 20) | `jiff` traz a base de fusos IANA embutida no Windows, que não tem uma; o cron das rotinas é avaliado pelo próprio daemon |
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
| Perfis do navegador dos bots | `%LOCALAPPDATA%\Botloft\browsers\<bot_id>\` (21.2) | |
| Pasta de trabalho da crew | `%USERPROFILE%\Botloft\<crew>\shared\` | uma pasta que o dono escolhe (`Crew.workFolder`) |
| Binário do daemon | `%LOCALAPPDATA%\Botloft\bin\botloftd.exe` (seção 14) | `--home` |
| App instalado | `%LOCALAPPDATA%\Botloft\` (`Botloft.exe`, o sidecar `botloftd.exe`, `uninstall.exe`; seção 15.4) | |

O instalador por usuário do Tauri põe o app em `%LOCALAPPDATA%\<produto>`, a mesma pasta dos dados. Os nomes não se cruzam, e o desinstalador só apaga os arquivos que instalou e remove a pasta se ela ficar vazia: os dados ficam.

`%LOCALAPPDATA%` e não `%APPDATA%`: o perfil roaming sincroniza em rede e não deve carregar SQLite nem segredos.

Workspaces ficam num caminho curto e visível para o usuário abrir no Explorer e para reduzir estouro do limite de 260 caracteres (bots rodam `npm install`).

**Pasta de trabalho da crew.** É onde os bots põem o que fazem para o dono e para os outros bots. Por padrão é a `shared\` da crew; o dono pode escolher outra ao criar a crew ou depois (`crews.setWorkFolder`), como uma pasta de projeto que ele já usa. O workspace de cada bot continua onde está, com a memória, as regras e os anexos dele: o daemon nunca grava nada na pasta escolhida. Cada bot recebe a pasta com `--add-dir` (7.4), e o Claude Code a trata como a pasta do bot: lê e, em "Aceitar edições", edita sem perguntar, e carrega o `CLAUDE.md` dela junto com a memória do bot. Só o da própria pasta (`CLAUDE.md`, `.claude\CLAUDE.md`, `.claude\rules\` e `CLAUDE.local.md`): numa pasta escolhida dentro de um projeto, os `CLAUDE.md` das pastas acima dela não são carregados pelo Claude Code (19). A pasta escolhida precisa ser um caminho completo e não pode ser um disco inteiro, ficar dentro da pasta de dados do Botloft ou contê-la, nem conter `workspaces_root` (com os workspaces de todos os bots). Se não existe, é criada. Trocar a pasta regrava as regras e reinicia cada bot da crew quando nada estiver em andamento, como a troca de modo (7.4). Arquivar a crew não apaga pasta nenhuma.

**Excluir não apaga pastas.** Excluir um bot ou uma crew (7.6) tira do Botloft e, a menos que o dono peça a Lixeira na confirmação (abaixo), deixa no disco o workspace de cada bot (memória, regras, anexos e o que ele fez), a `shared\` e a pasta que o dono escolheu: a confirmação do app diz que elas ficam e mostra o caminho (15.1), e o dono as apaga no Explorer se quiser. Do disco só sai o perfil do navegador do bot (21.2), que fica na pasta de dados e não tem nada que o dono tenha feito. Como a pasta fica, o slug dela continua ocupado, e um bot novo com o mesmo nome ganha sufixo (`revisor-2`). A conversa que o Claude Code guarda por conta própria (`%USERPROFILE%\.claude\projects\`) é dele e também fica. Se o dono marca a caixa da confirmação, a pasta que o Botloft criou vai para a Lixeira do Windows, de onde ainda dá para recuperar: o workspace do bot, ou a pasta inteira da crew (`<workspaces_root>\<crew>\`, com a `shared\` e os workspaces). A pasta de trabalho que o dono escolheu nunca é movida.

Nomes de pasta de crew e bot são slugs gerados na criação e **não mudam** quando o nome de exibição muda. Slug: ASCII minúsculo com hífens, acentos transliterados (`Revisão` -> `revisao`), no máximo 32 caracteres, nunca um nome de dispositivo do Windows (`con`, `lpt1`...) e nunca `shared` para bot. Colisão ganha sufixo (`docs-2`), inclusive com slug de item arquivado ou pasta que já exista no disco.

O **handle** do bot (`@revisao`) é derivado do nome pela mesma regra, acompanha renomeações e é único entre os bots ativos da crew; um nome que gere handle já usado é recusado com erro de validação.

### 5.1 Arquivos gerados em cada workspace

```
<workspace>\
  CLAUDE.md                        memória viva do bot (o bot edita; o daemon só cria se não existir)
  attachments\<aaaa-mm-dd>\        arquivos que o dono mandou no chat (9.5)
  .claude\
    settings.json                  regra de negação para os segredos e o que o Claude Code não carrega no bot, 7.5 (gerado pelo daemon, sobrescrito a cada start)
    rules\botloft.md               identidade, papel, crew, pasta de trabalho, guia de uso das tools, como explicar cada comando ao dono (10.1), que agendar é rotina (20), que arquivo pronto para o dono vai no chat com `share_file` (10) e, no chefe, como liderar (gerado pelo daemon)
  .botloft\
    mcp.json                       config MCP do bot (gerado pelo daemon)
```

Identidade e instruções vão em `.claude/rules/botloft.md` e não na linha de comando: o texto pode ser longo e a linha de comando do Windows é limitada. Regras sem frontmatter `paths` são carregadas no início de toda sessão, com a mesma prioridade de `.claude/CLAUDE.md` (confirmado na documentação oficial, ver seção 19). Plano B, se isso mudar: `--append-system-prompt` com texto curto apontando para o arquivo.

## 6. Configuração (`config.toml`)

```toml
port = 45710                  # só 127.0.0.1 no MVP
workspaces_root = ""          # vazio = %USERPROFILE%\Botloft
claude_path = ""              # vazio = resolver pelo PATH
start_with_windows = true     # o daemon sobe quando o dono entra no Windows (14)
keep_awake = true             # impede suspensão enquanto houver bot busy
log_level = "info"

[supervisor]
restart_backoff_initial_ms = 1000
restart_backoff_max_ms = 300000
fresh_start_if_dies_within_s = 15

[courier]
poll_interval_ms = 10000     # o maior intervalo entre ciclos do courier (9.1)
lease_ms = 15000
max_attempts = 8
retry_backoff_initial_ms = 2000
retry_backoff_max_ms = 120000

[bots]
approval_timeout_minutes = 60 # sem resposta do dono, a ferramenta é negada
attachment_max_mb = 20        # por arquivo; no máximo 10 arquivos por message
max_per_crew = 12             # até onde as sugestões do chefe levam uma crew (10.2)

[browser]                     # navegador dos bots (seção 21)
path = ""                     # vazio = Microsoft Edge do Windows
idle_minutes = 10
max_open = 4

[tasks]
max_hops = 4
default_deadline_minutes = 120
```

`start_with_windows`, `keep_awake` e `bots.approval_timeout_minutes` também mudam pelas Configurações do app (`settings.update`, 11.2): o daemon grava no próprio arquivo com `toml_edit`, só a chave que mudou, e as linhas e comentários do dono ficam (o comentário depois de um valor trocado também). O arquivo novo substitui o antigo inteiro (`config.toml.new` renomeado), para uma queda no meio não deixar metade. Valem na hora: `keep_awake` liga ou desliga o pedido de energia, `start_with_windows` muda os gatilhos da tarefa (14) e a espera das aprovações vale para o próximo pedido; uma espera maior chega ao `mcp.json` de cada bot, que reinicia retomando a conversa quando nada estiver em andamento (10.1, 7.4).

## 7. Ciclo de vida do bot

### 7.1 Estados

| Estado | Quando |
|---|---|
| `offline` | Sem processo e sem restart agendado (crew ou bot pausado) |
| `launching` | Processo criado, nos primeiros 1,5 s |
| `idle` | Processo vivo, sem turno em andamento e sem subagente trabalhando |
| `busy` | Turno em andamento ou na fila do Claude Code, ou subagente trabalhando em segundo plano |
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
| delivery escrita no stdin | `busy`; o `uuid` da linha fica esperando o replay |
| `user` com `isReplay` | o `uuid` escrito sai da espera e há um turno aberto. O replay pode vir no meio de um turno que já roda: uma message escrita entre duas chamadas de ferramenta entra nesse turno, e um só `result` fecha as duas (19). Um replay com `uuid` que ninguém escreveu (a saída do `/compact`, 8.6) só abre o turno |
| `system/init` | `busy`: há um turno aberto. Sem message por trás, é um turno do próprio Claude Code (um subagente terminou e ele seguiu sozinho) |
| `system/background_tasks_changed` | `busy` enquanto a lista trouxer tarefas `local_agent` (subagentes em segundo plano); a lista vem inteira a cada mudança. Comandos e monitores em segundo plano não contam: podem durar tanto quanto o bot |
| `result` | fecha o turno aberto; `idle` se nenhum `uuid` escrito espera o replay e não há subagente, senão continua `busy` |
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
- **Sessão:** o daemon guarda em `bots.session_id` o id da conversa que o Claude Code tem em disco. O primeiro start usa `--session-id <uuid novo>`. O Claude Code só grava a conversa quando o primeiro turno dela começa (19): o id é guardado quando a mensagem desse turno volta no stdout (`user` com `isReplay`, 8.1), não no `system/init`. Daí em diante os starts usam `--resume <session_id>`. Um processo que termina antes disso (um reinício pedido logo depois de criar o bot, por exemplo) não deixou conversa, e o próximo start usa `--session-id` com outro uuid novo.
- **Conversa nova:** o próximo start vem com um id novo se a sessão retomada morrer em menos de `fresh_start_if_dies_within_s`, se o Claude Code disser que não acha a conversa (um `result` com `subtype: "error_during_execution"` e `No conversation found` em `errors`, sem turno antes; o processo sai logo depois, 19) ou se o dono pedir `bots.restart {fresh: true}`. Esse `result` não é um turno: não vira item no chat nem fecha turno pendente. A conversa deixada para trás sai de `bots.session_id`, para o daemon não tentar retomá-la depois de reiniciar. O histórico do chat fica no banco do daemon (seção 8) e não depende do transcript do Claude Code.
- Mudanças em nome, papel ou instruções regravam as regras na hora, mas o bot só as lê no próximo start; o daemon não reinicia o bot sozinho.
- `auth_error` não entra em loop de restart: fica parado até o owner pedir `bots.restart` ou até o daemon ver o Claude Code conectado de novo.
- **Login do Claude Code:** o daemon roda `claude auth status` (JSON com `loggedIn` e, conectado, `email`, `subscriptionType` e `orgName`; sai com 0 conectado e 1 desconectado, seção 19) no ambiente do usuário, sem janela: quando acha o Claude Code, a cada 30 s enquanto desconectado ou desconhecido, na hora quando um bot cai em `auth_error` e quando o app pede (`system.refresh`). Conectado, não confere de novo sozinho. Quando o resultado passa de desconectado para conectado, os bots em `auth_error` sobem de novo sem o dono pedir. Um erro de conta com o login válido (`billing_error`, `account_on_hold`, organização bloqueada) não muda o resultado e por isso não vira loop. O resultado sai em `system.status.claudeSignedIn`, e a conta (e-mail, plano, organização) em `system.status.account.claude`. O login em si é feito pelo app (15.2).
- Cada processo de bot entra num **Job Object** com `KILL_ON_JOB_CLOSE`. Se o daemon morrer, a árvore de processos dos bots morre junto e não sobra `claude.exe` órfão. Fora do Windows, um grupo de processos (14.1).

### 7.4 Spawn

1. Resolver o binário: `claude_path` ou PATH, e depois os lugares dos instaladores, para quem só pôs a pasta no PATH do perfil do shell (no Windows: `%USERPROFILE%\.local\bin` do instalador nativo, `%LOCALAPPDATA%\Microsoft\WinGet\Links` e `%APPDATA%\npm`). Só o `claude.exe` nativo roda: passar por `cmd.exe` estraga o quoting dos argumentos. Por isso, ao achar o `claude.cmd` (ou `.ps1`) do npm, o daemon usa o executável nativo que o pacote do npm põe ao lado, em `node_modules\@anthropic-ai\claude-code\bin\claude.exe` (19). Se esse arquivo ainda é o script pequeno que o pós-instalação do npm troca pelo binário (menos de 4 KB), o launcher é recusado, a busca segue pelas entradas seguintes e o erro claro só aparece se nada servir. O PATH é separado só por `;`, como o Windows faz; aspas soltas numa entrada (comum em PATHs reais) não escondem as entradas seguintes.
2. `probe`: `claude --version` e exigir **>= 2.1.234**. Sem Claude utilizável, os bots ficam `offline`, `system.status.runtimeError` diz o motivo e o daemon tenta de novo a cada 30 s.
3. Gerar os arquivos da seção 5.1.
4. Comando, com cwd no workspace:

   ```
   claude -p --input-format stream-json --output-format stream-json --verbose
     --include-partial-messages --replay-user-messages
     (--session-id <uuid> | --resume <session_id>)
     --setting-sources project,local
     --mcp-config <workspace>\.botloft\mcp.json --strict-mcp-config
     --add-dir <pasta de trabalho da crew>
     --permission-mode <modo do bot>
     [--model <modelo do bot>]
     [--effort <esforço do bot>]
     --permission-prompt-tool mcp__botloft__permission_prompt
     --allowedTools mcp__botloft
     --disallowedTools CronCreate,CronDelete,CronList,ScheduleWakeup,RemoteTrigger
   ```

   - `--setting-sources project,local` e `--strict-mcp-config` deixam de fora hooks, skills, agents, modo de permissão e servidores MCP pessoais do dono: o bot vê o que o Botloft gera. O login da conta não é uma fonte de settings e continua valendo. As instruções pessoais do dono (`%USERPROFILE%\.claude\CLAUDE.md`) não saem por essa flag: o Claude Code as acha subindo as pastas a partir do cwd. Quem as deixa de fora, junto com a memória automática, é o `settings.json` gerado (7.5).
   - `--allowedTools mcp__botloft` libera as tools da crew sem aprovação. Uma regra `allow` no `settings.json` do projeto não bastaria: em `-p`, numa pasta que nunca passou pelo diálogo de confiança, o Claude Code não aplica as regras `allow` do projeto (documentado em `permissions`).
   - `--disallowedTools` tira do bot os agendamentos do próprio Claude Code. `CronCreate` e `ScheduleWakeup` só valem enquanto aquele processo vive: somem num reinício, expiram em 7 dias, e o Botloft não os vê nem os mostra. `RemoteTrigger` cria agendamentos na nuvem, fora do computador. Pedido para agendar, um bot usava `CronCreate` sem pedir permissão e dizia que tinha agendado (19). Trabalho em horário marcado é uma rotina (20).
   - Qualquer outra ferramenta que peça permissão passa pela tool de aprovação (10.1) e vira um pedido no chat.
   - `--add-dir` põe a pasta de trabalho da crew (5) entre as pastas do bot. Sem ela, gravar na `shared\`, que fica fora do workspace, pedia aprovação até em "Aceitar edições" (19).
   - `--permission-mode` vem do modo do bot (`Bot.permissionMode`), que o dono escolhe no chat (15.1): `default` (Manual: pergunta antes de editar, rodar comandos e usar a rede), `accept_edits` (`acceptEdits`: edita arquivos sem perguntar), `plan` (planeja e pede para seguir, 10.1), `auto` (um classificador libera o que é seguro e o resto vira pedido) e `bypass_permissions` (`bypassPermissions`: faz tudo sem perguntar, ver 13). Bot novo começa em `default`. `dontAsk` não é oferecido: nega o que não estiver liberado, e o bot não teria como pedir. Regras `deny` valem em todos os modos.
   - O Claude Code só lê a flag ao iniciar e não entra em `bypassPermissions` no meio da sessão. Por isso mudar o modo reinicia o processo com `--resume`, e a conversa continua: na hora se o bot está parado, ou quando o turno em andamento e as aprovações dele terminam, sem cortar o trabalho. Um bot que ainda não teve turno não tem conversa a retomar e reinicia com `--session-id` (7.3).
   - `--model` vem do modelo do bot (`Bot.model`), que o dono escolhe no chat ou ao criar o bot (15.1): `fable`, `opus`, `sonnet` ou `haiku`, os apelidos do Claude Code, que apontam sempre para a versão mais nova de cada família. `default` deixa a flag de fora, e o Claude Code usa o padrão da conta, que depende do plano (19). Bot novo começa em `default`. O Claude Code não troca de modelo sozinho conforme a tarefa. Mudar o modelo reinicia o processo como a troca de modo, e `--resume` com `--model` passa a usar o modelo novo. O `system/init` de cada turno traz o modelo em uso (`claude-opus-5-5`); o daemon o guarda em `Bot.modelInUse` para o app dizer qual é o padrão do plano. Um modelo que a conta não pode usar falha o turno com o erro `model_not_found` (8.1), e o processo continua vivo.
   - `--effort` vem do esforço do bot (`Bot.effort`), que o dono escolhe no chat (15.1): `low`, `medium`, `high`, `xhigh` ou `max`, os níveis do Claude Code, do mais rápido ao que mais pensa. Quanto mais alto, mais o bot pensa antes de responder e mais gasta do plano. `default` deixa a flag de fora, e o Claude Code usa o nível que ele mesmo define para o modelo (19); é o recomendado. Bot novo começa em `default`. A flag vale só para a sessão e não é guardada pelo Claude Code: o daemon a passa de novo a cada start. Mudar o esforço reinicia o processo como a troca de modo e de modelo, com `--resume`, quando nada estiver em andamento. O Claude Code também aceita a troca no meio da sessão (`/effort <nível>` como mensagem, ou um pedido de controle), mas o daemon reinicia, como faz com o modelo: a linha de comando continua sendo a única fonte do que o bot roda (19).
   - **O que a sessão aplica.** Assim que o processo sobe, o daemon pergunta ao Claude Code, por um pedido de controle (9.2), qual modelo e qual esforço a sessão aplica. A resposta traz o modelo antes do primeiro turno (`Bot.modelInUse`, que antes só vinha no `system/init`) e o nível de esforço. Num processo que subiu sem `--effort`, esse nível é o do próprio modelo: o daemon o guarda em `Bot.effortDefault` (`low` a `max`), e o app marca esse ponto como o recomendado. Um modelo que não tem níveis de esforço (Haiku 4.5) responde sem nível, mesmo com a flag: `effortDefault` vira `none`, e o app desliga o controle. Num processo com `--effort`, a resposta só repete a escolha do dono, e o que já se sabia do modelo fica. Trocar o modelo apaga `effortDefault` até o processo novo responder. Sem resposta (um Claude Code que não conheça o pedido), fica `null` e o app só não marca o recomendado.
   - O Claude Code sai do modo `plan` sozinho quando o dono aprova o plano. Cada turno começa com um `system/init` que traz `permissionMode` (19); se o processo começou em `plan`, o modo gravado ainda é `plan` e o init diz outro, o daemon grava o novo, sem reiniciar, para o app mostrar o que o bot faz e um reinício manter. Qualquer outra diferença vem de um processo que está para reiniciar no modo que o dono acabou de escolher, e é ignorada.
5. Ambiente: o bloco padrão do usuário (`CreateEnvironmentBlock`, o mesmo de um logon novo), **não** o ambiente do daemon. Um daemon iniciado de dentro de uma sessão do Claude Code herda `CLAUDECODE`, `CLAUDE_CODE_MESSAGING_SOCKET`, `ANTHROPIC_BASE_URL` e outras variáveis da sessão, que fariam o bot se achar filho dela. Por cima vão `BOTLOFT_BOT_ID`, `BOTLOFT_BOT_TOKEN`, `BOTLOFT_PORT` e `CLAUDE_CODE_ADDITIONAL_DIRECTORIES_CLAUDE_MD=1`, que faz o Claude Code carregar o `CLAUDE.md` da pasta de trabalho (5).
6. stdin, stdout e stderr em pipes, sem console (`CREATE_NO_WINDOW`). O stdin fica aberto enquanto o processo vive; fechá-lo encerra o Claude Code com código 0. O stderr vai para o log em nível `debug`, sem conteúdo de mensagem.
7. Os passos 3 a 6 (arquivos, bloco de ambiente, `CreateProcess`, Job Object) rodam **sem** a trava do supervisor: ela só decide quem sobe e depois registra o processo novo. Com o Defender, um start leva centenas de milissegundos, e com a trava presa o estado dos outros bots, as tools do MCP, o texto ao vivo e o courier esperavam todos os starts da passada. Um bot excluído enquanto sobe tem o processo morto ao ser registrado; um modo, modelo ou esforço trocado nesse meio-tempo reinicia o processo recém-criado.

### 7.5 `settings.json` gerado

```json
{
  "permissions": {
    "deny": ["Read(//c/Users/<usuário>/AppData/Local/Botloft/secrets/**)"]
  },
  "claudeMdExcludes": [
    "C:/Users/<usuário>/Botloft/CLAUDE.md",
    "C:/Users/<usuário>/Botloft/CLAUDE.local.md",
    "C:/Users/<usuário>/Botloft/AGENTS.md",
    "C:/Users/<usuário>/Botloft/.claude/CLAUDE.md",
    "C:/Users/<usuário>/Botloft/.claude/AGENTS.md",
    "C:/Users/<usuário>/Botloft/.claude/rules/**",
    "C:/Users/<usuário>/CLAUDE.md",
    "C:/Users/<usuário>/.claude/CLAUDE.md"
  ],
  "autoMemoryEnabled": false
}
```

(A lista de verdade repete os seis nomes para cada pasta acima da pasta da crew: `C:/Users/<usuário>/Botloft`, `C:/Users/<usuário>`, `C:/Users` e `C:/`.)

- Regras `deny` valem mesmo sem confiança na pasta: elas só restringem.
- Caminho absoluto em permission rule usa o prefixo `//` e a forma POSIX que o Claude Code aplica no Windows: `C:\Users\ana\...` vira `//c/Users/ana/...` (letra do drive em minúscula). Uma barra só (`/caminho`) é relativa à origem do settings, não à raiz, e não protegeria nada. O daemon converte `BOTLOFT_HOME` para essa forma ao gerar o arquivo.
- Em `-p` o diálogo de confiança da pasta nunca aparece (documentado), então bot novo começa a trabalhar sem passo manual.
- **Instruções de fora ficam de fora (`claudeMdExcludes`).** O Claude Code carrega `CLAUDE.md`, `CLAUDE.local.md`, `.claude/CLAUDE.md` e `.claude/rules/` do cwd e de toda pasta acima dele, como instruções de projeto, e `--setting-sources` não muda isso. O workspace fica dentro de `%USERPROFILE%` por padrão, então o `%USERPROFILE%\.claude\CLAUDE.md` do dono, que são as instruções pessoais dele para o Claude Code, chegava a todo bot; com `workspaces_root` dentro de um projeto (um daemon de dev), chegava o `CLAUDE.md` do projeto. O daemon lista, para cada pasta acima da pasta da crew, de `workspaces_root` até a raiz do disco, os seis nomes que o Claude Code procura: `CLAUDE.md`, `CLAUDE.local.md`, `AGENTS.md`, `.claude/CLAUDE.md`, `.claude/AGENTS.md` e `.claude/rules/**`. São caminhos exatos, sem `**` no começo: nenhum padrão alcança outra pasta.
- **O que continua carregando:** o `CLAUDE.md` e as regras do bot (5.1); o `CLAUDE.md` e as regras da pasta de trabalho da crew, a `shared\` ou a que o dono escolheu, que chegam pelo `--add-dir` (7.4); e um `CLAUDE.md` que o dono ponha na pasta da crew (`<crew>\CLAUDE.md`), que vale para todos os bots dela. O `CLAUDE.md` de política gerenciada (`C:\Program Files\ClaudeCode\CLAUDE.md`) não pode ser excluído (documentado) e continua valendo.
- **Forma do caminho em `claudeMdExcludes`:** o padrão é comparado com o caminho absoluto como o Claude Code o monta a partir do cwd: `C:/Users/ana/...` (ou com `\`), com a letra do drive e as maiúsculas iguais às do cwd, porque a comparação diferencia maiúsculas. A forma `//c/...` das permission rules **não** casa aqui. O daemon monta os padrões com o mesmo caminho que passa como cwd. Colchetes, chaves e parênteses num nome de pasta são sintaxe de glob e viram `?` (um caractere qualquer); espaço, acento e o resto casam como estão (19).
- **Sem memória automática (`autoMemoryEnabled: false`).** O Claude Code tem uma memória própria, ligada por padrão: grava notas sozinho em `%USERPROFILE%\.claude\projects\<projeto>\memory\` e carrega o índice delas em toda sessão. Nos bots ela fica desligada, porque a memória do bot é o `CLAUDE.md` do workspace (5.1), que o dono vê, edita e apaga junto com o bot. A memória automática fica fora do workspace, no perfil do dono, é gravada sem pedido de permissão mesmo no modo Manual, não aparece no app e sobra quando o bot é apagado; a pasta é escolhida pelo caminho do workspace ou, dentro de um repositório git, pelo repositório, e aí o bot carregava as notas que o Claude Code guardou para o próprio dono naquele repositório (19). Desligada, o prompt de sistema de cada bot também fica uns 3 mil tokens menor. O daemon não apaga nada em `%USERPROFILE%\.claude`: as notas que um bot gravou antes ficam onde estão e só deixam de ser carregadas.

### 7.6 Arquivar e excluir

**Arquivar** (`bots.archive`, `crews.archive`) tira o bot ou a crew do app e guarda tudo no banco: o processo para, o token é revogado, as rotinas são arquivadas junto (20.3) e o perfil do navegador é apagado (21.2).

**Excluir** (`bots.delete`, `crews.delete`) apaga do banco, de vez, e vale para ativos e arquivados. Não tem volta, e nenhuma pasta é apagada (5).

- **O bot para na hora.** O processo é morto sem esperar o turno, o token deixa de valer, o navegador fecha e o perfil dele é apagado, e os pedidos de permissão abertos caem. O supervisor esquece o bot: o que o processo ainda imprimir é ignorado, e uma reconciliação que tinha lido o bot antes não o sobe de novo.
- **Sai do banco** (12), numa transação: o bot, o chat dele, as aprovações, as messages que ele **recebeu** (com deliveries e o registro dos anexos), as rotinas com as execuções, os sites liberados no navegador e as tasks que ele pediu ou recebeu. Se era o chefe, a crew fica sem chefe (10.2).
- **Fica com os outros bots:** as messages que o excluído **mandou** continuam no chat de quem recebeu e na timeline da crew, sem remetente (`fromBotId: null`) e sem task. Uma que ainda esperava na fila chega com `from a deleted bot` e sem instrução de resposta (9.3).
- **Tasks em aberto** (`open` ou `expired`) somem com o bot, e o outro lado é avisado pelo daemon (`from Botloft`), se ainda está ativo: quem pediu uma task ao excluído lê que ela não vai ser feita; quem fazia uma task para o excluído lê que ninguém mais espera o resultado. Um `complete_task` dela responde que a task não existe.
- **Excluir uma crew** exclui todos os bots dela, ativos e arquivados, com tudo acima, e a crew. Ninguém é avisado: não sobra ninguém.
- **Avisos ao app:** `bot.deleted {botId, crewId}` e `crew.deleted {crewId}` (11.3). O app tira do store o bot (ou a crew com os bots) e o que era dele: rotinas, tasks, deliveries e o navegador. As outras coisas que mudam saem pelos avisos de sempre (`crew.changed` sem chefe, `message.created` dos avisos de task).
- **No app** (15.1): Excluir fica no menu do bot e no menu da crew, depois de Arquivar, com uma confirmação que diz o que sai e o que fica. Os arquivados, que o resto do app não mostra, ficam em Configurações > Arquivados, cada um com o seu Excluir; a lista vem de `archive.list` (11.2).
- **Lixeira, se o dono pedir:** `recycleFolder: true` em `bots.delete` manda o workspace do bot para a Lixeira do Windows; em `crews.delete`, a pasta da crew inteira (`<workspaces_root>\<crew>\`: a `shared\` e o workspace de cada bot, também dos arquivados e dos excluídos antes). Só vai uma pasta que esteja dentro de `workspaces_root`, nunca a própria raiz, e nunca uma que contenha a pasta de trabalho escolhida pelo dono (5): nesse caso nada é movido. O método responde na hora, como sem a opção; a pasta vai depois, fora dos workers do servidor, porque o processo recém-morto ainda segura arquivos por um instante: até 20 tentativas, uma a cada 500 ms. Quando termina, o daemon avisa com `folder.recycled {path, error}`: `error: null` se a pasta está na Lixeira (ou já não existia), senão o motivo de ela ter ficado. Nada é apagado de vez: o que a Lixeira não pode receber fica onde está, sem nova tentativa (14).

## 8. Chat

O chat de um bot é a sequência de itens que o daemon monta a partir do que entra no stdin e do que sai no stdout. Ele fica no banco (`chat_items`), é paginado pelo app e é a única visão da conversa: não há terminal.

### 8.1 Leitura do stdout

Uma linha JSON por evento, em UTF-8. Linha que não é JSON, ou maior que 8 MiB, é descartada com um aviso em `debug`. Tipos desconhecidos são ignorados (o protocolo cresce entre versões do Claude Code). O que o daemon usa:

| Evento | O que vira |
|---|---|
| `system/init` | vem no começo de cada turno: segue `permissionMode` e guarda `model` (7.4); um turno sem message por trás deixa o bot `busy` (7.2) |
| `system/background_tasks_changed` | quantos subagentes (`task_type: local_agent`) rodam em segundo plano (7.2) |
| `user` com `isReplay: true` | a message com aquele `uuid` começou a ser processada: a delivery ganha `read_at` (9.1) e o `uuid` sai da espera do supervisor (7.2). A conversa existe em disco a partir daqui: guarda `session_id` em `bots.session_id` (7.3) |
| `result` com `No conversation found` em `errors` | o Claude Code não achou a conversa a retomar e vai sair: não é um turno, não vira item nem fecha turno pendente; o próximo start começa conversa nova (7.3) |
| `stream_event` com `text_delta` | texto ao vivo da resposta (`chat.delta`, 8.3); não é gravado |
| `assistant`, bloco `text` | item `reply` com o texto (markdown) |
| `assistant`, bloco `tool_use` | item `tool` em `running`, com resumo da entrada e, num comando, a explicação do bot (10.1) |
| `user`, bloco `tool_result` | atualiza o item `tool` do mesmo `tool_use_id`: `done` ou `failed` e um trecho da saída |
| `assistant` com `error` | item `notice` e, conforme o erro, estado `rate_limited` ou `auth_error` (7.2); `model_not_found` vira o aviso `model_unavailable` |
| `rate_limit_event` | uso da conta (janelas de 5 h e 7 dias) em `system.status.usage`; status diferente de `allowed` leva a `rate_limited` |
| `result` | item `turn` com duração, tokens (8.7) e erro (se houve); fecha o turno aberto, com todas as messages que entraram nele (7.2). O daemon pergunta de novo o tamanho da conversa (8.6). O `result` de uma compactação (`local_command: "compact"`) fecha o turno sem item `turn` |
| `assistant` com `usage` | quanto a conversa ocupa naquele pedido ao modelo (8.6) |
| `system/status` | `status: "compacting"`: o Claude Code começou a compactar a conversa; com `compact_result`, terminou (8.6) |
| `system/compact_boundary` | a conversa foi compactada: aviso `compacted` ou `auto_compacted` no chat (8.6) |
| `control_response` | resposta a um pedido de controle do daemon (9.2): o que a sessão aplica (7.4) ou o tamanho da conversa (8.6) |

Blocos `thinking` não são gravados nem mostrados.

### 8.2 Itens

Todo item tem `id` (`cht_`), `botId`, `kind`, `createdAt` e `updatedAt`.

| `kind` | Campos | Origem |
|---|---|---|
| `inbound` | `message` (a `Message`, com anexos) | criado junto com a message para o bot: dono, outro bot ou daemon |
| `reply` | `text` | texto do bot |
| `tool` | `toolUseId`, `name`, `summary`, `explanation`, `input`, `status` (`running`, `done`, `failed`), `output`, `file` | ferramenta usada pelo bot; `explanation` é o que o bot diz que um comando faz (10.1; ausente nas outras ferramentas, quando o bot não disse nada e nos itens antigos); `file` é o caminho completo que uma chamada de `Write`, `Edit`, `MultiEdit` ou `NotebookEdit` altera (ausente nas outras e nos itens antigos) |
| `approval` | `approvalId`, `toolName`, `summary`, `explanation`, `input`, `status` (`pending`, `allowed`, `denied`, `expired`), `note` | pedido de permissão (10.1); `explanation` como no item `tool`, guardada só no item do chat |
| `question` | `question` (a `Question`, 23.3) | pergunta do bot ao dono (23.4), regravada quando ele responde ou descarta |
| `turn` | `durationMs`, `tokens` (8.7; `null` quando o Claude Code não disse e nos itens antigos), `error` | fim de um turno |
| `notice` | `level` (`info`, `warning`, `error`), `code` (`signed_out`, `usage_limit`, `turn_failed`, `model_unavailable`, `compacted`, `auto_compacted`, `compact_failed`; ausente em avisos antigos), `text` | avisos do daemon: limite de uso, login, turno com erro, conversa compactada (8.6). O app escreve os avisos com `code` no idioma do dono; `text` fica em inglês para quem não conhece o código e, em `turn_failed` e `compact_failed`, traz o detalhe do erro |

- `summary` é um trecho curto tirado da entrada, sem palavras do daemon: o comando do `Bash` e do `PowerShell` (o comando mesmo, nunca o que o bot diz dele: isso é a `explanation`), o arquivo do `Read`/`Edit`/`Write`, o padrão do `Grep`/`Glob`, a URL do `WebFetch`, a busca do `WebSearch`, o destinatário do `send_message` (`@writer`), a tecla do `browser_press`. Fica vazio onde só palavras diriam algo (`TodoWrite`, `complete_task`, `ToolSearch` que carrega ferramentas, a direção do `browser_scroll`) e onde o trecho não diz nada ao dono (a `ref` de um elemento do navegador). O app escreve o nome da ferramenta como uma ação no idioma do dono ("Mandar uma mensagem", "Abrir uma página"; uma ferramenta que ele não conhece aparece pelo nome) e, nas que ficam vazias, o que der para dizer a partir da entrada (as ferramentas carregadas, a direção da rolagem).
- `input` guarda o JSON da entrada até 4 KB (a de um comando, até 32 KB, para o dono ler inteiro o que permite, 10.1); `output`, até 8 KB de texto. O resto fica só no transcript do próprio Claude Code.
- A resposta do bot (`reply`) é guardada inteira.
- Uma message de um bot para outro aparece duas vezes: no chat de quem mandou, como o item `tool` do `send_message`; no chat de quem recebe, como `inbound`.

### 8.3 Ao vivo

- `chat.item {item, activity}`: item novo ou atualizado (tool que terminou, aprovação respondida). `activity` é a nova linha da conversa na barra lateral, quando mudou.
- `chat.delta {botId, text}`: pedaço do texto que o bot está escrevendo, na ordem. O daemon junta os pedaços que chegam em até 40 ms (ou até 4 KB) num só `chat.delta`, e manda o que juntou antes de qualquer outro evento do mesmo bot: um evento por token custava a cada app uma renderização. O app junta os pedaços num balão provisório, trocado pelo `reply` quando ele chega. Um app que conecta no meio de um turno não vê o texto parcial já passado, só o que vier depois e o `reply` final.

### 8.4 Arquivos do bot

O app mostra os arquivos que o bot fez num painel ao lado do chat (15.1). O daemon os acha de duas maneiras, porque um script ou comando (um PDF gerado por um `PowerShell`, por exemplo) não diz o que criou:

- **Pastas:** a pasta de trabalho da crew e a pasta do bot (5), percorridas com limites: até 6 níveis, 20 mil entradas vistas, sem links, sem entradas ocultas (`.`, `~$`) nem pastas geradas (`node_modules`, `target`, `__pycache__`, `venv`, `site-packages`). Na pasta do bot ficam de fora `CLAUDE.md` e `attachments\` (o que o daemon escreve e o que o dono mandou). Só entra arquivo alterado **depois de o bot ser criado**: o que já existia na pasta escolhida pelo dono não é do bot.
- **Chamadas do bot:** os caminhos dos itens `tool` com `file` (8.2), mesmo fora dessas pastas, se o arquivo ainda existe. Esses vêm com `writtenByBot`.

`files.list {botId}` devolve até 200 `BotFile` (`path`, `name`, `folder` relativa à pasta de trabalho ou à do bot, `mediaType` pela extensão, `size`, `modifiedAt`, `writtenByBot`), do mais novo para o mais antigo. `files.read {botId, path}` devolve `{mediaType, data}` (base64) de um arquivo que a lista poderia mostrar: dentro de uma das duas pastas ou escrito pelo bot. Qualquer outro caminho, relativo ou que suba de pasta dá `not_found`, para o painel não ler o resto do computador. Acima de 20 MiB dá `validation`. Nenhum caminho nem conteúdo vai para o log em `info` ou acima (8.5).

**No chat:** o bot mostra ao dono um arquivo que fez, ou que o dono pediu de novo, com a tool `share_file` (10). Só vale arquivo que `files.read` leria (dentro de uma das duas pastas ou escrito pelo bot), para o cartão poder mostrar e salvar o que aponta; um caminho relativo é da pasta do bot, onde ele roda. Não há item novo: o item `tool` da chamada guarda na `output` a resposta da tool, com um `BotFile` por arquivo em `shown`, e o app desenha cartões a partir dela (15.1). Um arquivo de antes do bot, que a lista deixa de fora, continua compartilhável: o cartão leva o `BotFile` ao painel.

### 8.5 Privacidade

Itens do chat são dado pessoal como o corpo das messages: nunca vão para o log em nível `info` ou acima. O log de `debug` registra só tipos de evento e ids.

### 8.6 Contexto da conversa

A conversa de um bot tem um tamanho máximo, a janela de contexto do modelo (1 milhão de tokens em Fable, Opus e Sonnet; 200 mil em Haiku). Tudo o que o bot leu e escreveu conta, e cada pedido ao modelo manda a conversa inteira: uma conversa cheia gasta mais do plano a cada turno. Perto do fim da janela, o Claude Code **compacta** a conversa sozinho: troca o que veio antes por um resumo e segue. O dono vê quanto da janela está em uso e pode compactar antes.

**Quem conta é o Claude Code.** O daemon pergunta, por um pedido de controle (9.2, `get_context_usage`), e guarda a resposta em memória, por bot:

| Campo de `ContextUsage` | De onde vem |
|---|---|
| `usedTokens` | `totalTokens`: o que a conversa ocupa agora, com as instruções e as ferramentas do Claude Code (uns 20 mil tokens numa conversa vazia) |
| `windowTokens` | `maxTokens`: o máximo que cabe |
| `autoCompactTokens` | `autoCompactThreshold`: onde o Claude Code compacta sozinho (967 mil numa janela de 1 milhão, 167 mil numa de 200 mil); `null` se `isAutoCompactEnabled` vier `false` |
| `compacting` | há uma compactação em andamento, ou pedida e esperando o turno atual terminar |
| `updatedAt` | quando o daemon soube |

- **Quando pergunta:** assim que o processo sobe (uma conversa retomada já responde com o tamanho dela, antes de qualquer turno), a cada `result` e depois de uma compactação. A resposta chega em um ou dois segundos, também no meio de um turno, e não abre turno nem entra na conversa.
- **Durante o turno:** cada `assistant` da conversa principal traz o `usage` do pedido ao modelo; `input_tokens + cache_creation_input_tokens + cache_read_input_tokens` é o que a conversa ocupava naquele pedido, e o daemon atualiza `usedTokens` com isso. Só vale com a janela já conhecida por uma resposta. Subagentes (`parent_tool_use_id`) e respostas de comando (`model: "<synthetic>"`) não contam.
- **Nada é gravado.** Um daemon novo pergunta de novo quando os bots sobem. Um bot parado mostra o último valor que o daemon viu, ou nada. Conversa nova (`bots.restart {fresh: true}`) apaga o valor até o processo responder.
- Um Claude Code que não conheça o pedido responde com erro, e `Bot.context` fica `null`: o app não mostra o contexto.
- O app recebe `Bot.context` na lista e `bot.context {botId, context}` a cada mudança (11.3).

**Compactar agora.** `bots.compact {botId}` escreve `/compact` no stdin do processo, como uma mensagem (9.2). O Claude Code trata a linha como o comando e compacta; nada vai para o modelo como pedido do dono. Como toda mensagem, ela espera o turno em andamento terminar e conta como um turno: o bot fica `busy` até o `result`. O bot precisa estar `idle`, `busy` ou `needs_approval`; senão, `conflict`. Com uma compactação já a caminho, o pedido não escreve de novo.

O que o Claude Code imprime (19), e o que o daemon faz:

1. `system/status` com `status: "compacting"`: `compacting` passa a `true` (também quando é o próprio Claude Code que decide compactar, no meio de um turno).
2. `system/status` com `compact_result` e `system/compact_boundary` (`compact_metadata.trigger`: `manual` ou `auto`): `compacting` volta a `false`, o chat ganha um aviso `info` (`compacted` para `manual`, `auto_compacted` para `auto`) e o daemon pergunta o novo tamanho.
3. Um `user` com `isSynthetic` traz o resumo. Não é resposta do bot nem vai para o chat.
4. No `/compact`, um `result` com `local_command: "compact"` fecha o turno. Não vira item `turn`: o aviso já diz o que houve. Na compactação automática não há `result` próprio; o turno segue.
5. Se não havia o que compactar, em vez de 1 a 3 vem um `assistant` com `local_command_run.command: "compact"` e `local_command_outcome.kind: "failed"`. Vira um aviso `warning` com `compact_failed` e o motivo, não uma resposta.
6. Se o processo morrer no meio, `compacting` volta a `false`.

Os três avisos não mudam a linha da conversa na barra lateral (`lastActivity`): compactar é manutenção, não novidade da conversa.

A compactação custa um pedido ao modelo com a conversa inteira (barato com o cache quente, 19) e depois cada turno manda só o resumo. O dono também pode escrever `/compact` no chat: a linha segue como qualquer mensagem dele (9.3), e o efeito é o mesmo.

### 8.7 Tokens

O dono vê quantos tokens cada bot gasta: em cada turno, no chat, e somados por bot, no diálogo de uso. Nunca em dinheiro: o `total_cost_usd` do Claude Code é preço de API e soma o processo inteiro (19), e quem usa um plano do Claude não paga isso.

- **De onde vem.** O `usage` do `result` é a soma dos pedidos ao modelo daquele turno (19). O item `turn` guarda `tokens`: `input` (`input_tokens`), `cacheWrite` (`cache_creation_input_tokens`), `cacheRead` (`cache_read_input_tokens`) e `output` (`output_tokens`, com o pensamento), e ainda `reloaded`, a conta do daemon descrita abaixo (0 nos itens antigos). O `modelUsage` não serve: ele acumula entre turnos.
- **O que conta.** O número em destaque é `input + cacheWrite + output`: o que o modelo leu sem vir do cache (o texto novo e a conversa recarregada, abaixo) e o que escreveu. O Claude Code manda quase tudo que é novo (a mensagem, o resultado de uma ferramenta) para o cache, por isso o `input` sozinho fica perto de zero e o novo está no `cacheWrite`. O `cacheRead` é quase todo a conversa relida a cada pedido, pesa bem menos e numa conversa longa seria a maior parte do total. O app não o mostra: um número "relidos" ao lado do gasto confundia quem lê. Ele continua gravado no item `turn` e somado no `usage.tokens`.
- **A conversa recarregada.** Cada pedido manda a conversa inteira. Quando o cache expirou, porque o bot ficou parado ou o processo reiniciou, a parte da conversa que o cache não tinha mais é gravada de novo e vem no `cacheWrite`, misturada com o que é novo; num turno curto depois de uma pausa ela é quase tudo (19). O daemon separa essa parte em `reloaded`, sem guardar nada: para cada pedido da conversa principal (sem subagentes nem respostas sintéticas, contado uma vez por `message.id`), o que a conversa já tinha e não veio do cache, `min(cacheWrite, antes - cacheRead)`, onde `antes` é `input + cacheWrite + cacheRead` do pedido anterior ou, sem pedido visto, o tamanho que o Claude Code respondeu (8.6). Uma conversa nova ou recém-compactada não tem `antes` e não conta nada. O valor é limitado ao `cacheWrite` do `result`. O app não mostra essa parte: ela entra no número do turno, porque é gasto de verdade do plano, e um número a mais ao lado confundia quem lê. Um turno curto depois de uma pausa mostra um número grande, e isso é verdade. O `reloaded` continua gravado no item `turn`.
- **No chat.** A linha do fim do turno diz a duração e os tokens ("Concluído em 4,2 s · 2,5 mil tokens"); o detalhe (lidos e escritos) fica no título da linha. A unidade é o turno, não a mensagem: mensagens que chegam juntas entram num turno só, e não dá para dividir os tokens entre elas.
- **Por bot.** `usage.tokens {since}` soma os itens `turn` com `tokens` terminados a partir de `since` (ms Unix), por bot, com nome, cor, o nome da equipe (dois bots podem ter o mesmo nome em equipes diferentes) e se está arquivado, do que mais gastou para o que menos. Bots sem turno no período ficam de fora; bots excluídos levam os itens junto (7.6). O app oferece hoje (desde a meia-noite local), 7 dias, 30 dias e tudo, com o total dos bots embaixo.
- Uma compactação (8.6) não vira item `turn` e não entra na conta.

### 8.8 Busca

O dono procura palavras em todas as conversas: o que ele mandou, o que outros bots e o Botloft mandaram, o que cada bot respondeu, e as perguntas dos bots com as respostas (23). Ferramentas, pedidos e avisos ficam de fora: são o trabalho, não a conversa.

- **Índice.** Uma tabela FTS5 (`chat_search`) sobre `chat_items`, lida por uma view (`chat_text`) que tira o texto do JSON de cada item: nada é gravado duas vezes. Triggers acompanham cada item novo, mudado ou apagado (a exclusão de um bot, 7.6, leva o índice junto); a migration indexa o que já existia. O tokenizador é `unicode61` sem acentos: "previsao" acha "previsão", e maiúsculas não contam.
- **Consulta.** `chat.search {query, botId?, crewId?, before?, limit?}` acha os itens com **todas** as palavras, em qualquer ordem; a última vale também como começo de palavra ("relat" acha "relatório"). Cada palavra é tomada ao pé da letra: aspas, `OR`, `NEAR` e `*` do FTS5 não fazem nada de especial. Precisa de pelo menos 2 letras ou números; menos que isso é `validation`. Só bots e crews ativos, do item mais novo ao mais velho; `before` pagina, `limit` de 1 a 100 (30 se ausente).
- **Resultado.** `SearchHit`: o `ChatItem`, o `crewId` e um trecho (`snippet`) de umas 18 palavras em volta do que foi achado, com cada palavra achada entre `\u0002` e `\u0003` (`SNIPPET_MARKS` no TypeScript), que o app destaca.
- **Abrir no ponto.** `chat.history {botId, until}` devolve do item achado até o mais novo, até 1 000 itens (o chat abre ali e o resto carrega como sempre). Um item que não é do bot dá `not_found`.
- **Por fora da fila.** `chat.search` responde quando fica pronta, como `files.list` (11.1): uma busca lenta não segura o texto ao vivo.
- **Privacidade.** A consulta é texto do dono e nunca vai para o log em `info` ou acima (8.5).

**No app.** "Buscar", no topo da barra lateral (ou Ctrl+K), abre a busca no meio da janela: um campo, a escolha de uma equipe ou de todas, e os resultados conforme o dono escreve, cada um com o rosto e o nome do bot, a equipe, a hora, quem escreveu (o dono, outro bot, o Botloft ou o próprio bot) e o trecho com as palavras em destaque. "Mostrar mais" traz os anteriores. Um clique abre o chat do bot nesse ponto, com o item em destaque por um instante. A busca guarda o que foi digitado enquanto o app está aberto, e o texto volta selecionado, pronto para uma busca nova. Sem nada achado: "Nada encontrado".

## 9. Mensagens e entrega

### 9.1 Fluxo

1. `messages.send` (owner) ou tool `send_message` (bot) grava `message`, `delivery` (`pending`) e o item `inbound` no chat do destinatário, numa transação. Anexos são gravados antes (9.5).
2. O **courier** dorme até a próxima coisa com hora marcada: a primeira delivery da fila de algum bot vencer, um lease acabar ou uma task aberta passar do prazo; no mínimo 250 ms e no máximo `poll_interval_ms`. Acorda na hora, via notify, quando entra delivery nova ou termina um envio, e quando um bot passa a aceitar mensagens (`bot.state` em `idle`, `busy` ou `needs_approval`). Antes, acordava a cada 500 ms, parado ou não.
3. Entrega em ordem, uma por vez por bot: de cada bot, só a delivery `pending` mais antiga pode sair, quando `next_attempt_at <= agora` e o bot não tem outra em `sending`. Ela vira `sending` com `lease_until`.
4. Se o bot não está em `idle`/`busy`/`needs_approval`: volta para `pending` com `next_attempt_at` em 5 s, **sem** contar tentativa. Quando o bot passa a aceitar mensagens, as deliveries dele que nunca falharam (`attempts = 0`) vencem na hora, sem esperar esses 5 s; as que falharam mantêm o backoff, para uma mensagem que derruba o bot não voltar em seguida. Bot ou crew arquivados: a delivery vira `dead` na hora.
5. Monta a mensagem (9.2) na hora do envio e escreve uma linha no stdin do processo, com timeout de 10 s.
6. Escrita aceita: `sent`, com a generation do processo. Falha de escrita (processo saindo): volta para `pending` sem contar tentativa.
7. Quando o Claude Code pega a mensagem, num turno novo ou no turno que já roda, ele a devolve no stdout com o mesmo `uuid` (`--replay-user-messages`), e a delivery ganha `read_at`. O app mostra isso como "lida", com os dois tiques em azul-claro (15.3).
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
- Um texto que começa com `/` e o nome de um comando do Claude Code é executado como comando, não enviado ao modelo. O daemon usa isso para compactar (`/compact`, 8.6).

**Pedidos de controle.** Pelo mesmo stdin, o daemon pergunta ao próprio Claude Code sobre a sessão, fora da conversa:

```json
{"type":"control_request","request_id":"botloft-context-7","request":{"subtype":"get_context_usage"}}
```

A resposta sai no stdout como `{"type":"control_response","response":{"subtype":"success","request_id":"...","response":{...}}}`, ou com `subtype: "error"` para um pedido desconhecido. Não abre turno, não muda o estado do bot, não fica no transcript e é respondido também durante um turno. É o protocolo que o Agent SDK usa por baixo: `get_context_usage` é o método `getContextUsage()` (documentado, com os campos da resposta); `get_settings` e o formato no fio não são documentados e foram vistos com teste real (19). O daemon só **pergunta** por esse caminho, e o que ele não conseguir saber fica vazio no app, sem mudar o que o bot faz: `get_settings` (7.4) e `get_context_usage` (8.6). O `request_id` começa pelo que foi perguntado e termina num contador, e é por ele que o daemon sabe de que é a resposta.

### 9.3 Texto que o bot recebe

A mensagem do **dono** vai como ele escreveu, sem envelope: é o usuário da sessão falando, com a autoridade de quem digita. A resposta a uma pergunta do bot vem depois de uma linha que cita a pergunta (23.4).

Mensagens de **outros bots** e **avisos do daemon** levam um envelope em inglês, como as regras geradas (5.1):

```
[botloft] from @revisor · crew Exemplo · task tsk_01J9Z... · due in 2 h
Reply with send_message(to: "revisor"). When the task is done, call complete_task(task_id: "tsk_01J9Z...").

<corpo da mensagem>
```

- Nota de outro bot: primeira linha `[botloft] from @revisor · crew Exemplo` e só a instrução de resposta.
- Resultado de task: `· result of task tsk_... · done` (ou `failed`) e a instrução de resposta.
- Aviso do daemon (task vencida, task de um bot excluído): `from Botloft` e o texto do aviso, sem instrução.
- Message de um bot que o dono excluiu enquanto ela esperava na fila (7.6): `from a deleted bot`, sem instrução de resposta, porque não há a quem responder.
- Rotina: `routine "<nome>" · scheduled <data e hora> (<fuso>)` e o aviso de que ninguém está olhando (20.5).
- O prazo é relativo (`due in 45 min`, `due in 2 h`, `overdue`) e calculado na hora do envio: o bot não sabe a hora atual, e o app mostra o horário absoluto a partir de `deadline_at`.

### 9.4 Tasks entre bots

- `send_message` com `kind: "task"` cria uma `task` ligada à message, na mesma transação.
- Campos: `requester_bot_id`, `assignee_bot_id`, `status` (`open`, `done`, `failed`, `cancelled`, `expired`), `deadline_at`, `hops`, `origin` (id da task que originou a cadeia).
- Prazo: `deadline_minutes` de 1 a 10 080 (uma semana); sem ele, `default_deadline_minutes`.
- Cadeia: a task que o bot está fazendo é a task `open` atribuída a ele com mais `hops` (a mais nova, no empate). Uma task criada por ele herda `origin` dessa task (ou o id dela, se ela for a primeira) e `hops + 1`. Uma task sem cadeia tem `hops = 1`. Acima de `max_hops`, a tool recusa com um erro que diz o passo, o limite e onde a cadeia começou. Notas não contam hops.
- `complete_task` só vale para quem recebeu a task, com status `open` ou `expired`. Grava o resultado e, na mesma transação, cria a message `result` para o solicitante.
- Task vencida vira `expired` no ciclo do courier, e o solicitante recebe um aviso do daemon (`from Botloft`). Ela continua aceitando resultado atrasado.
- `cancelled` fica reservado: nenhuma tool cancela task no MVP.
- Excluir um bot apaga as tasks que ele pediu ou recebeu e avisa o outro lado das que estavam em aberto (7.6).

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
| `crew_roster` | nenhuma | `crew`, `you`, `you_lead`, os outros bots da crew (handle, nome, papel, estado e `chief`) e `signals`, os avisos que as rotinas da crew esperam (20.13) |
| `send_message` | `to` (handle, com ou sem `@`), `body`, `kind?` (`note` padrão, `task`), `deadline_minutes?` (só task) | `message_id`, `task_id` e `due` (task), e um lembrete de que a resposta chega depois |
| `complete_task` | `task_id`, `result`, `status?` (`done` padrão, `failed`) | `task_id`, `status` e quem recebe o resultado |
| `my_tasks` | `role?` (`assigned`, `requested`) | tasks `open` e `expired`: id, de, para, status, prazo relativo, hops e o pedido original |
| `suggest_bot` | `name`, `role`, `instructions` (até 8 000 caracteres), `model?`, `reason` (até 1 000) | `created` e, criado, `handle`, `name`, `role`, `model` e um lembrete para mandar a primeira task; recusado ou sem resposta, o porquê (10.2). Só o chefe |
| `schedule_routine` | `name`, `prompt` (até 8 000 caracteres), `schedule` (como em 20.2), `bot?` (handle de outro bot da crew) | `created` e, criada, `routine_id`, nome, pedido, horário, fuso e a próxima vez; recusada ou sem resposta, o aviso de que nada foi agendado (20.12) |
| `share_file` | `files` (1 a 10 caminhos, absolutos ou relativos à pasta do bot) | `shown` (um `BotFile` por arquivo, 8.4) e um lembrete de que o dono vê cada um como cartão no chat; um arquivo fora das pastas do bot e não escrito por ele, ou que não existe, recusa a chamada inteira e diz qual e por quê |
| `send_signal` | `name`, `note?` (até 2 000 caracteres) | o aviso normalizado e, para cada rotina da crew que o espera, se rodou e por que não (20.13) |
| `ask_owner` | `question` (até 2 000 caracteres), `options?` (2 a 5, até 100 caracteres cada) | `question_id` e o lembrete de que a resposta chega depois, como mensagem; a tool não espera (23.2) |
| `permission_prompt` | `tool_name`, `input`, `tool_use_id` | decisão do dono (10.1). Chamada pelo Claude Code, não pelo modelo |
| `browser_*` | seção 21.4 | o navegador do bot: abrir, ler, clicar, digitar, rolar, ver a tela, pedir a mão do dono |

- Erro que o modelo pode corrigir (handle desconhecido, argumento inválido, limite de hops, task de outro bot) volta como resultado com `isError: true` e uma frase explicando. Só tool desconhecida ou chamada malformada vira erro JSON-RPC (`-32602`).
- Erro interno não expõe detalhes ao bot; vai para o log.

Endereçamento só dentro da crew. Bot não enxerga bots nem tasks de outras crews.

### 10.1 Aprovações

Com `--permission-prompt-tool mcp__botloft__permission_prompt`, toda ferramenta que precisaria de permissão chama a tool `permission_prompt`. A entrada é `{tool_name, input, tool_use_id}` (documentado; visto com 2.1.284).

1. O daemon grava uma `approval` (`pending`) e o item `approval` no chat, e passa o bot para `needs_approval`.
2. A chamada HTTP fica aberta até o dono responder (`approvals.answer {approvalId, allow, note?}`) ou até `approval_timeout_minutes`, que o dono escolhe nas Configurações (15 min a 8 h no app; o daemon aceita de 1 min a 1 dia). O `timeout` do `mcp.json` (10) só é lido quando o Claude Code inicia: aumentada a espera, cada bot reinicia quando nada estiver em andamento; diminuída, o daemon responde antes e nada reinicia.
3. Resposta ao Claude Code, em texto JSON:
   - Permitir: `{"behavior":"allow","updatedInput":<input original>}`.
   - Negar: `{"behavior":"deny","message":"The owner denied this."}`, com a nota do dono se houver.
   - Sem resposta no prazo: nega com `"The owner did not answer in time."` e a aprovação vira `expired`.
4. Se o processo do bot morrer ou o daemon reiniciar com a aprovação aberta, ela vira `expired`.
5. O bot volta a `busy` e o item do chat é atualizado.

**Permitir sempre.** Um pedido que se repete (o mesmo comando, páginas do mesmo site) pode ser permitido de vez, e o bot para de perguntar por ele. A regra é do bot, não da crew, e fica em `allow_rules`.

- **O que cobre.** O daemon decide pela ferramenta e pela entrada (`AllowScope {toolName, kind, value}`), e o item `approval` leva isso em `always` (`null` quando o pedido não pode ser permitido de vez):
  - `Bash` e `PowerShell`: aquele comando exato, sem os espaços das pontas (`command`). Comando com mais de 2 000 caracteres é um script, que vale ler a cada vez: não tem "sempre".
  - `WebFetch`: o site do endereço, como no navegador (21.5): o host em minúsculas sem `www.`, que cobre também os subdomínios (`site`).
  - `Read`, `Write`, `Edit`, `MultiEdit` e `NotebookEdit`: aquele arquivo, pelo caminho (`file`).
  - `WebSearch`: toda busca (`tool`, com `value` vazio).
  - Nada mais: plano, sugestão de bot, pedido de rotina, navegador e ferramentas desconhecidas pedem sempre.
- **Gravar.** `approvals.answer` com `allow` e `always: true` grava a regra (a mesma de novo não duplica) e avisa com `bot.rules {botId, rules}`. O escopo vem do item do chat, não da entrada gravada, que pode estar cortada.
- **Usar.** Ao receber `permission_prompt`, depois de conferir que a ferramenta está em `running`, o daemon procura uma regra do bot que cubra o pedido. Achando, permite na hora, sem item `approval`, sem `needs_approval` e sem incomodar o dono; o item `tool` do chat mostra o que rodou, como sempre.
- **Desfazer.** `rules.list {botId}` lista as regras, da mais antiga à mais nova, e `rules.delete {ruleId}` tira uma, devolvendo as que ficaram (e avisando com `bot.rules`). Sem a regra, o bot volta a perguntar. Excluir o bot (7.6) leva as regras dele.

**Explicação do comando.** Um pedido para rodar um comando mostrava só o comando, e quem não lê comandos permitia sem saber o que era. A entrada do `Bash` e do `PowerShell` traz, ao lado de `command`, um campo opcional `description`, escrito pelo modelo, que chega igual à tool de aprovação (19). O daemon o passa adiante como `explanation` do item `tool` e do item `approval` (8.2): uma linha, até 300 caracteres; ausente se o campo não veio, veio vazio ou só repete o comando.

- **Quem escreve é o bot.** A explicação vem antes do comando, nunca no lugar dele. O cartão (15.1) a mostra como texto puro, sem markdown nem links, diz que foi o bot que escreveu e que o que roda é o comando, e deixa o comando a um clique, inteiro e com as quebras de linha. Para isso a entrada de um comando é guardada até 32 KB; um comando maior aparece cortado, com o aviso de que não coube. O que o daemon devolve ao Claude Code é a entrada original, a mesma que o dono pôde abrir.
- **Sem explicação**, o cartão diz que o bot não disse para que serve o comando, sugere negar e perguntar, e já vem com o comando aberto. O campo é opcional e o modelo às vezes o deixa de fora (19), então esse caso continua existindo.
- **Regras do bot** (5.1): todo comando leva uma `description` para o dono, que pode não ser técnico: uma frase curta, na língua em que o dono escreve para o bot, dizendo o que o comando faz e por quê, em palavras do dia a dia, sem código, caminhos nem jargão. Sem a regra a descrição vinha curta e técnica, e às vezes em inglês (19). O daemon não conhece o idioma do app; quem acerta a língua é o bot, pela conversa.
- **Outras ferramentas** não têm explicação: a entrada delas não traz um texto escrito para quem lê. O pedido mostra a ação no idioma do dono (15.6) e o `summary` (o arquivo, a URL), com a entrada inteira em "Detalhes". Um pedido para escrever ou editar um arquivo mostra o nome do arquivo, e uma tela HTML aparece em rascunho na área de design antes do OK (22.3); plano, sugestão de bot, pedido de rotina (20.12) e navegador têm cartões próprios. A `description` do `Agent` é o título do trabalho do ajudante e continua sendo o `summary`.

**Plano.** No modo `plan`, o bot pede para seguir com a ferramenta `ExitPlanMode`, cuja entrada traz o plano em markdown (`{plan}`). O pedido passa pela mesma tool e vira no chat um cartão com o plano inteiro: a entrada dessa ferramenta é guardada até 32 KB (as outras, até 4 KB) e o resumo é a primeira linha do plano. "Aprovar plano" permite; "Pedir mudanças" nega com a nota do dono, e o bot continua planejando. Depois de aprovado, o Claude Code troca de modo sozinho (7.4). Que o pedido passa pela tool em `-p` e para qual modo o bot vai ainda precisam de teste real (19).

A aprovação só abre se `tool_use_id` for de uma ferramenta em `running` no chat daquele bot. O daemon espera até 2 s pelo evento, que às vezes chega depois da chamada. Senão nega na hora, sem incomodar o dono. Assim um bot que chame `permission_prompt` por conta própria não consegue pôr um pedido inventado na frente do dono. A descrição da tool também diz para não chamá-la.

### 10.2 Chefe e sugestões de bots

Toda crew nova nasce com um **chefe**: `crews.create` com `lead` cria a crew e esse bot juntos, e `Crew.leadBotId` aponta para ele. O app escreve nome e papel no idioma do dono ("Chefe", "Lidera a equipe: planeja o trabalho, sugere bots novos e distribui as tarefas"); as instruções são o objetivo que o dono escreveu para a crew. O chefe é gravado antes de o supervisor poder subi-lo, então o primeiro start já lê que lidera. O dono troca o chefe (`crews.setLead`) ou deixa a crew sem nenhum; o chefe antigo e o novo têm as regras regravadas e reiniciam quando nada estiver em andamento (7.4). Arquivar ou excluir o chefe deixa a crew sem chefe. Crews de antes não têm chefe até o dono escolher um.

- **Regras do chefe:** além das de todo bot, uma seção "You lead this crew": planejar, dividir o trabalho em partes que rodam em paralelo e distribuí-las com `send_message` (`kind: "task"`); conferir os resultados antes de responder ao dono; quando faltar um especialista, sugerir um bot com `suggest_bot`; escolher o modelo pelo trabalho (`haiku` para o simples e repetitivo, `sonnet` para quase tudo, `opus` ou `fable` só para o raciocínio mais difícil, lembrando que gastam mais do plano); manter a crew pequena; e contar ao dono o que cada bot faz, para ele acompanhar e falar com cada um no chat dele. Os outros bots leem que, se a crew precisar de outro bot, devem pedir ao chefe, que o `crew_roster` marca com `chief`.
- **`suggest_bot`:** aparece para todos os bots, mas só o chefe pode usar; os outros recebem um erro que manda pedir ao chefe. Antes de incomodar o dono, o daemon confere os campos (os de um bot, mais o limite de instruções e o porquê), que o nome está livre na crew e que a crew tem menos de `max_per_crew` bots ativos. Um pedido impossível volta como erro ao chefe.
- **Aprovação:** a sugestão vira um pedido no chat do chefe, pelo mesmo caminho das aprovações (10.1): `approval` com `toolName: "mcp__botloft__suggest_bot"`, entrada até 64 KB e resumo com o nome sugerido. O chefe fica `needs_approval`, e a chamada MCP espera a resposta até `approval_timeout_minutes`. A tool vem liberada por `--allowedTools mcp__botloft`, então o Claude Code não pede permissão antes: quem decide é o dono, pelo cartão.
- **Resposta:** permitir cria o bot na crew do chefe, com modo Manual e o modelo sugerido. O dono pode mudar nome, papel, modelo e instruções antes: `approvals.answer` leva `input`, o JSON da sugestão como ele deixou, que substitui a entrada gravada (o cartão mostra o que foi criado); `input` só vale para sugestões de bot e pedidos de rotina (20.12). Negar volta ao chefe com a nota do dono, e nada é criado. Sem resposta no prazo, o chefe lê que pode sugerir de novo depois. O bot novo sobe na hora e aparece na barra lateral, com o próprio chat; o resultado da tool lembra o chefe de mandar a primeira task.
- **Chefe em `bypass_permissions`:** a sugestão cria o bot na hora, sem cartão, como tudo o que esse modo faz sem perguntar (13). O resultado diz ao chefe que foi criado sem pedir.
- Bots criados assim são bots comuns: o dono conversa, muda, pausa e arquiva cada um como qualquer outro.

## 11. Protocolo do app (JSON-RPC 2.0 sobre WebSocket)

Endpoint: `ws://127.0.0.1:45710/rpc`. Mensagens seguem JSON-RPC 2.0: requests com `id`, respostas com `result` ou `error`, notificações do servidor sem `id`. O maior frame aceito do app é de 32 MiB, por causa dos anexos.

### 11.1 Sessão

- Primeiro request obrigatório: `session.hello {token, client: {name, version}, protocol: 2}` -> `{daemonVersion, protocol}`.
- Qualquer outro método antes do hello: erro `-32001` e a conexão fecha. Token errado também dá `-32001`; `protocol` diferente dá `-32004`; nos dois casos a conexão fecha. O hello tem que chegar em até 10 s.
- `Origin` aceito: `http://tauri.localhost`, `tauri://localhost` e `http://localhost:1420` (dev). Qualquer outro `Origin` recebe HTTP 403 antes do upgrade. Sem `Origin` (cliente nativo, testes) é aceito: navegadores sempre mandam o header, e o token continua obrigatório.
- Um cliente lento que deixa acumular mais de 4096 notificações é desconectado e recarrega o estado ao reconectar.
- Os requests rodam numa thread de bloqueio, nunca numa do runtime. Quem espera o banco numa thread do runtime (o courier, o supervisor, as tools) espera com `block_in_place`, que passa o resto do trabalho dessa thread para outra: parada ali, ela deixaria de rodar os timers e os sockets de todo o daemon até outra thread acordar, e um request lento seguraria até o `/health`. Os de uma conexão são respondidos um de cada vez, na ordem em que chegaram, e a resposta sai antes de qualquer notificação que venha depois dela: o app trata uma resposta como mais nova que tudo o que recebeu antes e grava o objeto inteiro (um `Bot`, uma `Routine`, a lista de uma carga). A exceção são as leituras que o app nunca cruza com notificações (`files.list`, `files.read`, `screens.list`, `attachments.read`, `usage.tokens`, `chat.search`). Elas correm por fora e respondem quando ficam prontas, de modo que varrer uma pasta grande ou ler um arquivo grande não segura o texto ao vivo. Os requests do navegador (21.7, 21.10) pertencem à conexão e são respondidos na hora.
- A versão 2 do protocolo troca `terminal.*` pelo chat (ADR 0001).

### 11.2 Métodos (MVP)

| Método | Params | Result |
|---|---|---|
| `system.status` | | versão, uptime, versão e caminho do claude (`claudeVersion`, `claudePath`), `runtimeError` (por que os bots não sobem), `claudeSignedIn` (7.3; `null` antes de conferir), `account` (o dono para a área da conta do app: `name`, o nome de exibição da conta do Windows, `GetUserNameExW(NameDisplay)`, ou o nome de usuário sem ele, lido uma vez; e `claude`, a conta do Claude com `email`, `plan` e `organization`, ou `null` desconectado), backlog de entrega, `usage` (uso da conta, 8.1) |
| `system.refresh` | | pede uma nova conferência do Claude Code (login e, se falhou, o executável) e responde na hora com o `system.status` atual; o app relê o status até ver o resultado |
| `crews.list` | | `Crew[]` |
| `crews.create` | `name, workFolder?, lead?` (`{name, role, instructions, model?}`: o chefe, 10.2) | `Crew` |
| `crews.rename` | `crewId, name` | `Crew` |
| `crews.setPaused` | `crewId, paused` | `Crew` |
| `crews.setLead` | `crewId, botId` (ou `null`, sem chefe) | `Crew`; o chefe antigo e o novo reiniciam quando nada estiver em andamento (10.2) |
| `crews.setWorkFolder` | `crewId, workFolder` (caminho, ou `null` para voltar à `shared\`) | `Crew`; os bots reiniciam na pasta nova quando nada estiver em andamento (5) |
| `crews.archive` | `crewId` | `Crew` |
| `crews.delete` | `crewId, recycleFolder?` | `{crewId}`; exclui a crew, ativa ou arquivada, com todos os bots dela; com `recycleFolder`, a pasta dela vai depois para a Lixeira (7.6) |
| `bots.list` | `crewId?` | `Bot[]` |
| `archive.list` | | `{crews, bots}`: o que está arquivado, que `crews.list` e `bots.list` deixam de fora, do arquivado mais recente ao mais antigo. `bots` traz todos os bots arquivados, também os das crews arquivadas (7.6) |
| `bots.create` | `crewId, name, role, instructions, color?, model?` | `Bot` |
| `bots.update` | `botId, name?, role?, instructions?, color?` | `Bot` |
| `bots.setPaused` | `botId, paused` | `Bot` |
| `bots.setPermissionMode` | `botId, mode` (`default`, `accept_edits`, `plan`, `auto`, `bypass_permissions`) | `Bot`; o bot reinicia no novo modo quando nada estiver em andamento (7.4) |
| `bots.setModel` | `botId, model` (`default`, `fable`, `opus`, `sonnet`, `haiku`) | `Bot`; o bot reinicia no novo modelo quando nada estiver em andamento (7.4) |
| `bots.setEffort` | `botId, effort` (`default`, `low`, `medium`, `high`, `xhigh`, `max`) | `Bot`; o bot reinicia no novo esforço quando nada estiver em andamento (7.4) |
| `bots.compact` | `botId` | `Bot`, com `context.compacting`; compacta a conversa depois do turno em andamento (8.6). `conflict` se o bot não está rodando |
| `bots.restart` | `botId, fresh?` | `Bot` |
| `bots.archive` | `botId` | `Bot` |
| `bots.delete` | `botId, recycleFolder?` | `{botId, crewId}`; exclui o bot, ativo ou arquivado; com `recycleFolder`, a pasta dele vai depois para a Lixeira (7.6) |
| `chat.history` | `botId, before?, limit?` ou `botId, until` | `ChatItem[]`, mais novo primeiro; `limit` de 1 a 200, 50 se ausente; `until`: do item até o mais novo, até 1 000 (8.8) |
| `chat.search` | `query, botId?, crewId?, before?, limit?` | `SearchHit[]`: itens com todas as palavras, mais novo primeiro, com um trecho em destaque (8.8) |
| `approvals.answer` | `approvalId, allow, note?, input?, always?` (`input`: a sugestão de bot como o dono a deixou, 10.2; `always`: com `allow`, grava o `always` do pedido como regra do bot, 10.1) | `Approval` |
| `rules.list` | `botId` | `AllowRule[]`: o que o bot faz sem perguntar, da regra mais antiga à mais nova (10.1) |
| `rules.delete` | `ruleId` | `{botId, rules}`: as regras que ficaram; o bot volta a perguntar pelo que saiu (10.1) |
| `messages.send` | `botId, body, attachments?` (`[{name, mediaType, data}]`, data em base64) | `Message` |
| `messages.list` | `crewId?, botId?, before?, limit?` | `Message[]` |
| `attachments.read` | `attachmentId` | `{mediaType, data}`, data em base64 (9.5) |
| `files.list` | `botId` | `BotFile[]`: os arquivos que o bot fez, do mais novo ao mais antigo (8.4) |
| `files.read` | `botId, path` | `{mediaType, data}`, data em base64, de um arquivo da lista (8.4) |
| `deliveries.list` | `state?, botId?` | `Delivery[]` |
| `deliveries.retry` | `deliveryId` | `Delivery` |
| `tasks.list` | `crewId?, status?` | `Task[]` |
| `browser.list`, `browser.watch`, `browser.unwatch`, `browser.take`, `browser.release`, `browser.input` | seção 21.7 | o navegador dos bots, a tela ao vivo e o dono no controle |
| `screens.list` | seção 22.4 | as telas HTML do bot |
| `questions.list` | `status?` (`open` se ausente) | `Question[]` de bots e crews ativos, da mais nova à mais velha (23.5) |
| `questions.answer` | `questionId, answer` | `Question`; a resposta vai ao bot como message do dono (23.3) |
| `questions.dismiss` | `questionId` | `Question`; fecha sem avisar o bot (23.3) |
| `settings.get` | | `Settings`: `startWithWindows`, `keepAwake` e `approvalWaitMinutes`, como estão no `config.toml` (6) |
| `usage.tokens` | `since` (ms Unix, não negativo) | `BotTokens[]`: `botId`, `name`, `color`, `crew` (nome da equipe), `archived`, `turns` e `tokens` somados (8.7) |
| `settings.update` | `startWithWindows?, keepAwake?, approvalWaitMinutes?` (1 a 1440) | `Settings`; grava o que veio e aplica na hora (6, 14). Se a tarefa não pode mudar, nada é gravado e volta um erro |

`Crew` traz `workFolder`, o caminho da pasta de trabalho (a escolhida ou a `shared\`), `workFolderChosen` e `leadBotId`, o chefe (10.2). `Bot` traz também `permissionMode`, `model`, `modelInUse`, `effort` e `effortDefault` (7.4), `context` (8.6; `null` enquanto o Claude Code não disse o tamanho da conversa) e `lastActivity`: o último item do chat resumido em uma linha, para a lista de conversas: `kind` (`owner`, `message`, `reply`, `tool`, `approval`, `question`, `notice`), `text`, `tool` e `at`. O `text` não tem palavras do daemon: a mensagem do dono vem sem "You:", e uma ferramenta ou um pedido vêm só com o resumo (num comando, com a explicação do bot, se ele deu uma, 10.1), com a ferramenta à parte em `tool`; o app completa no idioma do dono ("Mandar uma mensagem · @writer", "Aguardando aprovação: rodar um comando"). `lastReplyAt` é quando o bot terminou a última resposta (`null` antes da primeira), lido à parte porque a linha da conversa pode já ser outra coisa (uma ferramenta, a mensagem de outro bot); o app marca a conversa como não lida com ele (15.1).

### 11.3 Notificações do servidor

`bot.state`, `bot.changed`, `bot.context` (8.6), `bot.rules` (10.1), `bot.deleted`, `crew.changed`, `crew.deleted`, `folder.recycled` (7.6), `chat.item`, `chat.delta`, `message.created`, `delivery.changed`, `task.changed`, das rotinas `routine.changed` e `routine.run` (20.8), e do navegador `browser.changed`, `browser.action` e, só para quem assiste, `browser.frame` (21.7), e das telas `screen.draft` (22.3), e das perguntas `question.changed` (23.5).

### 11.4 Erros

| Código | Significado |
|---|---|
| `-32001` | não autenticado |
| `-32002` | não encontrado |
| `-32003` | conflito de estado (ex.: bot arquivado, aprovação já respondida) |
| `-32004` | validação |
| `-32005` | runtime indisponível (claude ausente ou versão antiga) |

Além desses, os códigos padrão do JSON-RPC: `-32700` (JSON inválido), `-32600` (request inválido), `-32601` (método desconhecido), `-32602` (params inválidos) e `-32603` (erro interno; a mensagem não traz corpo de mensagem nem token). `crews.archive` e `bots.archive` são idempotentes. `crews.delete` e `bots.delete` não: o que já foi excluído não existe mais, e a segunda chamada dá `-32002`, como qualquer outro método com esse id.

## 12. Dados (SQLite)

Pragmas: `journal_mode=WAL`, `synchronous=NORMAL`, `foreign_keys=ON`, `busy_timeout=5000`, `temp_store=MEMORY`, cache de 16 MB e `PRAGMA optimize` ao abrir. Com WAL, `NORMAL` não corrompe o banco; uma queda de energia pode perder os últimos commits, mas nenhum commit espera o disco (com `FULL`, cada item de chat pagava um flush no Windows). As consultas quentes passam pelo cache de statements do rusqlite, e toda coluna que aponta para outra tabela tem índice: as tabelas nunca encolhem, e sem índice a exclusão varre a tabela filha uma vez por linha apagada. Migrations numeradas em `botloft-store/migrations/NNNN_nome.sql`, versão em `PRAGMA user_version`. Tempo em milissegundos Unix (`INTEGER`). Arquivamento é lógico (`archived_at`). Exclusão (7.6) apaga as linhas: as chaves estrangeiras não têm `ON DELETE`, então o daemon apaga ou solta, na ordem e numa transação só, tudo que aponta para o bot (`questions`, `approvals`, `chat_items`, `browser_sites`, `allow_rules`, `routine_runs`, `attachments`, `deliveries`, as `messages` recebidas, `routines`, `tasks`), e zera `messages.from_bot_id`, `messages.task_id`, `tasks.origin_task_id` e `crews.lead_bot_id` onde apontavam para o que saiu. Uma crew sai depois dos bots dela.

| Tabela | Colunas principais |
|---|---|
| `crews` | `id, name, slug, paused, work_dir, lead_bot_id, created_at, archived_at` (`work_dir` é a pasta escolhida; `NULL` usa a `shared`. `lead_bot_id` é o chefe, sem chave estrangeira: ele é gravado junto com a crew) |
| `bots` | `id, crew_id, name, handle, slug, role, instructions, color, paused, permission_mode, model, model_in_use, effort, effort_default, token_hash, session_id, created_at, archived_at` (`effort_default` é o nível do próprio modelo, como o Claude Code disse: `low` a `max`, `none` ou `NULL`, 7.4) |
| `messages` | `id, crew_id, from_kind (owner/bot/system), from_bot_id, to_bot_id, kind (note/task/result/system), body, task_id, created_at` |
| `attachments` | `id, message_id, name, media_type, size, path, created_at` |
| `deliveries` | `id, message_id, bot_id, state, attempts, next_attempt_at, lease_until, last_error, sent_generation, turn_uuid, read_at, updated_at` |
| `tasks` | `id, crew_id, requester_bot_id, assignee_bot_id, status, deadline_at, hops, origin_task_id, result, created_at, updated_at` |
| `chat_items` | `id, bot_id, kind, data (JSON), created_at, updated_at` |
| `approvals` | `id, bot_id, tool_use_id, tool_name, input, status, note, created_at, answered_at` |
| `settings` | `key, value` |
| `routines`, `routine_runs` | seção 20.7; `messages` ganha `routine_id` |
| `browser_sites` | `bot_id, host, allowed_at`: sites que o dono deixou o bot usar no navegador (21.5) |
| `allow_rules` | `id (rul_), bot_id, tool_name, kind, value, created_at`, única por `(bot_id, tool_name, kind, value)`: o que o dono permitiu de vez para o bot (10.1) |
| `questions` | seção 23.7; `messages` ganha `question_id` |
| `chat_search` | índice FTS5 do texto de `chat_items`, pela view `chat_text`, mantido por triggers (8.8) |

Índices mínimos: `deliveries(state, next_attempt_at)`, `messages(crew_id, created_at)`, `tasks(assignee_bot_id, status)`, `bots(crew_id)`, `chat_items(bot_id, id)`, `chat_items(created_at) WHERE kind = 'turn'` (8.7), `attachments(message_id)`.

## 13. Segurança e privacidade

- Daemon escuta **só em 127.0.0.1** no MVP.
- Token do owner: 32 bytes aleatórios em `secrets\owner.token`, ACL com acesso só para o SID do usuário atual (DACL protegida, sem herança).
- Token de bot: 32 bytes aleatórios, novo a cada generation. O valor cru existe apenas no ambiente do processo do bot; o daemon guarda só o **SHA-256**, e só em memória: todo processo de bot morre junto com o daemon (Job Object), então nenhum token sobrevive a um reinício e não há motivo para gravá-lo. A coluna `bots.token_hash` fica sem uso.
- Logs nunca registram tokens, conteúdo de mensagens, anexos nem itens do chat em nível `info`. Bots podem manipular dados sensíveis (inclusive de saúde); o daemon trata corpo de mensagem, anexo e saída de ferramenta como dado pessoal.
- `log_level` vale só para os crates do Botloft; dependências ficam em `warn`, porque em `debug`/`trace` a pilha de WebSocket registra frames, que podem conter mensagens. `RUST_LOG` sobrepõe tudo e é só para depuração local.
- Nome de anexo vira só o nome do arquivo (sem `..`, sem pasta, sem caracteres proibidos no Windows) antes de ir para o disco.
- Isolamento entre bots é cooperativo (mesmo usuário do Windows). Documentar isso no README sem prometer sandbox.
- Navegador dos bots (21): perfil próprio por bot, sem os logins do navegador do dono. A porta do DevTools escuta só em 127.0.0.1 e não tem senha: enquanto o navegador está aberto, outro programa do computador pode controlá-lo, como pode fazer com o resto do que roda com o usuário. Páginas não chegam a ela: o Chromium recusa WebSocket com `Origin` de site sem `--remote-allow-origins`. Trocar a porta por um pipe fica para depois. O texto das páginas pode trazer instruções para o bot; as tools dizem a ele que não valem como pedido do dono, e nos modos que perguntam, cada site novo passa pelo dono (21.5).
- Pasta de trabalho escolhida (5): todos os bots da crew a editam como a própria pasta. Por isso ela não pode tocar a pasta de dados do Botloft (segredos, banco) nem conter os workspaces de todos os bots, e não pode ser um disco inteiro.
- Modo `bypass_permissions` (7.4): o bot faz tudo sem perguntar. As regras `deny` de leitura só cobrem as ferramentas de arquivo do Claude Code, alguns comandos do Bash (`cat`, `head`, `tail`, `sed`, `tee`) e redirecionamentos, não um script em Python ou Node, e o Windows nativo não tem sandbox. Um bot nesse modo pode ler `secrets\owner.token`, o banco e as pastas de outros bots, e uma mensagem de outra pessoa pode levá-lo a isso. O app só liga o modo depois de uma confirmação que diz isso, e o bot fica marcado em vermelho (15.1). Um chefe nesse modo também cria bots sem perguntar (10.2), e a confirmação diz isso quando o bot é o chefe.

## 14. Integração com o Windows

| Tema | Solução |
|---|---|
| Iniciar com o Windows | `botloftd service install` registra uma **Tarefa Agendada por usuário**, pela API COM do Agendador (as mensagens do `schtasks.exe` são traduzidas e não dá para lê-las). Dois gatilhos: "ao fazer logon" do usuário, que sobe o daemon na hora, e um gatilho de horário com início no passado repetido **a cada 1 min**, que o traz de volta se ele morrer: com `MultipleInstancesPolicy = IgnoreNew`, a repetição não faz nada enquanto o daemon roda. Token interativo e privilégio mínimo (só com o usuário logado, na sessão dele), sem limite de execução, roda na bateria, prioridade 5 (a padrão, 7, passaria "abaixo do normal" para todos os bots). A ação é `<home>\bin\botloftd.exe serve --home <home> --scheduled`. Não usar Windows Service: roda em outra sessão e sem acesso à autenticação do Claude Code do usuário. Subcomandos `service status`, `service restart`, `service stop`, `service uninstall` (os dados ficam). |
| Escolhas do dono | As Configurações do app decidem os gatilhos. **Rodando:** o de 1 min, mais o de logon se `start_with_windows` (6). **Parado pelo dono** (`service stop`, que o app chama ao fechar quando o dono escolheu parar os bots, 15.1): só o de logon, se `start_with_windows`, e nada o traz de volta antes disso. `install` e `restart` registram os gatilhos de rodando. O de 1 min também dispara depois de um logon novo; por isso, sem `start_with_windows`, o daemon iniciado pela tarefa (`--scheduled`) confere se é um logon novo: `<home>\signin` guarda o logon em que o Botloft rodou pela última vez (sessão e hora do logon, `WTSQuerySessionInformationW` com `WTSSessionInfo`, iguais para o app e para a tarefa), escrito por `install`, `restart`, pelo daemon ao subir e ao mudar a escolha. Logon novo: ele tira os gatilhos da tarefa e sai, até o app abrir e rodar `install`. Logon igual (o daemon morreu e voltou pelo gatilho de 1 min): roda. Iniciado pelo logon depois de parado pelo dono, o daemon devolve o gatilho de 1 min. Logon desconhecido conta como o mesmo, para o daemon nunca deixar de rodar por não saber |
| Reinício da tarefa (testado no Windows 11 25H2) | `RestartOnFailure` **não** reinicia a tarefa quando o processo sai com código de erro, nem numa execução por gatilho de horário nem numa sob demanda. A repetição de um gatilho de logon só começa no próximo logon, não quando a tarefa é registrada. Por isso o gatilho de horário faz o papel de vigia: morto o daemon, ele voltou em 43 s. **Reboot** (0.2.0, 2026-09-29): o daemon subiu 1 s depois do logon, pelo gatilho de logon, com o app fechado, e o bot voltou 13 s depois |
| Uma tarefa por pasta de dados | `Botloft` para `%LOCALAPPDATA%\Botloft`; `Botloft-<8 hex do SHA-256 do caminho>` para outra pasta (`--home` ou `BOTLOFT_HOME` de dev), para instalar um daemon de dev sem tocar no real. `--home` vale para todos os subcomandos e vem antes de `BOTLOFT_HOME` |
| Binário instalado | `service install` copia o próprio executável para `<home>\bin\botloftd.exe` e a tarefa roda essa cópia, nunca a do app: assim o instalador do app troca os arquivos dele com o daemon rodando. Um exe em uso não pode ser sobrescrito mas pode ser renomeado, então o antigo vai para `botloftd.<n>.old` e é apagado numa instalação seguinte. Se o binário não mudou e o daemon dessa versão já roda pela tarefa, `install` não o reinicia; senão para a tarefa, espera o `/health` sumir, inicia de novo e espera o `/health` com a nova versão (20 s) |
| Janela de console | O manifesto do daemon pede `consoleAllocationPolicy = detached` (Windows 11 24H2 e depois): iniciado pela tarefa, ele não ganha console nem janela; num terminal, continua usando o console do terminal. Em Windows mais antigo, `serve` larga o console se for o único processo nele (`FreeConsole`). O manifesto entra como recurso (`embed-resource`), porque a ferramenta de manifesto do linker não conhece o elemento e avisa a cada build |
| Não suspender | Um power request (`PowerCreateRequest` + `PowerSetRequest(PowerRequestSystemRequired)`, motivo "Botloft bots are working") enquanto houver bot `busy`, se `keep_awake = true` (o dono muda nas Configurações e vale na hora). Bot esperando aprovação não conta. Diferente de `SetThreadExecutionState`, o pedido é de um handle e não de uma thread, então qualquer worker do Tokio o liga e desliga, e `powercfg /requests` o mostra. Não impede a suspensão pedida pelo usuário (tampa, menu Iniciar) |
| Processos órfãos | Job Object por bot (seção 7.3) |
| Encerramento | `ctrl_c`, `ctrl_close`, `ctrl_shutdown`, `ctrl_logoff` do Tokio: parar courier, sinalizar bots, flush do banco. Esses sinais são de console: iniciado pela tarefa, sem console, o daemon não tem garantia de recebê-los, e o Windows encerra o processo no logoff e no desligamento. Isso equivale a um crash, e o sistema já é feito para ele (mensagens gravadas antes de sair, bots mortos pelo Job Object, aprovações abertas expiram no próximo start). Um erro fatal na subida vai para o log, porque ninguém vê o stderr da tarefa |
| Caminhos longos | manifesto `longPathAware` no daemon e no app; usar APIs com `\\?\` ao apagar árvores de workspace |
| Arquivo em uso | arquivar e excluir não apagam workspace (5). O perfil do navegador do bot é apagado depois que o navegador fecha, com novas tentativas por 5 s enquanto o Edge ainda segura os arquivos. A pasta que vai para a Lixeira (abaixo) é tentada de novo por 10 s enquanto o processo morto ainda a segura |
| Lixeira (testado no Windows 11 25H2, 2026-09-30) | `IFileOperation` do shell, numa thread própria em STA (o shell exige, e uma thread do daemon pode já estar em MTA, como a do Agendador), com `FOF_NO_UI`, `FOF_ALLOWUNDO`, `FOFX_RECYCLEONDELETE` e `FOFX_EARLYFAILURE`. Sem poder perguntar, o shell apaga de vez o que a Lixeira não pode receber; por isso um `IFileOperationProgressSink` aborta em `PreDeleteItem` todo item que chega sem `TSF_DELETE_RECYCLE_IF_POSSIBLE`, e a pasta fica. Visto: a pasta vai inteira e aparece no registro `$I` da Lixeira com o caminho original; uma pasta com caminhos de mais de 260 caracteres também vai inteira, com o arquivo do fundo; com um arquivo aberto por outro processo a operação falha, a pasta fica como estava e vai na tentativa seguinte. Os testes tiram da Lixeira o que puseram |
| Instância única | lock exclusivo em `<BOTLOFT_HOME>\botloftd.lock` (`File::try_lock`), solto pelo sistema quando o processo termina, mesmo em crash; se já estiver preso, sair com erro claro. É um lock por pasta de dados, e não um mutex de nome fixo, para o daemon de dev (`BOTLOFT_HOME`) rodar ao lado do instalado |
| Porta ocupada | se 45710 estiver em uso por outro processo, sair com erro (sem porta alternativa no MVP) |

### 14.1 Linux e macOS

Em fatias (18, item 8). O que já vale fora do Windows:

| Tema | Solução |
|---|---|
| Processos dos bots | Cada processo de bot e de navegador lidera um **grupo de processos** próprio (`process_group(0)`), e tudo o que ele inicia entra no grupo. Parar o bot manda `SIGKILL` ao grupo inteiro (`kill(-pgid)`), como `TerminateJobObject`. Um grupo que já sumiu (`ESRCH`) é esquecido, para um número reaproveitado não atingir outro grupo. Um processo que cria sessão própria (`setsid`) sai do grupo. Quando o daemon morre, o grupo não morre junto como com `KILL_ON_JOB_CLOSE`: o `claude` sai sozinho quando o stdin fecha (7.4), mas um navegador continua. No Linux, o `systemd --user` mata todo o cgroup da unit quando o daemon para ou morre (`KillMode=control-group`). No macOS o launchd só mata o grupo do próprio daemon, e um daemon iniciado à mão não tem quem mate nada. Por isso, nos dois sistemas, cada grupo de bot e de navegador é anotado ao subir em `<home>/run/groups` (uma linha JSON com o id do grupo e os argumentos, arquivo `0600`). O daemon seguinte da mesma pasta de dados, logo depois de pegar o lock e antes de subir qualquer bot, mata (`SIGKILL` ao grupo) cada grupo cujo líder ainda roda com **todos** aqueles argumentos (`ps -ww -o command= -p`), e começa a lista de novo. Os argumentos incluem caminhos e ids únicos do bot (o perfil do navegador, o `mcp.json`, a sessão), então um número reaproveitado por outro programa nunca é atingido. Só o número de grupos mortos vai para o log |
| Ambiente do bot | O que o **shell de login** do dono monta: `$SHELL` (senão o shell da conta, `getpwuid_r`, senão `/bin/sh`) com `-l -c`, sem terminal, imprime uma marca e `env -0`. Assim o bot acha o que o terminal do dono acha (`~/.local/bin`, Homebrew), mesmo com o daemon iniciado pelo sistema com um `PATH` mínimo. A marca separa o que o perfil imprime; `_`, `SHLVL`, `PWD` e `OLDPWD` ficam de fora. Mais de 10 s, saída sem `PATH` ou erro: vale o ambiente do daemon. A leitura vale por 20 s, menos que os 30 s em que o Botloft percebe o Claude Code instalado (15.1). As variáveis de sessão do Claude Code saem nos dois casos |
| Claude Code | O `claude` no `PATH` desse ambiente, senão em `~/.local/bin` (instalador nativo), `~/.claude/local`, `/opt/homebrew/bin` e `/usr/local/bin` |
| Nome do dono | O nome completo da conta (primeiro campo do GECOS), senão o login; `USER`, `LOGNAME` e o nome da pasta pessoal cobrem um uid sem entrada (contêineres) |
| Navegador | 21.2 |
| Iniciar com o sistema (Linux) | `botloftd service install` grava a unit `~/.config/systemd/user/<nome>.service` (`botloft.service` para a pasta padrão, `botloft-<8 hex>.service` para outra, como a tarefa do Windows), com `Type=exec`, `ExecStart` com cada palavra entre aspas (`%` e `$` dobrados), `Restart=on-failure` com `RestartSec=5`, `KillMode=control-group`, `TimeoutStopSec=20` e `WantedBy=default.target`. `start_with_windows` vira `systemctl --user enable` (o gerenciador do usuário a sobe no login) ou `disable`. Depois de uma queda o systemd a traz de volta; um stop pedido (logout, `service stop`) termina com saída limpa e não volta. Por isso não há gatilho de minuto nem conferência de logon novo (`sign_in_id` é sempre vazio): sem `enable`, nada a sobe num login novo. Estado por `is-active` e `is-enabled`, cujas palavras não são traduzidas |
| Iniciar com o sistema (macOS) | Um launch agent em `~/Library/LaunchAgents/io.github.httpsphl.botloft.daemon[-<8 hex>].plist`, no domínio `gui/<uid>`. `RunAtLoad` segue `start_with_windows`. O `KeepAlive` é `PathState` com o arquivo `<home>/run/keep-alive`: o launchd traz o daemon de volta enquanto o arquivo existir. Um `KeepAlive` simples, ou com `SuccessfulExit`, implica rodar no login. O arquivo é criado ao iniciar (`service install`, `restart` e o daemon iniciado pelo agente) e apagado no stop pedido e na saída limpa do daemon (logout incluído). Iniciar recarrega o agente do arquivo (`bootout`, `bootstrap`, com novas tentativas logo depois do `bootout`) e dá `kickstart`; parar apaga o arquivo e manda `SIGTERM` (`launchctl kill`); gravar a escolha só reescreve o arquivo, que o launchd lê no próximo login. Estado por `launchctl print` (`state = running`) |
| Os dois | `service install`, `status`, `restart`, `stop` e `uninstall` iguais aos do Windows; as mensagens dizem "systemd user service" ou "launch agent" no lugar de "scheduled task". Um teste no CI (`tests/service.rs`, só com `BOTLOFT_SERVICE_TEST=1`) instala com o systemd e o launchd de verdade, mata o daemon com `SIGKILL` e o vê voltar, para e confere que não volta, reinicia e desinstala. O runner Ubuntu só tem gerenciador de usuário com `loginctl enable-linger` |
| Não suspender | Como no Windows, enquanto houver bot `busy` e `keep_awake = true`, mas com um processo auxiliar que segura o pedido enquanto vive: `caffeinate -i -w <pid do daemon>` no macOS; no Linux, `systemd-inhibit --what=idle --mode=block` com um `sh` que espera o daemon sumir (`kill -0` a cada 5 s). Os dois impedem só a suspensão por inatividade, não a pedida pelo dono (tampa, Suspender). O auxiliar sai quando o daemon morre, então uma queda não deixa o computador acordado. Um auxiliar que sai em 200 ms (sem logind, num contêiner) vira erro no log, e o daemon segue |
| Lixeira | Ainda não: avisa que só existe no Windows |

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
    files/        painel dos arquivos que o bot fez: lista, prévia, abrir
    browser/      painel do navegador do bot: tela ao vivo, cursor, pedido de site (21.8), dono no controle (21.10)
    screens/      área de design: as telas HTML do bot, ao vivo enquanto ele escreve (22.5)
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
  setup/          a tela de instalação (setup.html, 15.7): SetupHost, FakeSetupHost
```

Componentes dependem só de `BotloftApi` e `Host`, nunca do cliente concreto nem do Tauri.

O store recarrega crews, bots e `system.status` a cada (re)conexão e depois segue as notificações. `system.status` não tem notificação e é relido a cada 15 s, menos com a janela fora de vista (minimizada, escondida perto do relógio ou aberta escondida no login do Windows): aí para, e é relido quando ela volta. Fora de vista, as animações também param onde estão (`paused.css`) e continuam quando a janela volta. O Windows pode não avisar a página quando o próprio app esconde a janela, então o app marca isso ele mesmo (`shell/visibility.ts`), e a janela voltar à frente desmarca.

O chat e as mensagens de uma equipe guardam todas as páginas já carregadas. As linhas fora de vista não são diagramadas nem pintadas até chegarem perto da tela (`content-visibility: auto`, em `offscreen.css`), e guardam o tamanho que tiveram para a barra de rolagem não pular. As 8 mais novas ficam de fora: é onde os cartões chegam com o brilho de atenção, que a contenção cortaria.

- **Deliveries:** entram no store pela message (cada message tem uma). Vêm as 500 atualizadas mais recentemente e todas as mortas, depois cada `delivery.changed`.
- **Tasks:** todas, depois cada `task.changed`.
- **Chat e timeline** não ficam no store. Cada um carrega uma página (50) do que mostra, pede as anteriores sob demanda com `before` e acrescenta o que chega por notificação (`chat.item`, `chat.delta`, `message.created`).

Layout, como um app de mensagens:

- **Barra lateral:** crews como seções, com os bots como conversas. Cada conversa mostra avatar, nome, estado (cor, ícone e texto) e a prévia da última atividade (`lastActivity`) com a hora. Um bot que respondeu desde a última vez que o dono olhou o chat dele fica marcado como não lido, como num app de mensagens: nome em negrito, prévia e hora em destaque e um ponto na cor de destaque no fim da linha, até o dono abrir o chat. Vale toda resposta (`lastReplyAt`, e cada item `reply` que chega), mesmo com outra coisa depois dela na conversa. Com o chat aberto e a janela à vista, o que chega já conta como visto; com a janela escondida, conta quando ela volta. O app guarda no `localStorage` quando o dono olhou cada bot (`botloft.seen`) e, na primeira vez, a hora de começar a marcar (`botloft.seenSince`): respostas de antes contam como vistas, para uma atualização não marcar todos os bots de uma vez. O dono também marca um bot como não lido pelo menu dele ("Marcar como não lida"; num bot marcado, "Marcar como lida"), para lembrar de voltar à conversa: a marca fica no `localStorage` (`botloft.unread`) até ele abrir o chat, e marcar o chat aberto o fecha e volta à página da crew. Ao lado do nome de cada crew, um número na cor de destaque diz quantos bots dela estão não lidos. Ela aparece desde a conexão, também na tela de boas-vindas antes da primeira crew. Um clique direito numa conversa (ou a tecla de menu do teclado) abre o menu do bot onde o ponteiro está, sem abrir o bot: são as mesmas ações do menu "mais" do cabeçalho (editar, tornar chefe ou deixar de ser, reiniciar com uma conversa nova, abrir a pasta, marcar como não lida ou lida, arquivar, excluir), vindas do mesmo lugar no código (`useBotActions`), para os dois nunca divergirem. O menu fica inteiro dentro da janela, anda com as setas e fecha com Esc, com um clique fora ou quando a lista rola. Um botão na barra de título, ao lado do nome do Botloft, esconde a barra lateral (ou Ctrl+B), para numa tela pequena sobrarem só o chat e o painel do bot: ela desliza para a esquerda e o resto se alarga. A escolha fica no `localStorage` (`botloft.sidebar`). Escondida, um ponto no botão avisa quando algo espera o dono (o mesmo que marca o ícone na barra de tarefas, 15.2) ou, parado e na cor de destaque, quando algum bot tem resposta não lida. Na tela de boas-vindas, antes da primeira crew, ela não se esconde. Uma seta ao lado do nome de cada crew a recolhe: ficam só o nome, o número de não lidos e, se um bot dela espera o dono, um ponto na cor de aviso; a conversa aberta continua à vista. Um botão no topo recolhe ou expande todas. As crews recolhidas ficam no `localStorage` (`botloft.collapsedCrews`).
- **Página das crews:** clicar em "Equipes", no topo da barra lateral, abre no meio da janela todas as crews como cartões: nome, número de bots, os rostos dos primeiros seis (parados, para a página não redesenhar cada chama), quantos bots trabalham e quantos esperam o dono, ou "Pausada". Um cartão abre a crew. É também o que aparece quando nenhuma crew está escolhida (a crew aberta foi arquivada ou excluída), e ela continua aberta depois de uma reconexão; só a primeira carga abre a primeira crew.
- **Área da conta**, no pé da barra lateral, como nos apps de chat: um círculo com a inicial, o nome do dono (do Windows) e o plano do Claude ("Plano Max"; sem plano, a organização ou o e-mail; desconectado, "Sem conta do Claude conectada"). Um clique abre um menu para cima com o e-mail e: **Uso** (as janelas de uso do plano, 8.1, com a parte usada e quando renovam; antes da primeira resposta de um bot, um aviso de que ainda não há dados), **Configurações** (abaixo), **Idioma** (submenu ao lado, com a escolha na hora), **Novidades** (a página de releases no navegador) e **Ajuda** (o README no navegador). Tema, tamanho e idioma saíram da barra de título; ela só mostra o idioma nas telas de preparo, que ainda não têm barra lateral.
- **Configurações**, um diálogo com as partes listadas à esquerda (abas verticais): **Geral**, **Conversa**, **Avisos**, **Aparência**, **Arquivados** e **Sobre**. As escolhas que são só do app ficam no `localStorage` (`botloft.whenClosed`, `botloft.tray`, `botloft.openAtSignIn`, `botloft.notifyNeeds`, `botloft.notifyDone`, `botloft.markReplies`, `botloft.sound`, `botloft.appSounds`, `botloft.enterSends`, `botloft.followBot`, `botloft.lessMotion`), como o tema. Em Geral, primeiro o **Idioma**, numa lista suspensa (um botão com a escolha que abre a lista embaixo, no visual dos menus, com as setas, Enter e Esc, que fecha só a lista); depois **Em segundo plano**, três chaves, cada uma com uma linha que diz o que a escolha atual faz: "Continuar trabalhando depois de fechar o Botloft" (ligada, os bots seguem depois que a janela fecha; desligada, fechar a janela a esconde, chama `daemon_stop` e fecha, e os bots continuam de onde pararam quando o app abre de novo, com "Iniciando seus bots…" no lugar de "Preparando o Botloft…"; a escolha é do app), "Mostrar o Botloft perto do relógio" (só com a anterior ligada; abaixo), "Iniciar com o Windows" (`startWithWindows`, 14), "Abrir a janela ao entrar no Windows" (só com a anterior ligada; desligada, o Botloft começa perto do relógio, ou só abre quando o dono abrir) e "Não deixar o computador dormir enquanto os bots trabalham" (`keepAwake`, com a linha de que a tampa e o Suspender ainda põem o computador para dormir). As duas do daemon vêm de `settings.get` ao abrir, mudam na hora na tela e voltam atrás, com um aviso, se `settings.update` falhar. Em Conversa, "Enter envia a mensagem" (desligada, Enter quebra a linha e Ctrl+Enter envia, e a dica do compositor diz isso) e "Abrir o navegador e as telas quando um bot começar a usar" (desligada, só o ponto no botão avisa), e quanto tempo um pedido de permissão espera pelo dono (15 min, 30 min, 1 h, 2 h, 4 h ou 8 h; um valor posto à mão no `config.toml` entra na lista), com a linha de que sem resposta o pedido é negado (10.1). Em Avisos, "Quando um bot precisar de você" (ligada), "Quando um bot terminar" (desligada), "Marcar o ícone quando um bot responder" (ligada, `botloft.markReplies`: a marca do ícone na barra de tarefas, 15.2, aparece também enquanto algum bot está não lido; só no Windows, onde essa marca existe) e "Tocar um som" (ligada), com a linha de que, com a janela fechada, os avisos só chegam com o Botloft perto do relógio quando ele não está. Em Aparência, tema, tamanho e "Menos animações", que marca a raiz com `data-motion="less"` e para tudo como o Windows faria (15.3). Em **Arquivados**, uma linha que explica que os bots e as equipes arquivados estão parados e fora da vista, mas o Botloft ainda guarda as conversas deles, e a lista, lida de `archive.list` ao abrir a parte: primeiro as equipes ("Equipe · 2 bots · arquivada há 3 dias"), depois os bots arquivados de equipes ativas ("Bot de Ops · arquivado há 3 dias"); um bot de uma equipe arquivada sai junto com ela e não aparece sozinho. Cada linha tem um botão Excluir, que abre a mesma confirmação dos menus (7.6), dizendo que o bot ou a equipe "sai do Botloft de vez" em vez de "para agora", porque já estava parado. Sem nada arquivado, a parte diz isso. Não há como desarquivar. Em **Sobre**: as versões do Botloft e do Claude Code, o e-mail e "Procurar atualizações", que consulta o feed na hora (15.5) e diz que esta é a versão mais recente ou mostra "Ver atualização", que abre o diálogo de atualizar. A linha do Botloft na tela de boas-vindas segue as mesmas escolhas.
- **Perto do relógio:** com os bots continuando depois de fechar e "Mostrar o Botloft perto do relógio" ligada, o app põe um ícone na área de notificação (o ícone do app, com um ponto quando algo espera o dono, como o overlay da taskbar). A dica diz "Botloft: " e o que acontece: um bot que precisa do dono pelo nome, algo esperando, quantos bots trabalham ou nenhum. Um clique abre a janela; o menu tem essa mesma linha, "Abrir o Botloft", "Pausar todas as equipes" (ou "Retomar todas as equipes", com todas pausadas) e "Sair do Botloft", que fecha o app e deixa os bots trabalhando. Com o ícone, fechar a janela só a esconde; sem ele, o app fecha.
- **Avisos:** quando um bot passa a precisar do dono (pedido de permissão, com a ação no idioma do dono, por exemplo "Rodar um comando", sem o comando; ou entrar na conta do Claude) e, se o dono quiser, quando um bot passa de trabalhando a disponível ("Scout terminou", com a equipe). Com o Botloft fora da frente (janela escondida, minimizada ou sem foco), é um aviso do Windows, com o som do Windows se "Tocar um som" estiver ligada; com ele na frente, só um som curto feito na hora (duas notas, sem arquivo). **Sons do app** (`botloft.appSounds`, ligada por padrão) é outra chave, para sons mais baixos que esse, também feitos na hora e no mesmo timbre, só com o Botloft na frente: um "tic" quando a mensagem do dono sai, um "pop" quando chega uma resposta no chat aberto, duas notas claras quando um bot compartilha um arquivo nesse chat (só o que chega depois de abrir o chat) e uma faísca subindo quando nasce um bot ou uma equipe (de qualquer jeito: pelo dono, pelo chefe ou por outra janela). Nada toca a menos de 1,5 s de outro som, para muitos bots juntos não virarem um coral; só o pedido de um bot que precisa do dono passa por cima. Nada de som em clique, tecla, troca de chat ou texto chegando. O que já estava assim quando o app conectou não avisa. Abrir o Botloft pelo aviso (o que traz a janela aberta, 15.2) até 10 min depois abre o bot do último aviso.
- **Área principal com um bot:** cabeçalho com nome, estado e ações; o chat; o compositor embaixo. O compositor aceita texto, colar imagem e arrastar ou escolher arquivos. Enter envia e Shift+Enter quebra linha, ou, se o dono escolheu nas Configurações, Ctrl+Enter envia e Enter quebra linha (a dica aparece enquanto o dono escreve). O texto que o dono começou a escrever fica guardado por bot: ir para outro chat e voltar não o perde. Fica só na memória do app enquanto ele está aberto, porque uma mensagem pode ter dado pessoal; os arquivos escolhidos e ainda não enviados não são guardados.
- **Modo do bot**, no compositor, ao lado do clipe, como no Claude Code: um botão com o modo atual abre um menu para cima, "Modo", com Automático, Manual, Aceitar edições e Plano, cada um com uma linha que fala do bot pelo nome ("Scout decide o que precisa do seu OK") e a marca no atual. Separado, "Ignorar permissões" com o botão Ativar, que abre uma confirmação dizendo que o bot não fica preso à pasta dele (13). Nesse modo o botão fica vermelho e o cabeçalho mostra "Não pergunta nada" em vermelho. Com o bot ocupado, uma linha acima do compositor avisa que ele muda de modo quando terminar o que está fazendo.
- **Modelo do bot**, no compositor, ao lado do botão de enviar: mostra o modelo em uso ("Opus 5.5"; antes do primeiro turno, o nome escolhido ou "Padrão") e abre um menu para cima, "Modelo", com Padrão do plano, Fable, Opus, Sonnet e Haiku. Cada um tem uma linha que fala do bot pelo nome ("Scout fica rápido e capaz, bom para quase tudo"; o padrão diz qual modelo é hoje), e o pé do menu lembra que modelos mais capazes gastam o limite do plano mais rápido. A janela de criar e editar bot tem a mesma escolha. Com o bot ocupado, a mesma linha acima do compositor avisa que ele troca quando terminar.
- **Esforço do bot** (7.4), no compositor, ao lado do modelo: um botão com o nível em uso ("Médio") abre para cima "Esforço", um controle deslizante de cinco pontos, de "Mais rápido" a "Mais inteligente" (Baixo, Médio, Alto, Extra alto, Máximo), como o dos apps do Claude. O ponto **recomendado** vem marcado: é o nível que o modelo do bot usa sozinho (`effortDefault`). Embaixo, o nome do nível e uma linha que fala do bot pelo nome ("Scout equilibra rapidez e raciocínio, bom para quase tudo"); o pé lembra que mais esforço gasta o limite do plano mais rápido e, fora do recomendado, traz "Usar o recomendado". As palavras seguem o controle na hora, mas o bot só é avisado quando o controle fica 400 ms num ponto ou o menu fecha, para não reiniciar a cada passo. Parar no ponto recomendado grava `default`: o bot segue o nível do modelo, também depois de trocar de modelo. Enquanto o Claude Code não disse o nível do modelo, nada vem marcado e o botão diz só "Esforço". Num modelo sem níveis (Haiku) o botão fica apagado e o menu explica, sem o controle. Com o bot ocupado, a linha acima do compositor avisa que ele muda quando terminar.
- **Espaço da conversa** (8.6), no compositor, antes do esforço: um anel que enche com a parte da janela em uso e passa à cor de aviso a partir de 80% do caminho até a compactação automática. Um clique abre "Espaço da conversa": o uso ("556k / 1M (56%)"), uma barra com a marca de onde a conversa se compacta sozinha, quanto falta para isso ("Faltam 411k para ela se compactar sozinha, em 967k"), uma linha dizendo que conversa mais cheia gasta mais do plano a cada mensagem, e **Compactar agora** (`bots.compact`), com a linha do que isso faz. Durante a compactação o botão diz "Compactando…" e o anel pulsa; com o bot ocupado, a linha acima do compositor avisa que ele compacta quando terminar; com o bot parado, o botão fica desligado e diz por quê. Sem `Bot.context`, o anel não aparece. No chat, a compactação vira um aviso no idioma do dono: feita a pedido, feita sozinha porque a conversa encheu, ou o motivo de não ter dado.
- Num chat estreito, a linha de baixo do compositor nunca se divide em duas: o modo e o esforço mostram só o ícone, as setinhas dos menus somem e o nome do modelo é cortado por último, em vez de tudo sair da caixa. No cabeçalho, o @ do bot é cortado antes do nome, e Pausar e Reiniciar mostram só o ícone (o nome fica na dica e para o leitor de tela).
- **Área principal com uma crew:** a timeline (messages entre os bots e do dono) e as tasks. Embaixo do nome, "Trabalha em <pasta>", que abre a pasta no Explorer; o menu da crew tem "Abrir pasta de trabalho" e "Mudar pasta de trabalho…", que abre o seletor de pastas do Windows e confirma antes de mudar, avisando que cada bot reinicia quando terminar o que está fazendo.
- **Nova crew:** nome; "Para que é esta equipe?", o objetivo que vira as instruções do chefe, com a explicação de que a equipe começa com um Chefe que planeja o trabalho e sugere os bots de que precisa; "Pasta de trabalho", com "Escolher pasta…" (o seletor do Windows; sem escolha, "Uma pasta nova dentro do Botloft", a `shared\`); e "Modelo do Chefe". Criada, o app abre o chat do chefe.
- **Chefe:** uma coroa ao lado do nome na barra lateral e nos cartões da crew, e "Chefe" no cabeçalho do bot, com a dica "Lidera <crew>: planeja o trabalho e sugere bots novos". O menu do bot tem "Tornar chefe da equipe" ou "Deixar de ser chefe".
- **Excluir** (7.6): "Excluir bot" é a última ação do menu do bot (no cabeçalho e no clique direito) e "Excluir equipe" a última do menu da crew, as duas em vermelho, depois de Arquivar. A confirmação leva o nome no título ("Excluir Scout?") e diz, em palavras simples, o que sai e o que fica: o bot para agora e sai do Botloft de vez, com a conversa, as rotinas e as tarefas de que fazia parte, e não dá para desfazer; se ele é o chefe, que a equipe fica sem chefe; e, num quadro à parte, que a pasta dele continua no computador, com o caminho para copiar. A da crew diz quantos bots param e saem com ela, e mostra a pasta de trabalho, lembrando que a pasta de cada bot também fica. No mesmo quadro, uma caixa desmarcada: "Mandar esta pasta para a Lixeira" (na crew, "Mandar as pastas da equipe para a Lixeira"). Marcada, o texto do quadro passa a dizer que a pasta vai para a Lixeira, de onde ainda dá para recuperar, e a crew mostra a pasta dela no Botloft em vez da `shared\`; se o dono escolheu a pasta de trabalho, o quadro diz que a pasta de cada bot vai e a escolhida fica onde está. O app manda `recycleFolder` só com a caixa marcada. Se a pasta não pôde ir (`folder.recycled` com `error`), um aviso diz que ela não foi para a Lixeira e continua lá, com o caminho e o motivo. Confirmado, o app tira o bot ou a crew da tela sem esperar o aviso do daemon; se falhar, um aviso diz que não foi possível e nada muda. Excluído o bot aberto, fica a página da crew; excluída a crew aberta, nenhuma fica escolhida (sem crews, a tela de boas-vindas). Uma message de um bot que saiu, arquivado ou excluído, aparece na conversa e na timeline como de "um bot que não está mais na equipe".
- **Sugestão de bot** (10.2): um cartão no chat do chefe, "<chefe> sugere um bot novo", com o porquê e os campos editáveis Nome, Modelo, Função e Instruções, a linha de que o bot começa na hora e recebe o trabalho do chefe, um campo para dizer ao chefe por que não, e os botões Criar bot e Agora não. Respondido, vira uma linha ("Você criou Designer", "Você recusou Designer") que abre o que foi sugerido.
- **Pedido de permissão** (10.1): um cartão no chat, "<bot> quer <ação>" ("Scout quer rodar um comando"), com Permitir, Negar e um campo para dizer ao bot por que não. Num comando, o texto principal é a explicação do bot, em letra comum; embaixo, menor, "<bot> escreveu isto. O que roda de verdade é o comando abaixo.", e "Ver o comando", uma seta que abre o comando inteiro, em letra de código e com as quebras de linha. Sem explicação, o cartão diz "<bot> não disse para que serve este comando. Na dúvida, negue e pergunte." e já vem com o comando aberto. Um comando grande demais para guardar aparece até onde coube, com o aviso de que está cortado. Nas outras ferramentas, o resumo (o arquivo, a URL) e a entrada inteira em "Detalhes". Quando o pedido tem `always` (10.1), entre Permitir e Negar fica um terceiro botão que diz o que cobre: "Permitir sempre este comando", "Permitir sempre em example.com", "Permitir sempre neste arquivo" ou só "Permitir sempre"; a dica dele avisa que o bot não vai pedir aquilo de novo e que dá para desfazer nos detalhes. Respondido, o cartão vira uma linha ("Permitido: rodar um comando") com a explicação ou, sem ela, o comando. A linha da ferramenta acima do cartão segue a mesma ideia: "Rodar um comando" e a explicação do bot (sem ela, o comando), e um clique abre o comando e a saída; a lista de conversas mostra o mesmo texto.
- **Detalhes do bot** (pasta, instruções, sessão) ficam num painel, fora do caminho da conversa. No fim dele, "Permitido sem perguntar" lista as regras do bot (10.1): a ação no idioma do dono ("Rodar um comando") e, embaixo, o comando, o site ou o arquivo, cada uma com um X que faz o bot voltar a perguntar. A lista é lida ao abrir o painel e segue `bot.rules`; vazia, diz que "Permitir sempre" coloca as coisas ali. Detalhes e arquivos são painéis laterais que o dono redimensiona arrastando a borda esquerda (ou com as setas do teclado, Shift anda mais; duplo clique volta ao padrão): 256 a 900 px, sempre deixando ao chat ao menos 22rem, e a largura de cada um fica no `localStorage` (`botloft.panel.details`, `botloft.panel.files`, `botloft.panel.browser`, `botloft.panel.screens`). O chat encolhe com eles e nunca passa por baixo nem por cima do painel.
- **Arquivos do bot** (8.4): um botão "Mostrar arquivos" no cabeçalho abre, à direita do chat, um painel com o que o bot fez, do mais novo ao mais antigo (ícone e cor pelo tipo, nome, pasta, tamanho, "há 5 min"). Cada tipo tem seu ícone num quadrado da sua cor, para Word, planilha e PDF se distinguirem de longe: PDF vermelho, Word azul, planilha verde, apresentação laranja, imagem roxa, áudio e vídeo rosa, compactado âmbar, código azul-petróleo, texto e o resto cinza. O tipo vem primeiro da extensão (os `.doc`, `.xls`, `.ppt` e OpenDocument chegam como `application/octet-stream`) e depois do media type. Um clique mostra o arquivo no próprio painel: imagem, PDF, markdown e texto aparecem; o resto diz que não tem prévia. "Abrir" usa o programa que o Windows escolheu e "Mostrar na pasta" abre o Explorer com o arquivo marcado. A lista é lida ao abrir o bot, a cada 4 s enquanto ele trabalha com o painel aberto e a janela à vista, e de novo quando ele para: cada leitura percorre as pastas do bot, então ninguém paga por ela sem olhar a lista. Com o painel fechado, o contador abaixo se atualiza quando o turno acaba. Com o painel fechado, o botão ganha um contador dos arquivos que apareceram desde a última vez que o dono olhou; abertos, esses levam a etiqueta "Novo". No chat, a linha de uma ferramenta que mudou um arquivo (`Write`, `Edit`...; item `tool` com `file`, 8.2) e não falhou ganha o botão "Ver em arquivos", que abre o painel direto na prévia desse arquivo; se ele não existe mais, o painel diz. Um `share_file` que deu certo (8.4) não vira linha: cada arquivo vira um cartão no chat, com o quadrado do tipo (ou a miniatura de uma imagem de até 8 MB, lida com `files.read`), nome, extensão e tamanho, "Abrir" e "Salvar como…" (`save_file_as`, 15.2). Um clique no arquivo abre a prévia no painel. Num chat estreito, o botão de salvar fica só com o ícone. Se a resposta da tool não puder ser lida, ou se a chamada falhou, fica a linha da ferramenta, com o erro. Detalhes, arquivos, o navegador (21.8) e as telas (22.5) dividem o mesmo lugar: abrir um fecha o outro. O painel abre deslizando da borda direita (460 ms, sem pressa no começo) e fecha voltando para ela (360 ms) antes de sair, com o conteúdo na largura final o tempo todo e o chat acompanhando; quem troca de painel começa da largura do anterior. Fechando, o painel já não recebe clique nem leitor de tela. Com movimento reduzido, abre e fecha de uma vez. O painel segue o que o bot começa a fazer, para o dono ver acontecer: o navegador abre quando o bot sobe o navegador ou pede uma mão (21.8), as telas quando ele começa a escrever uma (22.5), no lugar do painel que estiver aberto e uma vez por começo, a menos que o dono tenha desligado isso nas Configurações. Fechado pelo dono, fica fechado, com o ponto no botão. O navegador nas mãos do dono (21.10) nunca é trocado. O painel volta com o bot: o app guarda, por bot e até fechar (no store, não no `localStorage`), o painel que ficou ao lado do chat, aberto pelo dono ou pelo bot, e trocar de bot ou ir à página da crew e voltar o mostra de novo, já aberto, sem deslizar. Um painel que o dono fechou volta fechado. Só o painel é guardado: o arquivo ou a tela em foco dentro dele e o navegador nas mãos do dono, que é devolvido ao trocar de bot (21.10), não. Voltar conta como abrir o bot: um navegador que o bot abriu nesse meio-tempo fica atrás do botão, e o que abre sozinho ao abrir um bot (um pedido de mão ainda sem resposta, uma tela que ele ainda escreve) e o que ele começa dali em diante entram no lugar do painel que voltou.

### 15.2 Comandos Tauri

| Comando | Função |
|---|---|
| `daemon_status` | GET `/health` local; diz se o daemon roda, está parado ou se outro programa ocupa a porta. Quando o daemon roda, diz também se ele é `outdated`: versão menor que a do app, que é a do sidecar (um workspace Cargo só). Pré-release é ignorado; versão ilegível nunca é antiga |
| `daemon_install` | roda `botloftd --home <home> service install` a partir do sidecar (o `botloftd.exe` ao lado do app), sem janela, e espera: ele se copia para `<home>\bin`, registra e inicia a tarefa e espera o `/health` (seção 14). Um erro volta na mensagem de uma linha que o daemon escreve no stderr. Substitui o `daemon_start` do M4, que iniciava o daemon destacado ao lado do app |
| `daemon_restart` | `botloftd --home <home> service restart`, do mesmo jeito |
| `daemon_stop` | `botloftd --home <home> service stop`, do mesmo jeito: para o daemon até o app abrir de novo, ou até o próximo logon se ele inicia com o Windows (14) |
| fechar a janela | não é comando próprio: o app ouve `onCloseRequested` da janela (botão, Alt+F4, barra de tarefas; permissões `core:window:allow-hide` e `core:window:allow-destroy`). Se o dono escolheu parar os bots ao fechar, esconde a janela, chama `daemon_stop` e deixa fechar, mesmo se ele falhar; com o ícone perto do relógio, só esconde a janela; senão, fecha |
| janela escondida ao abrir | a janela nasce com `visible: false` e o `setup` a mostra, a menos que o app tenha sido aberto com `--autostart` (ao entrar no Windows): aí ela fica escondida se o Botloft espera perto do relógio, e o app a mostra se o dono quer a janela ou se algo deu errado antes de conectar. Mostrar é `show`, `unminimize` e `setFocus` (permissões de cada um) |
| `open_at_sign_in` | grava ou apaga o valor `Botloft` na chave Run do usuário (`HKCU\Software\Microsoft\Windows\CurrentVersion\Run`) com `"<app>" --autostart`, entre aspas por causa de caminhos com espaço (o `tauri-plugin-autostart` grava sem aspas). Ligando, também libera o app se ele estava desligado nos apps de inicialização do Gerenciador de Tarefas (`StartupApproved\Run`). Build de dev nunca se registra. O app liga quando os bots iniciam com o Windows e o dono quer o Botloft perto do relógio ou a janela aberta, e desliga senão, a cada mudança. No Linux é a entrada `~/.config/autostart/botloft.desktop` (`Exec` com o app entre aspas, `"`, `` ` ``, `$` e `\` escapados e `%` dobrado); no macOS, o launch agent `~/Library/LaunchAgents/io.github.httpsphl.botloft.app.plist` (`RunAtLoad`, `LimitLoadToSessionType = Aqua`). Desligar apaga o arquivo |
| `launched_at_sign_in` | se o app foi aberto com `--autostart` |
| uma cópia só | `tauri-plugin-single-instance`, só em build de release (a de dev roda ao lado da instalada): abrir o Botloft de novo, pelo menu Iniciar ou por um aviso, traz a janela da cópia que roda e manda `botloft://reopened` ao app |
| ícone perto do relógio | não é comando próprio: `TrayIcon` e `Menu` do Tauri (feature `tray-icon`, `core:tray` e `core:menu` do `core:default`, e `core:app:allow-default-window-icon`), com as mudanças em fila para nunca criar dois ícones. O ícone vive no processo, não na página: uma página recarregada tira o ícone `botloft` que ficou (`getById`/`removeById`) antes de criar o seu. No build de dev o texto do ícone termina em `(dev)`, para distinguir da cópia instalada |
| avisos do Windows | `tauri-plugin-notification` (`notification:default`). Sem `sound`, o toast é mudo; com "Tocar um som", `sound: "Default"`. O app instalado usa o próprio AppUserModelID; em dev, o do PowerShell |
| `claude_sign_in` | abre `claude auth login` numa janela de console própria (`CREATE_NEW_CONSOLE`) com o `claudePath` do `system.status` e espera; responde se saiu com 0. Só roda um caminho absoluto de arquivo existente chamado `claude.exe`. Tira do ambiente `CLAUDECODE` e `CLAUDE_CODE_*`, que um app aberto de dentro de uma sessão do Claude Code herdaria. Quem abre é o app, e não o daemon, porque a janela aberta pelo app em primeiro plano vem para a frente. Na janela o dono vê o que acontece e pode colar um código se o navegador pedir. No Linux e no macOS (`claude`, sem `.exe`) é uma janela de terminal: o Terminal pelo `osascript` no macOS; no Linux o primeiro de `x-terminal-emulator`, `gnome-terminal`, `konsole`, `xfce4-terminal`, `kitty`, `alacritty` e `xterm` no `PATH`. O terminal roda `sh -c` com o comando, que grava o código de saída num arquivo numa pasta temporária `0700`; o app espera esse arquivo por até 15 min, porque um terminal pode passar o comando a um processo que já roda e voltar na hora. Fechar o terminal sem entrar deixa o app esperando até esse limite |
| `read_owner_token` | lê `secrets\owner.token` (só no app local) |
| `open_path` | abre uma pasta no Explorer; recusa arquivos, que o Explorer executaria. Fora do Windows, `open` no macOS (que recusa um `.app`, uma pasta que ele iniciaria) e `xdg-open` no Linux, como em `open_file`, `reveal_file` e `open_url` |
| seletor de pastas | não é comando próprio: o app usa o `tauri-plugin-dialog` (`open({directory: true})`, permissão `dialog:allow-open`) para a pasta de trabalho da crew (5) |
| `open_file` | abre um arquivo do bot com o programa que o Windows usa para ele (`explorer.exe <arquivo>`). Só extensões de documento, imagem, som e vídeo (`pdf`, `png`, `md`, `docx`, `xlsx`, `mp4`...); qualquer outra, como `.exe`, `.bat`, `.ps1` ou `.lnk`, é recusada, porque o Explorer a executaria |
| `reveal_file` | abre o Explorer na pasta do arquivo, com ele marcado (`/select,`). Nada é executado. No macOS, `open -R`; no Linux, que não tem um jeito comum de marcar o arquivo, abre a pasta dele |
| `save_file_as` | salva uma cópia de um arquivo do bot onde o dono escolher: o próprio comando abre o diálogo Salvar como do sistema (`tauri-plugin-dialog`, pelo Rust, com o nome do arquivo e um filtro pela extensão) e copia; responde `false` se o dono cancelar. Quem escolhe o destino é sempre o diálogo, então a página não grava arquivo em lugar nenhum que o dono não tenha escolhido. Copiar um arquivo sobre ele mesmo não faz nada |
| `open_url` | abre no navegador padrão um link de uma resposta do bot; só `http` e `https`, porque qualquer outro esquema pode iniciar um programa. Seguir o link dentro do app trocaria a janela pela página |
| overlay na taskbar | não é comando próprio: o app usa `setOverlayIcon` da janela (permissão `core:window:allow-set-overlay-icon`) e marca o ícone com um ponto enquanto algo espera o dono: aprovação pendente, bot em `auth_error`, ou mensagem não entregue a um bot ativo. Entregas mortas para bot arquivado não contam: foram abandonadas de propósito. Com "Marcar o ícone quando um bot responder" (15.1), o mesmo ponto aparece enquanto algum bot está não lido |

O app acha o daemon como o daemon acha a si mesmo (seção 5): `BOTLOFT_HOME` ou `%LOCALAPPDATA%\Botloft`, com a porta lida do `config.toml` dessa pasta (45710 se ausente). Um daemon de dev com seu próprio `BOTLOFT_HOME` é encontrado sem configuração extra. A CSP libera `ws://127.0.0.1:*` pelo mesmo motivo, `blob:` para a imagem e o PDF da prévia de arquivos (15.1) e `http://127.0.0.1:*` em `frame-src` para as telas (22.2).

Onboarding (M5): **configuração sem perguntas e sem jargão.** O dono não precisa saber que existe um daemon, uma porta ou uma tarefa agendada; a interface nunca usa essas palavras. Fala de "Botloft" e de "rodar em segundo plano", e o texto técnico (erro do daemon, caminho, porta) fica dobrado sob "Details".

- Nada rodando: o app chama `daemon_install` sozinho ("Getting Botloft ready…", com uma linha dizendo que o Botloft roda os bots em segundo plano e que nas Configurações o dono escolhe se eles seguem depois que a janela fecha e se o Botloft inicia com o Windows). Se o dono escolheu parar os bots ao fechar, é "Starting your bots…", sem a linha. Se falhar, "Botloft couldn't start" com "Try again".
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

Movimento: o app se mexe para parecer vivo, sem atrapalhar o trabalho. Animações curtas (150 a 400 ms, saída suave) em CSS, sem biblioteca (`motion.css`), e todas param quando o Windows desliga os efeitos de animação (`prefers-reduced-motion`) ou quando o dono liga "Menos animações" nas Configurações (`motion-less.css` repete as mesmas regras para `:root[data-motion="less"]`, e `reducedMotion()` olha os dois).

- **Mascote:** cada bot tem um humor que segue o estado, e a chama queima de verdade: o contorno muda de forma quadro a quadro e o sombreado acompanha (`mascot.css`, `mascot-flame.css`). Parado, a chama balança devagar e os olhos dão uma olhada para baixo e para o lado de vez em quando, como nas poses do desenho; trabalhando (`busy`, `launching`), queima forte e solta faíscas pelo alto; esperando o dono (`needs_approval`), inclina rápido e dá pulinhos; no limite de uso, queima baixo, de pálpebras pesadas que caem de vez em quando; pausado ou parado, vira uma brasa pequena e dorme, de olhos fechados em dois arcos. Os olhos piscam fechando de cima e de baixo, cada bot num ritmo, para uma equipe não piscar junta. Mexem-se os mascotes do cabeçalho e da resposta em andamento, e os da barra lateral e dos cartões da crew enquanto o bot está acordado, parado também: é pela chama devagar e pelas olhadas que o dono vê de relance que ele está livre. Nessas listas, só um bot dormindo mostra o humor sem se mexer (`ListAvatar`), porque cada chama que muda de forma redesenha a tela a cada quadro e um bot desligado não tem o que mostrar. O histórico do chat fica parado. O humor soma ao estado, que continua com cor, ícone e texto.
- **Chegadas:** o que chega enquanto o dono olha entra subindo e aparecendo: mensagens, ferramentas, pedidos, avisos e bots novos na barra lateral. O que já estava lá quando ele abriu o chat (ou o app) aparece parado, e páginas antigas carregadas em cima não se mexem. A resposta já aparece enquanto é escrita, então não anima de novo ao terminar.
- **Trabalhando:** os três pontos pulam em sequência; o texto sendo escrito tem um cursor piscando no fim; a mão de "Precisa de aprovação" acena; um pedido pendente pulsa uma vez ao aparecer. No estado do bot, o ponto de "Disponível" solta dois anéis, um depois do outro, e na linha de "Trabalhando" corre um trecho aceso (`StateIcons.tsx`); com menos movimento, a linha fica inteira acesa.
- **Trocar de bot ou de crew:** o painel principal some rápido e o novo sobe (View Transitions do webview); a barra lateral fica onde está.
- **Superfícies:** menus e seletores abrem a partir de onde estão presos; diálogos aparecem com o fundo escurecendo; avisos entram deslizando; `<details>` abre deslizando; a linha da aba selecionada desliza até a nova; as barras de uso enchem ao abrir; botões afundam um pouco ao clicar; o mascote da tela de boas-vindas e do chat vazio flutua.

No chat:

- O chat ocupa toda a largura que sobra, alinhado à esquerda, com o compositor na mesma largura (sem coluna centralizada, que deixava vazio dos dois lados); o dia é uma pílula no meio.
- O dono fala em balões à direita; o bot, à esquerda, com markdown.
- Embaixo da mensagem do dono, e em cada linha da timeline da crew, fica onde ela está (9.1), com ícone e texto: esperando, entregando, "Entregue" com um tique e, quando o bot começou a trabalhar nela, "Lida" com dois tiques. Os dois tiques de "Lida" ficam em azul-claro, como nos apps de mensagem, para o dono ver de relance que o bot leu; a palavra continua na cor apagada das outras linhas, e é ela que diz o estado. A cor é o token `--read` (`#1787c9` no tema claro, `#5cc4f2` no escuro): mais clara que o `--work`, que é de bot trabalhando e de mensagem sendo entregue, e com contraste de ao menos 3:1 sobre `canvas`, `panel` e `sunken` nos dois temas, o que um teste confere. O azul dos apps de mensagem (`#53bdeb`) não passa de 2,2:1 no tema claro, por isso o claro usa um tom mais fechado.
- Mensagens de outros bots aparecem à esquerda, com o avatar e o nome de quem mandou.
- O que o bot faz com as ferramentas aparece em linhas compactas (ícone, ferramenta, resumo e estado), agrupadas por turno, que abrem para mostrar entrada e saída.
- Pedido de aprovação é um cartão com o que o bot quer fazer e os botões Permitir e Negar.
- Pedido para seguir com um plano (10.1) é um cartão com o plano em markdown, um campo para o que deve mudar e os botões Aprovar plano e Pedir mudanças. Respondido, vira uma linha que abre o plano de novo.
- Anexos aparecem como miniatura (imagem) ou cartão com nome, tipo e tamanho.
- Arquivos que o bot compartilha (`share_file`) aparecem como cartões na fala dele, com Abrir e Salvar como… (15.1).

Identidade: o mascote do Botloft é uma chama com olhos. O ícone do app (`app/app-icon.svg`, de onde `pnpm tauri icon app-icon.svg` gera os PNG, o ICO e o ICNS; as pastas `android` e `ios` que ele cria são apagadas) é o mascote laranja, na cor que mostra o desenho como foi feito, sobre um quadrado preto, montado com os mesmos contornos e camadas de `mascotArt.ts`. Cada bot usa o mesmo personagem como avatar, com uma cor própria escolhida na criação (e mudada ao editar), sem fundo e com um contorno fino e discreto (escuro no tema claro, claro no escuro) para as cores claras não sumirem. O avatar é a primeira pose da folha de poses da chama, vetorizada (`mascotArt.ts`): o corpo com as labaredas, uma gotinha de fogo solta acima da ponta, camadas de luz e sombra recortadas pelo contorno e olhos pretos com brilhos e reflexo avermelhado embaixo, tudo retingido para a cor do bot (matiz, saturação e luminosidade), então todo bot tem a mesma luz e sombra. A referência da retintura é a primeira cor da paleta (`#FF7A59`): um bot laranja fica igual ao desenho, e as cores claras da paleta mantêm a profundidade em vez de desbotar. A gotinha passa um pouco acima do quadrado do avatar, como as faíscas. O mascote do próprio Botloft (barra de título, mensagens do daemon) é o do ícone: laranja sobre o quadrado preto. Na janela de criar e editar bot, a cor sai de uma paleta de 16 (`PALETTE`, que o daemon também dá em ordem a quem não escolhe) ou de "Escolher qualquer cor", que abre um seletor livre: um quadrado de saturação e brilho sobre o matiz escolhido (também com as setas do teclado; Shift anda mais), uma barra de matiz e a cor em hex e em R, G e B, tudo editando a mesma cor. O daemon aceita qualquer `#RRGGBB`. Um bot com cor fora da paleta abre a janela com o seletor aberto. A cor do avatar identifica o bot e não comunica estado: estado continua sendo cor + ícone + texto, como descrito acima. O mascote se mexe conforme o que o bot faz (ver Movimento), sem substituir isso.

### 15.4 Instalador e sidecar

- `pnpm bundle` (em `app/`) roda `scripts/sidecar.mjs`, que compila o `botloftd` em release e o copia para `src-tauri/binaries/botloftd-<target triple>.exe`, e depois `tauri build --config src-tauri/tauri.bundle.conf.json`. Esse arquivo liga o `externalBin` e o NSIS; ele fica fora do `tauri.conf.json` porque o `tauri-build` copia o `externalBin` também em dev e no `cargo clippy`, o que exigiria o sidecar em todo build e sobrescreveria o `target\debug\botloftd.exe` com a cópia de release.
- Quem baixa o Botloft instala pela tela de instalação (15.7), que roda este instalador NSIS em silêncio. O NSIS com as próprias telas só aparece na atualização (15.5), na desinstalação e quando a tela de instalação não pode abrir.
- NSIS por usuário (`installMode = currentUser`), sem pedir administrador, em `%LOCALAPPDATA%\Botloft` (seção 5). O instalador não mexe no daemon: ele roda da própria cópia em `<home>\bin`, então o arquivo do sidecar nunca está em uso.
- Hook `NSIS_HOOK_PREUNINSTALL` (`src-tauri/windows/hooks.nsh`): numa desinstalação de verdade, `botloftd service uninstall` para o daemon e apaga a tarefa e o binário, e o valor `Botloft` da chave Run sai (15.2); bots e dados ficam. Numa atualização (`/UPDATE`), o instalador do Tauri 2.12 nem roda o desinstalador antigo: só troca os arquivos, e o daemon continua rodando até o app novo abrir e atualizá-lo (15.2). Um desinstalador chamado com `/UPDATE` também não mexe no daemon.
- Marca do instalador, sem nome de terceiros à vista: o instalador e o desinstalador usam o ícone do app (`icons/icon.ico`); as telas de boas-vindas e de conclusão têm a imagem lateral `windows/installer-sidebar.bmp` (164×314, o mascote e "Botloft" sobre o preto) e as outras telas o cabeçalho `windows/installer-header.bmp` (150×57, à direita, com `MUI_HEADERIMAGE_RIGHT` no arquivo de hooks). As duas imagens saem do `app-icon.svg` sem o quadrado preto. O rodapé mostra o `bundle.copyright` no lugar de "Nullsoft Install System", e o editor é "Botloft" (`bundle.publisher`), em Aplicativos instalados e como empresa nas propriedades do `Botloft.exe` e da tela de instalação.
- Editor antigo: até a 0.6.0 o `bundle.publisher` ficava vazio e o Tauri usava `github`, a segunda parte do identificador, o que num programa sem assinatura parece alguém se passando pelo GitHub. O Tauri também dá o nome do editor à chave que guarda a pasta de instalação (`Software\<editor>\Botloft`), lida para atualizar na mesma pasta e para o "desinstalar antes de instalar" do assistente. Por isso os hooks adotam a instalação antiga: `BotloftAdoptOldFolder` copia a pasta de `Software\github\Botloft` para `Software\Botloft\Botloft` quando a chave nova não existe e passa a instalar nela, a menos que `/D` tenha escolhido outra. Roda antes da primeira tela (`MUI_CUSTOMFUNCTION_GUIINIT`), o que cobre o assistente e a atualização passiva, e no começo da instalação (`NSIS_HOOK_PREINSTALL`), para o modo silencioso, que não tem telas. Depois de instalar, o `NSIS_HOOK_POSTINSTALL` apaga a chave antiga, e a `Software\github` só se ficar vazia. A tela de instalação (15.7) lê a chave nova e, na falta dela, a antiga. **Verificado** (Windows 11 25H2, 2026-09-30): sobre a 0.6.0 instalada com o editor antigo e com o app aberto, o instalador novo rodado como o atualizador (`/P /UPDATE /R`) terminou em 3 s, criou `Software\Botloft\Botloft` com a pasta que estava na chave antiga, apagou a antiga e a `Software\github` vazia, deixou o editor "Botloft" em Aplicativos instalados e reabriu o app da mesma pasta; o daemon não foi tocado. Os casos de pasta escolhida pelo dono, `/D` e modo silencioso foram testados com um instalador mínimo compilado com o mesmo arquivo de hooks e chaves de teste.
- O manifesto do app (`src-tauri/windows/app.manifest`) é o padrão do Tauri (controles comuns v6) mais `longPathAware`.
- **Linux e macOS** (18, item 8): o mesmo `pnpm bundle` gera os pacotes do sistema em que roda. O Tauri junta ao `tauri.conf.json` o `tauri.linux.conf.json` ou o `tauri.macos.conf.json`, que escolhem os pacotes. Linux: `.deb` (o Tauri põe as dependências: WebKitGTK, GTK e o `libayatana-appindicator3-1` do ícone na bandeja) e AppImage, feitos no Ubuntu 22.04 para o AppImage rodar em distribuições mais antigas (ele usa a glibc do sistema em que foi feito). macOS: `.app` e `.dmg` para Apple Silicon, a partir do macOS 11. Sem conta Apple Developer, o app não é notarizado: tem só assinatura ad-hoc (`signingIdentity = "-"`). Sem nenhuma assinatura, o macOS diria que o app está danificado; assinado assim, quem baixa o libera uma vez em Ajustes do Sistema → Privacidade e Segurança → Abrir Mesmo Assim. O sidecar vai dentro do pacote, ao lado do app, como no Windows. A tela de instalação (15.7) é só do Windows.
- **Remover o `.deb`** faz o que o hook de desinstalação do NSIS faz. O `prerm` (`src-tauri/linux/prerm.sh`, `preRemoveScript`) roda como root, só com `remove` ou `purge`: a atualização pelo app instala o `.deb` novo e chama o `prerm` com `upgrade`, que não mexe em nada. Para cada `~/.config/systemd/user/botloft.service` em `/home/*` e `/root`, roda `botloftd --home ~/.local/share/Botloft service uninstall` como o dono da pasta (`runuser`, com `HOME`, `XDG_RUNTIME_DIR` e o bus da sessão dele), se o gerenciador do usuário estiver no ar (`/run/user/<uid>/bus`). Depois apaga a unit, o link de `default.target.wants` e `~/.config/autostart/botloft.desktop`, o que cobre quem está fora da sessão. Bots e dados ficam. Só a unit da pasta padrão (`botloft.service`) é tratada; um daemon de dev com outra pasta é de quem o instalou. O AppImage e o `.app` do macOS não têm desinstalador: o README e a política de assinatura dizem para rodar `botloftd service uninstall` antes. **Testado** num contêiner Debian, com usuários de mentira e um `botloftd` falso: com `upgrade` nada muda; com `remove`, o daemon de quem tem sessão é chamado como esse usuário, com a pasta certa, e os arquivos de todos saem.

### 15.5 Atualizações

- O app usa o `tauri-plugin-updater`. O feed é `latest.json` do último release publicado no GitHub (`releases/latest/download/latest.json`). O app o consulta ao abrir e a cada 6 h; sem rede ou sem release, fica quieto. Build de dev não consulta, para não se trocar pelo app publicado.
- Com versão nova, aparece "Update available" na barra de título. Um clique abre o diálogo: a versão, "What's new" (se o release tiver notas) e o aviso de que o Botloft fecha, instala e abre de novo, e de que os bots pausam por um instante e continuam de onde pararam. "Update now" baixa com progresso e roda o instalador; se falhar, o erro fica em "Details" e dá para tentar de novo. Antes de rodar o instalador no Windows, o plugin chama `cleanup_before_exit`: esconde a janela, tira o ícone perto do relógio e solta todo recurso que a página guarda, inclusive a atualização achada. Se o instalador não chega a abrir (o dono recusa o UAC, um antivírus o barra), o app segue vivo sem nada disso; por isso, depois de uma falha, o app mostra a janela, refaz o ícone e a próxima tentativa consulta o feed de novo. Sem isso, a segunda tentativa falhava com "The resource id … is invalid" (visto num PC com a 0.9.0, 2026-10-02).
- O feed aponta para o instalador NSIS do release, `Botloft_<versão>_x64-update.exe`, e não para a tela de instalação (15.7). O instalador roda como `/P /UPDATE /R`: passivo, sem perguntas, e reabre o app. O hook de desinstalação não faz nada com `/UPDATE` (15.4), o daemon segue rodando, e o app reaberto o atualiza porque ele ficou `outdated` (15.2).
- Os artefatos são assinados com a chave do updater do Tauri (`createUpdaterArtifacts`), e a chave pública fica no `tauri.conf.json`. `requireSignedVersion` exige que a assinatura traga a versão, para um feed adulterado não empurrar uma versão antiga de volta. O instalador não tem assinatura Authenticode, então o SmartScreen avisa na primeira execução.
- Release: a versão fica só no `[workspace.package]` do `Cargo.toml` (o `tauri.conf.json` não repete a versão e usa a do crate). Um push de tag `vX.Y.Z` roda `.github/workflows/release.yml`. Um job por sistema (Windows, Ubuntu 22.04 e macOS Apple Silicon) confere tag e versão, roda `pnpm bundle` com `TAURI_SIGNING_PRIVATE_KEY` (segredo do repositório), no Windows também `pnpm bundle:setup` (15.7), depois `release.mjs stage <sistema>` e o atestado de origem dos seus arquivos, e os guarda como artefato do workflow. O job final junta os três e roda `release.mjs publish`, que abre um release **rascunho** com tudo e um `latest.json` só. No Windows são a tela de instalação (`-setup.exe`) e o instalador do atualizador (`-update.exe`, com o `.sig`). No Linux, `Botloft_<versão>_amd64.AppImage` e `.deb`, cada um com `.sig`. No macOS, `Botloft_<versão>_aarch64.dmg` e o `.app.tar.gz` que o updater instala, com `.sig`. O `latest.json` tem `windows-x86_64`, `linux-x86_64-appimage`, `linux-x86_64-deb` e `darwin-aarch64`: o plugin procura primeiro a chave com o tipo de instalação (`<so>-<arch>-<instalador>`), então quem instalou o `.deb` recebe o `.deb` (instalado com `dpkg -i`) e quem usa o AppImage recebe o AppImage. O updater só enxerga o release depois que o dono o publica. Rodado à mão (`workflow_dispatch`), o workflow faz os mesmos arquivos e os deixa como artefatos, sem conferir a tag, sem atestado e sem release.
- Atestado de origem: entre o `stage` e o `publish`, o workflow roda `actions/attest@v4` nos dois `.exe` (permissões `id-token`, `attestations` e `artifact-metadata`). É um atestado de proveniência SLSA assinado pelo Sigstore: qualquer pessoa confere com `gh attestation verify <arquivo> --repo httpsphl/botloft` que o instalador saiu deste workflow, deste repositório e daquele commit. As notas do rascunho começam com "Verify your download": o SHA-256 dos dois instaladores e esse comando. Terminam com o link da política de assinatura de código, que o README também mostra no passo de download: a SignPath Foundation pede o termo "Code signing policy" na página inicial e nas de download. O VirusTotal fica de fora das notas enquanto a tela de instalação tiver alertas de aprendizado de máquina (na 0.6.0, 2 de 71: Microsoft `Wacatac.C!ml` e Trapmine), à espera da análise de falso positivo.
- Assinatura de código (Authenticode): ainda não há certificado, e sem ele o release sai sem assinatura. O caminho já está pronto: `app/scripts/sign.mjs <arquivo>` assina com o `signtool` do Windows SDK o certificado que estiver no repositório de certificados da máquina, escolhido pela impressão digital em `BOTLOFT_SIGN_THUMBPRINT` (uma chave em nuvem aparece ali depois que o cliente dela entra na conta), com carimbo de tempo RFC 3161 (`BOTLOFT_SIGN_TIMESTAMP`) e confere a assinatura em seguida. Sem a variável, não faz nada; com ela e sem o certificado, falha, para um release nunca sair sem assinatura por engano. Quem chama: o Tauri, pelo `bundle.windows.signCommand` do `tauri.bundle.conf.json`, para tudo que entra no instalador (o `Botloft.exe`, o sidecar `botloftd.exe`, os plugins do NSIS, o desinstalador e o próprio instalador; ele pula o que já estiver assinado); e o `setup.mjs` para a tela de instalação, que o Tauri não monta. A cópia do daemon em `<home>\bin` é o mesmo arquivo assinado do instalador. O `release.yml` passa as duas variáveis do repositório. O `.sig` do atualizador e o atestado de origem são feitos depois, sobre os arquivos já assinados.
- Todo binário leva nome do produto e versão nos metadados, que é o que um serviço de assinatura confere: o app e a tela de instalação pelo `tauri-build`, o instalador pelo NSIS e o `botloftd.exe` por um `VERSIONINFO` no `botloftd.rc` (produto "Botloft", descrição "Botloft background service", versão do crate), preenchido pelo `build.rs`.
- Opções de certificado vistas em 2026-09-30, para uma pessoa física no Brasil: o Azure Artifact Signing não atende (pessoa física só nos EUA e no Canadá); a SignPath Foundation assina de graça projetos abertos aprovados, com o nome dela como editor e uma aprovação manual por release, mas recusou o pedido do Botloft em 2026-10-01 por falta de sinais públicos (estrelas, forks, colaboradores, citações fora do GitHub) e aceita um novo pedido quando o projeto for mais conhecido; o certificado Open Source da Certum (a partir de € 49, chave em nuvem) sai no nome do desenvolvedor; a SSL.com tem validação individual com assinatura em nuvem e action oficial, por cerca de US$ 300 por ano. Nenhuma tira o aviso do SmartScreen de imediato: a reputação cresce a cada release assinado com a mesma identidade. A política pública está em `docs/code-signing-policy.md`.
- Falso positivo do Defender (0.6.0, 2026-09-30): no PC do dono, o Microsoft Defender pôs em quarentena, por detecção de nuvem, o `-setup.exe` baixado (`Trojan:Win32/Wacatac.C!ml`) e depois o `Botloft.exe` instalado e em uso (`Trojan:Script/Wacatac.C!ml`), apagando os atalhos; o daemon não foi tocado e os bots seguiram. São palpites de aprendizado de máquina sobre binários novos e sem assinatura. Os dois arquivos foram enviados à Microsoft como falso positivo. É o motivo de a assinatura ter deixado de ser opcional para distribuir o app.

### 15.6 Idiomas

- O app fala **inglês, português (Brasil) e espanhol**. Todo texto que o dono lê fica em `app/src/i18n/<idioma>/`, um arquivo por área (`common`, `shell`, `onboarding`, `updates`, `bots`, `chat`, `crews`, `messages`, `setup`). O inglês é a referência: o formato dele é o tipo `Messages`, e um texto que falte ou sobre em outro idioma não compila. Texto com valores é função (`ready(version)`), com o plural escrito para cada idioma.
- Componentes leem com `useT()`; código fora do React (toasts, formatação, erros da conexão) com `t()` na hora do uso.
- Escolha na área da conta (Idioma, ou Configurações), e no botão de idioma da barra de título só nas telas de preparo: "Idioma do sistema" segue o Windows (o primeiro idioma suportado entre os preferidos; `pt-PT` vira `pt-BR`; nenhum, inglês) ou um idioma fixo. A escolha fica no `localStorage` do app (`botloft.locale`) e marca `<html lang>`.
- Datas e horas (`lib/format.ts`) usam o idioma escolhido.
- Textos que nomeiam o sistema ("Iniciar com o Windows", "quando você entra no Windows", os avisos e as animações do sistema) recebem o nome dele: `Windows`, `macOS` ou `Linux`, lido uma vez do user agent da janela do app (`lib/system.ts`; um desconhecido conta como Windows). O aviso de navegador que não abriu fala do Edge no Windows e de Chrome, Chromium ou Edge nos outros (21.2).
- O daemon não escreve texto para o dono: avisos vêm com `code` e a linha da conversa com `kind` (8.2, 11.2), e o app escreve. Continuam como vêm: nomes, mensagens, respostas e saídas de ferramenta; o resumo de ferramenta que o daemon tira da entrada (o comando, o arquivo), sem o nome da ferramenta, que o app escreve; e as mensagens de erro do daemon (validação, falhas), mostradas como texto técnico.
- O instalador NSIS e a tela de instalação (15.7) também trazem os três idiomas e escolhem pelo idioma do Windows.
- Tom: palavras simples, sem jargão; "você" em português, "tú" em espanhol. Glossário: crew = equipe / equipo; task = tarefa / tarea; Allow / Deny = Permitir / Negar / Denegar; role = função / rol.

### 15.7 Tela de instalação

**O que é.** O arquivo que o dono baixa, `Botloft_<versão>_x64-setup.exe`, abre uma janela do próprio Botloft: o mascote, o nome, um botão e mais nada. Por baixo, ela roda o instalador NSIS (15.4) em modo silencioso (`/S`), que continua fazendo o trabalho pesado: arquivos, atalhos com a identidade que os avisos do Windows usam (15.1), a entrada em Aplicativos instalados, o desinstalador, o WebView2 e o hook do serviço. Assim o dono só vê a marca do Botloft, e a atualização, a desinstalação e o serviço não mudam.

**Processo.**

- É um app Tauri pequeno, o crate `botloft-setup` em `app/src-setup/`, com a interface em React em `app/src/setup/` (entrada `app/setup.html`). Ela usa os mesmos tokens, componentes (`ui/`), mascote (`BotAvatar`) e textos (`i18n/<idioma>/setup.ts`) do app. A interface depende só de `SetupHost` (`src/setup/host.ts`), com o `FakeSetupHost` para os testes e para a prévia: com `pnpm dev`, `/setup.html` num navegador comum mostra a tela, e `?relation=older&installed=0.5.0&running&fail` experimenta cada caso.
- O instalador NSIS vai dentro do executável (`include_bytes!`, pelo caminho em `BOTLOFT_SETUP_PAYLOAD` na compilação). Sem ele (build de dev, `cargo clippy`), o setup fica em modo de ensaio: mostra as telas e finge instalar em 3 s, sem tocar na máquina.
- Janela de 480×380, fixa, centrada e sem a moldura do Windows, com uma faixa de título própria (arrastável, só com o botão de fechar), no tema do Windows, claro ou escuro. Ela abre escondida e aparece quando a primeira tela está pronta. O WebView2 da janela guarda os arquivos dele em `%TEMP%\Botloft-setup-webview`, a mesma pasta a cada vez, e não numa pasta do setup em `%LOCALAPPDATA%`.
- O manifesto do setup declara `requestedExecutionLevel` `asInvoker`: o Windows trata programas com "setup" no nome como instaladores e pediria administrador sem isso. Nada no setup precisa de administrador.
- Antes de abrir a janela, o setup confere se o WebView2 existe. Sem ele, grava o instalador NSIS numa pasta temporária, roda-o com as telas dele (que já têm a marca, 15.4, e baixam o WebView2) e sai.

**Telas.**

- **Pronto para instalar:** o mascote parado, "Botloft", uma linha sobre o que ele é e o botão **Instalar**. Embaixo, "Instala só para você, sem pedir administrador." Em "Details", a pasta (`%LOCALAPPDATA%\Botloft`, ou a da instalação anterior) e a versão. O dono não escolhe pasta.
- **Já instalado**, lido da entrada em Aplicativos instalados (`HKCU\...\Uninstall\Botloft`: `DisplayVersion` e `InstallLocation`):
  - versão mais antiga: o botão vira **Atualizar**, com "Você tem a <versão>. Esta é a <versão>.";
  - a mesma versão: "O Botloft já está instalado.", com **Abrir o Botloft** e **Instalar de novo**;
  - versão mais nova: "Você já tem uma versão mais nova (<versão>).", só com **Abrir o Botloft**.
- **App aberto:** uma linha avisa que o Botloft fecha para instalar e que os bots continuam trabalhando, porque eles rodam no serviço (14). O setup fecha o `Botloft.exe` da pasta instalada antes de rodar o NSIS.
- **Instalando:** o mascote trabalhando, "Instalando o Botloft…" e uma barra sem porcentagem (o NSIS em silêncio não diz o progresso, e leva poucos segundos). O botão de fechar some até terminar, para a instalação não ficar pela metade.
- **Pronto:** "Tudo pronto." e, um instante depois, o setup abre o `Botloft.exe` da pasta em Aplicativos instalados e fecha. Se não conseguir abrir, diz para abrir o Botloft pelo menu Iniciar. Na primeira instalação, o app segue para o preparo de sempre (15.1).
- **Falhou:** o mascote cansado, "Não deu para instalar o Botloft.", **Tentar de novo** e **Usar o instalador clássico**, que roda o NSIS com as telas dele. Em "Details", o código de saída do NSIS.

**Arquivos.** O setup grava o instalador NSIS em `%TEMP%\Botloft-setup-<aleatório>\`, roda `/S`, espera e apaga a pasta, também quando falha. O instalador clássico também espera: a janela se esconde, o setup aguarda o NSIS terminar, apaga a pasta e sai. Fora a pasta do WebView2, o setup não escreve log, não usa a rede e não guarda nada. A instalação em si, a pasta e o registro são os do NSIS (5, 15.4).

**Build e release.**

- `pnpm bundle:setup` (em `app/`, `scripts/setup.mjs`) roda `vite build --mode setup`, que gera só a `setup.html` em `app/dist-setup`, e `cargo build -p botloft-setup --release --features tauri/custom-protocol` com `BOTLOFT_SETUP_PAYLOAD` apontando para o instalador NSIS que o `pnpm bundle` acabou de gerar. O resultado vai para `target\release\bundle\setup\Botloft_<versão>_x64-setup.exe`. Com `-- --rehearse`, sai sem o instalador dentro. O setup não usa o CLI do Tauri, porque não precisa de bundle: basta o executável.
- O release (15.5) passa a ter a tela de instalação como `Botloft_<versão>_x64-setup.exe`, o nome que o README já indica, e o instalador NSIS como `Botloft_<versão>_x64-update.exe` com o `.sig`, que é para onde o `latest.json` aponta. O `release.mjs stage windows` copia os dois com esses nomes para `target\release\bundle\release\`, escreve `windows.json` (os arquivos e a entrada do updater) e mostra os SHA-256; o `publish` junta o que cada sistema deixou lá num `latest.json`, sobe tudo e se recusa sem nada. O `stage` roda sozinho para conferir os arquivos sem criar release.
- O CI do app também roda `vite build --mode setup`, para a página do setup não quebrar só na hora do release.
- O setup não tem assinatura Authenticode, como o NSIS (15.5): o SmartScreen avisa ao abrir. O NSIS gravado na pasta temporária não tem a marca da internet e não passa de novo pelo SmartScreen.
- O executável fica maior, com a janela mais o NSIS inteiro: 15,4 MB na 0.5.0, contra 6,8 MB do NSIS sozinho.

**A verificar** (com o NSIS do Tauri 2.12; anotar aqui o resultado):

- `/S` por cima de uma versão instalada: se desinstala a anterior (como o `/P` faz pela página de reinstalação) ou só sobrescreve, e se algum arquivo antigo fica. **Mesma versão** (0.5.0 sobre 0.5.0, 2026-09-29): nenhum processo de desinstalador apareceu (olhando a cada 100 ms); o NSIS só sobrescreveu os arquivos. Falta ver com uma versão mais antiga (I3).
- `/S` com o app aberto: se o NSIS fecha o app sozinho, pergunta ou falha. O setup fecha o app antes (verificado: "Instalar de novo" com o app aberto fechou o app, instalou e abriu de novo), mas o comportamento do NSIS sozinho precisa ficar anotado.
- `/S` com uma versão mais nova instalada: se sai com erro (`silentDowngrades`) e com qual código.
- O atualizador aceita o instalador com o nome novo (`-update.exe`) e o roda como NSIS. **Na fonte** (`tauri-plugin-updater` 2.13, `extract_exe`): o tipo vem dos bytes baixados, e qualquer `.exe` é tratado como NSIS; o nome não conta. **Verificado** (2026-09-30): o app instalado na 0.5.0 se atualizou para a 0.6.0 pelo `-update.exe`, com a janela passiva do NSIS, e a entrada em Aplicativos instalados e a pasta ficaram as mesmas.
- Com o manifesto `asInvoker`, o Windows não pede administrador para o setup. **Verificado** (Windows 11 25H2, 2026-09-29): o `Botloft_0.5.0_x64-setup.exe` abriu a janela direto, sem pedido do UAC.
- O identificador dos atalhos continua o mesmo, para os avisos saírem com o nome do Botloft.

**Instalação real** (I1, Windows 11 25H2, 2026-09-29, 0.5.0): numa máquina com os arquivos do Botloft mas sem a entrada em Aplicativos instalados, a tela nova instalou em cerca de 3 s, criou a entrada (editor "Botloft", pasta `%LOCALAPPDATA%\Botloft`), abriu o app dessa pasta e apagou a pasta temporária. O daemon seguiu rodando, porque o instalador não mexe nele (15.4).

Um teste que instala uma cópia em outra pasta e depois a desinstala apaga a entrada da instalação de verdade, porque a entrada tem o nome do produto e não o da pasta; e deixa a pasta de teste como a lembrada (a chave do editor, 15.4). Foi o que aconteceu nesta máquina. Para testar numa pasta separada, use outra conta do Windows.

**Marcos.**

| Marco | Entrega | Pronto quando |
|---|---|---|
| **I1** Tela | crate `botloft-setup`, telas com `FakeSetup` e a prévia `?setup`, instalação pelo NSIS em silêncio, abrir o app ao terminar, modo de ensaio sem o instalador dentro | instalar pela tela nova numa máquina sem o Botloft e o app abrir; testes das telas e do `SetupHost` |
| **I2** Release | `pnpm bundle:setup`, `release.mjs` e `release.yml` com os dois arquivos; o README vai num PR próprio, junto com a 0.6.0 | um release rascunho com a tela de instalação, o `-update.exe`, o `.sig` e o `latest.json` certo |
| **I3** Acabamento | versão já instalada (mais antiga, igual, mais nova), app aberto, falha com o instalador clássico, sem WebView2 | cada caso testado à mão no Windows e anotado em "A verificar" |

Esta seção entra no PR do I1, junto com o código.

## 16. Qualidade

- Rust: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`.
- TS: `tsc --noEmit`, `biome check`, `vitest run`.
- Limite **flexível de 300 linhas** por arquivo; passou disso, dividir por responsabilidade.
- Toda lógica de supervisor, courier e chat testada com `FakeRuntime` (sem Claude real). O `FakeRuntime` fala `stream-json` de verdade: recebe as linhas do stdin e o teste escreve os eventos do stdout.
- CI: GitHub Actions em `windows-latest` (principal, workspace inteiro e app) e em `ubuntu-latest` e `macos-latest` para os membros padrão (`botloft-core`, `botloft-store`, `botloftd`), com clippy e testes. Os testes do navegador rodam com o Edge ou o Chrome da imagem. No Ubuntu, a imagem traz os auxiliares de sandbox deles sem setuid, e eles se recusam a abrir; o CI os deixa como os pacotes deixariam (`root`, `4755`). O crate do app (`botloft-app`) também passa por clippy e testes nesses runners; no Ubuntu, com as bibliotecas de desktop do Tauri (`libwebkit2gtk-4.1-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`, `libxdo-dev`). A janela de instalação (15.7) é só do Windows (18, item 8).

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

1. Rotinas: horário semanal, intervalo ou cron, com fuso, sobreposição e horários perdidos. Desenho na seção 20.
2. Caixa de perguntas ao owner (bot pergunta, owner responde, resposta volta como mensagem). Desenho na seção 23.
3. "Permitir sempre" nas aprovações, gravado como regra do bot. Feito (10.1).
4. Busca FTS5 em mensagens e no chat. Feito (8.8).
5. Histórico de versões das instruções do bot.
6. Acesso remoto com token por dispositivo (Tailscale).
7. Sinais entre bots disparando rotinas. Feito (20.13).
8. Suporte Linux/macOS. Em fatias: (1) CI com Ubuntu e macOS para o daemon; (2) runtime do bot (grupo de processos, ambiente, `claude`, navegador); (3) iniciar com o sistema (`systemd --user`, LaunchAgent); (4) app; (5) empacotamento e release. Feitas: 1 a 5 (14.1, 15.2, 15.4 a 15.6). O macOS sai sem notarização (sem conta Apple Developer por enquanto).
9. Navegador dos bots, com o dono assistindo ao vivo. Desenho na seção 21.

## 19. Pontos a verificar na versão alvo do Claude Code

Conferência na documentação oficial (code.claude.com/docs) em 2026-09-28; esforço, pedidos de controle, uso de contexto e compactação, em 2026-09-30; itens mais novos trazem a própria data. "Confirmado" quer dizer documentado; o teste real na versão alvo continua no checklist do marco indicado.

| Item | Seção | Resultado | Teste real |
|---|---|---|---|
| Formato de entrada `--input-format stream-json` | 9.2 | **Não documentado**. **Testado com 2.1.284**: `{"type":"user","uuid":...,"message":{"role":"user","content":[blocos]}}` abre um turno; bloco `image` em base64 é aceito | feito (M4.1) |
| Vários turnos num processo; mensagem durante um turno | 9.2 | **Testado com 2.1.284**: dois turnos seguidos no mesmo processo; a mensagem escrita durante um turno virou o turno seguinte. Cada turno começa com `system/init` e termina com `result`. **Testado de novo com 2.1.284** (Haiku, as flags do daemon): escrita enquanto o modelo escrevia uma resposta só de texto, a mensagem ganhou turno próprio (`system/init`, replay, `result`); escrita entre duas chamadas de ferramenta, voltou como replay logo depois do resultado de uma ferramenta, **dentro do mesmo turno**, sem `system/init` novo, e um só `result` (`num_turns: 5`) fechou as duas. Contar um turno por message escrita deixou um chefe `busy` para sempre num chat real; o supervisor conta pelos replays (7.2) | feito |
| Processo sem entrada | 7.2 | **Testado com 2.1.284**: fica calado e vivo até a primeira mensagem; sai com 0 quando o stdin fecha | feito (M4.1) |
| `--replay-user-messages` | 9.1 | Flag no `--help` sem detalhes na documentação. **Testado com 2.1.284**: devolve a mensagem com `isReplay: true` e o mesmo `uuid` quando o Claude Code a pega (no começo do turno dela ou no meio do turno em andamento, veja acima), não quando é lida. O `system/init` do turno vem antes do replay. Um `/compact` sem nada a compactar também devolve o comando com o `uuid` enviado | feito (M4.1) |
| Eventos do `--output-format stream-json` | 8.1 | Parcialmente documentado (`headless`): `system/init`, `stream_event` com `text_delta` (exige `--verbose` e `--include-partial-messages`), `system/api_retry` com `error` (`rate_limit`, `authentication_failed`...), `result`. **Visto com 2.1.284**: também `rate_limit_event` (status, `resetsAt`, uso das janelas de 5 h e 7 dias), `system/post_turn_summary`, `system/task_summary`, `system/thinking_tokens`; erro de API vem em `assistant.error` e `result.terminal_reason` | feito (M4.1) |
| `--permission-prompt-tool` | 10.1 | Confirmado (`cli-reference`, `headless`). **Testado com 2.1.284**: chama a tool com `{tool_name, input, tool_use_id}` e segue `{"behavior":"allow","updatedInput":...}`. Pelo daemon e pelo app: permitir depois de 95 s funcionou; negar com nota fez o bot citar a nota e não usar a ferramenta | feito (M4.1) |
| `description` na entrada do `Bash` e do `PowerShell` | 10.1 | Confirmado (`hooks`, entrada do `PreToolUse`): nas duas ferramentas, `description` é "Optional description of what the command does", ao lado de `command`, `timeout` e `run_in_background`. **Testado com 2.1.284** em 2026-09-30 (`-p` com uma tool de aprovação que só grava a entrada e nega; depois com a linha de comando de 7.4; Haiku 4.5, Sonnet 5.5 e o padrão do plano, Opus 5.5): a tool de aprovação recebe a mesma entrada do bloco `tool_use`, e os 32 pedidos de `Bash` e `PowerShell` vistos traziam `description`. Sem regra, o texto é curto e técnico ("Install requests package"); com o dono escrevendo em português, Haiku e Opus descreveram em português e o Sonnet em inglês, também pelo daemon. Com a regra de 10.1, os três escreveram em português, em palavras simples e com o porquê ("Instala o pacote requests, que ajuda o Python a acessar sites"). Pelo daemon e pelo app de dev (Sonnet, modo Manual): o cartão mostrou a explicação, "Ver o comando" abriu o script em heredoc inteiro, e permitir rodou esse comando; o log de `debug` não teve o texto. **O campo não é garantido**: em uso real com 2.1.284, dois bots em Sonnet fizeram 28 comandos sem `description` (o `tool_use` já vinha só com `command`), e os outros bots, quase todos em Opus, mandaram nos 15 que fizeram. A falta não se repetiu em teste, então não foi visto se a regra a evita: o cartão tem o caso sem explicação | feito; manual (PR): ver se bots antigos em Sonnet passam a explicar depois de reiniciar |
| `--permission-prompts host` | 10.1 | Documentado só para o SDK. **Visto com 2.1.284**: sem o aperto de mão do SDK, nega tudo (`system/permission_denied`). Não usado | não se aplica |
| `timeout` por servidor MCP | 10 | Confirmado (`env-vars`): HTTP tem 60 s por request e 5 min sem resposta por padrão; `timeout` >= 1000 no servidor sobe os dois. **Testado com 2.1.284**: a aprovação respondida depois de 95 s chegou ao bot | feito (M4.1) |
| Binário nativo na instalação do npm | 7.4 | **Visto com 2.1.287** (pacote `@anthropic-ai/claude-code` no npm): o `bin` do pacote é `bin/claude.exe`, um script de 500 bytes que o `postinstall` (`install.cjs`) troca pelo binário do pacote opcional da plataforma (`@anthropic-ai/claude-code-win32-x64`); com `--omit=optional` ou `--ignore-scripts`, o script fica. O `claude.cmd` e o `claude.ps1` do npm só chamam esse `.exe`. Numa instalação real no Windows, `bin\claude.exe --version` respondeu `2.1.287 (Claude Code)`, e o Botloft rodou com `claude_path` apontando para ele. Não documentado: o caminho pode mudar numa versão futura | feito; manual (PR): instalar só pelo npm e ver o Botloft achar o Claude Code sem `claude_path` |
| `--setting-sources project,local` | 7.4 | Confirmado (`cli-reference`). **Testado com 2.1.284**: sem os hooks, skills e agents do usuário; modo `default`; login da assinatura continua valendo | feito (M4.1) |
| `claude auth status` | 7.3 | Confirmado (`cli-reference`): JSON por padrão, sai com 0 conectado e 1 desconectado. **Testado com 2.1.284**: conectado traz `"loggedIn": true`; com `CLAUDE_CONFIG_DIR` vazio, `"loggedIn": false`, `"authMethod": "none"` e saída 1. As credenciais ficam em `%USERPROFILE%\.claude\.credentials.json` (`authentication`) | feito |
| `claude auth login` | 15.2 | Confirmado (`cli-reference`, `authentication`): abre o navegador e volta por um servidor local; se o navegador não alcançar esse servidor, mostra um código para colar no terminal. Se o subcomando sai sozinho depois do login ainda não foi visto: o app só depende do código de saída | manual (PR) |
| `permissionMode` no `system/init` | 7.4 | **Visto com 2.1.284**: o `system/init` de cada turno traz `permissionMode` com o valor da CLI (`default`, `acceptEdits`, `plan`, `auto`, `bypassPermissions`) | feito |
| Modos de permissão em `-p` | 7.4 | Confirmado (`permission-modes`): `deny` vale em todos os modos, inclusive `bypassPermissions`, que não pode ser ligado no meio da sessão; em `-p`, `auto` manda para a tool de aprovação o que o classificador não libera e `plan` continua bloqueando edições. O `auto` depende do plano e do modelo; o que `--permission-mode auto` faz sem ele ainda não foi visto | manual (PR) |
| `ExitPlanMode` em `-p` | 10.1 | A lista de ferramentas diz que ele pede permissão. Não visto: se o pedido chega à tool de aprovação com `{plan}` na entrada, e para qual modo o bot vai depois de aprovado (o daemon segue o `system/init` do turno seguinte) | manual (PR) |
| Modelo em `-p` | 7.4 | Confirmado (`model-config`, `sessions`): apelidos `fable`, `opus`, `sonnet`, `haiku` (e `best`, `opusplan`, `[1m]`, não usados); sem `model` nas settings, vale o padrão da conta; `--resume` mantém o modelo da sessão, a menos que `--model` escolha outro. **Visto com 2.1.284** (plano Max): sem `--model`, o `system/init` traz `"model": "claude-opus-5-5"`; um modelo inexistente sobe o processo, o init o repete, e cada turno termina com `assistant.error: "model_not_found"` e `result.is_error`; `/model haiku` mandado como mensagem troca o modelo no meio da sessão, com uma resposta sintética e sem custo, mas o daemon reinicia o processo em vez disso, como na troca de modo. Qual é o padrão em cada plano, e quais modelos cada plano tem, não está documentado | feito |
| `--add-dir` em `-p` | 5, 7.4 | Confirmado (`cli-reference`, `memory`): pastas a mais que o Claude Code trata como de trabalho; o `CLAUDE.md` delas só carrega com `CLAUDE_CODE_ADDITIONAL_DIRECTORIES_CLAUDE_MD=1`. **Testado com 2.1.284** (`--setting-sources project,local`, `acceptEdits`): com a flag e a variável, o bot citou o `CLAUDE.md` da pasta sem abrir arquivo e gravou nela sem pedir; sem a flag, o `Write` na mesma pasta foi negado (`permission_denials`). A flag aceita vários valores: a linha de comando do daemon não tem prompt posicional, então nada é engolido | feito |
| Chefe e `suggest_bot` com o Claude Code real | 10.2 | **Testado com 2.1.284** (daemon e app de dev): a crew criada pelo app subiu o chefe com `--model sonnet` e `--add-dir` na pasta da crew. Pedido um site, o chefe carregou `crew_roster`, `suggest_bot` e `send_message` pelo `ToolSearch` e sugeriu um Designer com papel, instruções, modelo e porquê; a chamada esperou o cartão. Com o modelo trocado para `haiku` no cartão, o bot subiu com `--model haiku`, e o chefe citou a troca. O chefe mandou uma task, o Designer entregou com `complete_task`, e o chefe conferiu o resultado e corrigiu o HTML. O log de `debug` não teve texto de mensagem | feito |
| Rotinas com o Claude Code real | 20 | **Testado com 2.1.284** (daemon e app de dev): uma rotina "a cada 5 minutos" criada pelo app, num chefe em Haiku, rodou às 09:10 e às 09:15, contadas a partir da criação; cada message chegou com o envelope da rotina, apareceu no chat com "Rotina · <nome>" e fechou como `done` com o `result` do turno. Desligada, não rodou às 09:20. O log de `debug` só teve ids | feito |
| Prévia de PDF no painel de arquivos | 15.1 | O PDF vai para um `<iframe src="blob:...">` (a CSP libera `frame-src blob:`). Não visto no WebView2 do app: no navegador de dev o painel funciona com imagem, markdown e texto | manual (PR): abrir um PDF no painel com `pnpm tauri dev` |
| Subagentes em segundo plano e turnos do próprio Claude Code | 7.2 | **Testado com 2.1.284** (`-p`, `stream-json`, Haiku, dois subagentes no `Agent`): o `result` do turno chega enquanto os subagentes ainda rodam. Eles avisam com `system/background_tasks_changed` (`tasks: [{task_id, task_type: "local_agent", description}]`, a lista inteira de quem ainda roda; `[]` no fim), `task_started`, `task_progress`, `task_updated` e `task_notification` (`status`, `summary`). Ao terminarem, o Claude Code abre turnos por conta própria (`system/init` sem `user` com `isReplay` antes, `assistant` e `result`), sem message escrita. Um turno assim já apareceu num chat real: o bot ficou `idle` durante ele. Comandos `local_bash` de dentro de um subagente aparecem em `task_started` com `owned_by_subagent` | feito |
| Regras `allow` do projeto em `-p` sem confiança | 7.4 | Confirmado (`permissions`): não são aplicadas numa pasta nunca confiada; `deny` vale sempre. Por isso `--allowedTools mcp__botloft` | M4.1 |
| Agendamentos do próprio Claude Code nos bots | 7.4, 20 | **Testado com 2.1.284** (`-p --setting-sources project,local --strict-mcp-config`): o `system/init` lista `CronCreate`, `CronDelete`, `CronList`, `ScheduleWakeup` e `RemoteTrigger` entre as 28 ferramentas; com `--disallowedTools` e os cinco nomes, eles saem da lista (23). Bots de verdade, pedidos para conferir algo todo dia, chamaram `CronCreate` sem pedir permissão, e a resposta da ferramenta avisava "Session-only (not written to disk, dies when Claude exits). Auto-expires after 7 days" | feito |
| `--session-id`, `--resume` em `-p` | 7.3 | Confirmado (`cli-reference`, `sessions`): a sessão retoma histórico e modelo; flags como `--mcp-config` têm de ser passadas de novo. **Testado com 2.1.284**: depois de reiniciar o bot e depois de reiniciar o daemon, o bot lembrou arquivos, a imagem e a mensagem de outro bot | feito (M4.1) |
| Quando a conversa passa a existir em disco; `--resume` de uma que não existe | 7.3 | Não documentado. **Testado com 2.1.284** (`-p`, `stream-json`, Haiku, sem MCP). `--resume` com um id que o Claude Code não tem escreve `No conversation found with session ID: <uuid>` no stderr e, no stdout, sem `system/init` antes, um `result` com `subtype: "error_during_execution"`, `is_error: true`, `num_turns: 0`, `duration_ms: 0`, `total_cost_usd: 0`, `errors: ["No conversation found with session ID: <uuid>"]` e sem `terminal_reason`; o processo sai sozinho com código 1, de 1,8 a 5,7 s depois de começar. A conversa só é gravada quando o primeiro turno começa: não foi retomada a de um processo com `--session-id` que ficou 6 s vivo sem mensagem, nem a de um morto ao imprimir o `system/init` do primeiro turno; mortos ao imprimir o `user` com `isReplay` (0,6 a 1,4 s depois do init), o primeiro `assistant` ou o `result`, os processos seguintes retomaram a conversa e ficaram calados à espera de mensagem. Pelo daemon de dev: `bots.create` e `bots.setPermissionMode` 20 ms depois subiram o segundo processo com `--session-id`, sem nada no chat; depois de um turno, a troca de modo retomou a conversa e o bot repetiu a palavra do turno anterior; com o transcript tirado do lugar e o daemon reiniciado, o `--resume` falhou, nada foi para o chat, o processo seguinte começou conversa nova e `bots.session_id` ficou vazio | feito |
| Tools MCP adiadas | 10 | **Visto com 2.1.284**: as tools do `botloft` chegam adiadas; antes da primeira `send_message` o bot chama `ToolSearch` com `select:mcp__botloft__send_message`. O chat mostra isso como "load send_message" | feito (M4.1) |
| Imagem inline na entrada | 9.5 | **Visto com 2.1.284**: o Claude Code guarda cada bloco `image` recebido em `%TEMP%\claude\<projeto>\<sessão>\images\<n>.png`, e o bot pode abrir essa cópia com `Read` sem pedir aprovação. O anexo original continua na pasta do bot | feito (M4.1) |
| Transcript `.jsonl` | 7.3 | Confirmado (`sessions`): formato interno, muda entre versões. O chat do Botloft não depende dele | não se aplica |
| Inbox entre sessões em `-p` | 9 | Documentado como indisponível em `-p`; o courier escreve no stdin | não se aplica |
| Carregamento de `.claude/rules/*.md` sem frontmatter | 5.1 | Confirmado (`memory`) e **testado com 2.1.283** (modo interativo) e **2.1.284** (`-p`): o bot respondeu nome, handle e crew tirados das regras | feito (M4.1) |
| Expansão `${VAR}` em headers do `mcp.json` | 10 | Confirmado (`mcp`); alguns nomes de credencial conhecidos são lidos vazios, `BOTLOFT_BOT_TOKEN` não é um deles. **Testado com 2.1.284**: o header chegou com o valor da variável de ambiente | feito (M3) |
| Revisão do MCP que o Claude Code usa | 10 | Especificação MCP 2026-07-28 (sem `initialize`) e versões antigas. **Visto com 2.1.284**: manda `server/discover` com os headers de 2026-07-28 e, se falhar, `initialize` com 2025-11-25; o mesmo num servidor stdio | feito (M3) |
| Sintaxe de caminho Windows em permission rules | 7.5 | Confirmado (`permissions`) e **testado com 2.1.283** (interativo) e **2.1.284** (`-p --setting-sources project,local`): `Read(//c/.../**)` em `deny` bloqueou a leitura com "File is in a directory that is denied by your permission settings", sem perguntar, e o `result` listou a negação em `permission_denials` | feito (M4.1) |
| Edge sem janela controlado por CDP | 21.2, 21.3 | **Visto com o Edge 154** (Windows 11): `--headless=new --remote-debugging-port=0` escreve `DevToolsActivePort` em ~400 ms; `Target.setAutoAttach` no alvo do navegador prende a aba que já existe e cada popup (`openerId`, esperando o depurador); `Emulation.setDeviceMetricsOverride` deixa a tela em 1280 × 800, e chamado de novo com a página aberta muda o `innerHeight` dela na hora; `Page.startScreencast` manda quadros JPEG com o tamanho da tela, também depois dessa mudança (com `maxHeight` no limite de 21.3), mas o quadro do tamanho novo se perde quando há quadros anteriores ainda sem `screencastFrameAck`, e uma página parada não manda outro: o daemon espera esses serem confirmados e recomeça o screencast (`stopScreencast`, `startScreencast`), que então manda um quadro; clique por `Input.dispatchMouseEvent` segue links; `alert` chega em `Page.javascriptDialogOpening`. `Target.targetInfoChanged` chega com o endereço novo da aba, mas com o endereço no lugar do título, e não vem de novo quando a página ganha o título. Ler o título com `Target.getTargetInfo` depois do `Page.loadEventFired` falhou de vez em quando nos testes (ficava o endereço); o `document.title` lido na página, num mundo isolado, não falhou. `Target.createTarget` com `about:blank` abre uma aba que chega por `attachedToTarget` como as outras; `Target.activateTarget` numa aba que ficou atrás a traz de volta, e o screencast dela volta a mandar quadros. Um Edge parado numa página que se mexe (um timer ocupado e uma animação) gastou 84% de um núcleo sem ninguém assistindo e 126% com o screencast, em ~450 MB e 7 a 10 processos. `Browser.setWindowBounds` com `windowState: "minimized"` na janela do `Browser.getWindowForTarget` deixou a página `hidden`, com o timer a menos de um por segundo e o processador abaixo de 1%; `normal` a trouxe de volta inteira, e `Page.captureScreenshot` funcionou em seguida. `Page.setWebLifecycleState` com `frozen` também parou a página, mas `active` a deixou `hidden` e lenta, e um `Page.bringToFront` depois disso travou o `Runtime.evaluate`: não serve. `Emulation.setCPUThrottlingRate` gastou mais, não menos. O user agent traz `HeadlessChrome`. Num popup esperando o depurador, `Page.enable` não responde até `Runtime.runIfWaitingForDebugger`; um mundo isolado com o mesmo nome (`Page.createIsolatedWorld`) volta o mesmo contexto até a página navegar, não enxerga os globais da página e lê `iframe` do mesmo site | feito |
| Imagem no resultado de uma tool MCP | 21.4 | Resultado com `{"type": "image", "data", "mimeType"}` em `content` (especificação MCP). **Testado com 2.1.284** (Haiku): pedido só pelas cores do logo no screenshot, o bot respondeu certo, e as cores não estavam no texto da página: o modelo recebe a imagem | feito |
| Tools do navegador com o Claude Code real | 21 | **Testado com 2.1.284** (daemon e app de dev, Haiku, modo Manual): o bot carregou `browser_open`, `browser_click`, `browser_look` e `browser_screenshot` pelo `ToolSearch`; o pedido de example.com apareceu no chat e, permitido, ele abriu a página e clicou no link, que levou ao iana.org; o `browser_screenshot` lá pediu o iana.org, e o segundo screenshot não pediu de novo. O painel mostrou a página e o cursor ao vivo. Pausar o bot fechou o navegador (os processos do Edge sumiram). O log de `debug` só teve nomes de tools, métodos e ids | feito |
| Rascunho de tela a partir do `Write` | 22.3 | **Visto com 2.1.284**: com `--include-partial-messages`, o `Write` chega em `stream_event` (`content_block_start` com `tool_use` e `name: "Write"`, depois `input_json_delta`; 379 pedaços para um arquivo de 3 KB). **Testado com 2.1.284** (daemon e app de dev, Haiku, modo Manual): pedida uma landing page, a área de design abriu sozinha e a página se montou versão a versão (umas 70 de 8 KB) antes do OK para gravar; permitido, a tela passou a vir do disco. Numa segunda página, negar o `Write` tirou a tela da área de design. O log de `debug` só teve status e ids. O painel do navegador do app de desktop do Claude bloqueia `iframe` para 127.0.0.1 (`ERR_BLOCKED_BY_CLIENT`), então a prévia foi vista num Edge sem janela | feito |
| Dono no controle com o Claude Code real | 21.10 | **Testado com 2.1.284** (daemon e app de dev, Haiku, modo Manual): pedido para abrir uma página de login local e chamar o dono, o bot carregou `browser_ask_owner` pelo `ToolSearch` sozinho, com a tarefa na língua do dono. Pelo cartão, o painel abriu já nas mãos do dono; clique, digitação, troca de campo e Enter chegaram ao Edge, e a página de login entrou na conta. Devolvido, a tool respondeu com a página nova e o bot listou os pedidos. A senha digitada não apareceu no log, no banco nem na conversa do Claude Code (o bot leu só o que a página mostrava). Numa página comprida, a roda do mouse sobre a tela ao vivo rolou a página do bot e não o painel | feito |
| Abas nas mãos do dono com o Claude Code real | 21.10 | **A verificar**: com um bot no meio de uma tarefa num site, o dono assume, abre uma aba nova em outro endereço e devolve. O bot deve ler o aviso de que a aba mudou (no erro da tool que ia agir, ou no `browser_look`), ler a aba nova e seguir a tarefa nela ou abrir de novo a página de que precisa, sem agir às cegas. Recarregar sem assumir: o bot lê "The owner reloaded the page." e continua | manual (PR) |
| Cursor do bot nas telas | 22.3 | **Testado com 2.1.284** (daemon e app de dev, Haiku, modo Manual), com o app num Edge sem janela: pedidas quatro páginas (floricultura, cafeteria, livraria, academia), o cursor seguiu o título, cada cartão, cada item de plano e cada linha de tabela enquanto o bot escrevia, e a prancheta rolou junto. Visto aqui: num `iframe` de outra origem, o script às vezes lê a página com `innerHeight` 0 e só depois recebe o tamanho (por isso a espera do `resize`); a versão escondida fala antes de ir para a frente (por isso vale a marca do quadro à mostra). O conteúdo das páginas não foi para o log | feito |
| Bot compartilha arquivos com `share_file` | 10, 8.4 | Ferramenta nossa, sem comportamento novo do Claude Code: a tool vem liberada por `--allowedTools mcp__botloft` e é carregada pelo `ToolSearch` como as outras. **Testado com 2.1.284** (daemon de dev, Haiku, modo Aceitar edições), só com a regra de 5.1 e a descrição da tool: pedido um relatório salvo na pasta de trabalho, o bot o escreveu e chamou `share_file` sozinho; pedido "me manda o relatório de novo", chamou de novo sem escrever o caminho na resposta. A primeira chamada veio com o caminho do Windows com `\` sem escape, que o próprio Claude Code recusa como JSON inválido; o bot repetiu certo. Por isso a descrição do campo pede barras normais. O conteúdo não foi para o log | feito |
| Esforço em `-p` | 7.4 | Confirmado (`cli-reference`, `model-config`, `headless`): `--effort` aceita `low`, `medium`, `high`, `xhigh`, `max` (e `ultracode`, não usado), vale só para a sessão e não é guardado; os níveis dependem do modelo (Fable, Opus e Sonnet atuais têm os cinco; Opus 4.6 e Sonnet 4.6 não têm `xhigh`); sem escolha, o padrão é `high`, menos em Opus 5.5 e Sonnet 5.5 (`medium`) e Opus 4.7 (`xhigh`); `/effort <nível>` funciona em `-p` e vale só para a sessão; as chaves `effortLevel` e `modelSettings` das settings não aceitam `max`; trocar o esforço no meio da sessão mantém o cache em Opus 5.5, Sonnet 5.5 e Fable 5.1 e o invalida nos outros (`prompt-caching`). **Testado com 2.1.284** (`--setting-sources project,local`, plano Max): `--effort max` e `xhigh` com `--model sonnet` são aplicados; um valor desconhecido só avisa no stderr e usa o padrão; `--resume` sem a flag volta ao padrão do modelo; sem a flag, Sonnet 5.5 e Opus 5.5 aplicam `medium` e Fable 5.1 `high`; Haiku 4.5 não tem níveis (o `initialize` do SDK lista `supportedEffortLevels` para os outros e nada para ele) e ignora a flag sem erro. `/effort low` mandado como mensagem troca na hora, com uma resposta sintética e sem custo, e o pedido de controle `apply_flag_settings {effortLevel}` também; o daemon reinicia o processo em vez disso. O que acontece com um nível que o modelo não tem (`xhigh` num modelo antigo) não foi visto: os apelidos do Botloft apontam para modelos com os cinco | feito |
| Pedidos de controle no stdin | 9.2 | `getContextUsage()` é documentado no Agent SDK (`agent-sdk/typescript`), com `totalTokens`, `maxTokens`, `rawMaxTokens`, `percentage`, `autoCompactThreshold` e `isAutoCompactEnabled`; o formato no fio e `get_settings` **não são documentados**. **Testado com 2.1.284**: `{"type":"control_request","request_id","request":{"subtype":"get_context_usage"}}` é respondido com `control_response` (`subtype: "success"`, o mesmo `request_id`, `response`) em até 2,5 s, antes do primeiro turno, entre turnos e durante um turno, sem `system/init` nem `result`; `get_settings` responde `{effective, sources, applied: {model, effort, ...}}`, com `effort: null` em Haiku; um `subtype` desconhecido responde `subtype: "error"`. Numa sessão retomada, a primeira resposta levou uma vez mais de 15 s. **Pelo daemon de dev** (Haiku e Sonnet): o modelo e o esforço do modelo chegaram ao app antes do primeiro turno, e `--model sonnet --effort low` apareceu na linha de comando do bot depois de `bots.setEffort` | feito |
| Uso de contexto no `stream-json` | 8.6 | Parcialmente documentado: `usage` e `modelUsage` no `result` (`agent-sdk/typescript`), janela de 1 milhão em Fable, Sonnet 5+ e Opus 4.7+ e compactação automática em cerca de 967 mil (`model-config`). **Visto com 2.1.284**: cada `assistant` traz `message.usage` (`input_tokens`, `cache_creation_input_tokens`, `cache_read_input_tokens`, `output_tokens`), e a soma dos três primeiros bate com o `totalTokens` do `get_context_usage` a menos de 1%; `result.modelUsage[modelo]` traz `contextWindow` (200000 em Haiku 4.5, 1000000 em Sonnet 5.5) e `maxOutputTokens`; `result.usage.iterations[]` traz cada pedido do turno; `autoCompactThreshold` é a janela menos 33 mil (167000, 967000; 67000 com `--autocompact 100k`, que também encolhe `maxTokens`); `/context` como mensagem devolve um `assistant` sintético com `context_usage` e abre um turno, por isso o daemon usa o pedido de controle. `total_cost_usd` é o total do processo, não do turno. **Pelo daemon de dev**: o app recebeu o tamanho ao subir o bot, a cada pedido ao modelo e no fim do turno | feito |
| Tokens do turno no `result` | 8.7 | Parcialmente documentado: `usage` no `result` (`agent-sdk/typescript`), sem dizer se é do turno ou do processo. **Testado com 2.1.284** (Haiku, um processo com dois turnos): `result.usage` é a soma dos pedidos do turno (um turno com um `Read` deu `input_tokens` 18 = 10 + 8 e `cache_read_input_tokens` 46703 = 21692 + 25011, a soma de `iterations[]`) e recomeça a cada turno; `modelUsage` e `total_cost_usd` acumulam o processo. `output_tokens` inclui `output_tokens_details.thinking_tokens`. **Visto numa instalação real** (Sonnet, 2026-09-30): o primeiro turno de um bot sete horas depois do anterior, e três minutos depois de o Botloft reiniciar, levou 11 s e deu `input_tokens` 2, `cache_creation_input_tokens` 72636, `cache_read_input_tokens` 0 e `output_tokens` 983: a conversa inteira gravada de novo no cache, que levou à conta de `reloaded` (8.7). Não verificado: se os tokens de subagentes entram nesse `usage`; a conta de `reloaded` com teste real, num turno de cache quente (deve dar 0) e num depois de o cache expirar | feito |
| Compactar em `-p` | 8.6 | `/compact [instruções]` é documentado (`commands`) sem dizer se vale em `-p`; o custo com cache quente e frio está em `prompt-caching`. **Testado com 2.1.284** (Haiku): `/compact` como mensagem imprime `system/status` (`status: "compacting"`), outro com `status: null` e `compact_result: "success"`, `system/init`, `system/compact_boundary` (`compact_metadata: {trigger: "manual", pre_tokens, post_tokens, duration_ms, ...}`), um `user` com `isSynthetic` e o resumo, dois `user` com `isReplay` (a saída `Compacted` e o comando, com o `uuid` enviado) e um `result` com `local_command: "compact"`, `num_turns: 0` e custo; levou de 12 a 16 s numa conversa pequena, e o bot lembrou depois o que tinha sido dito antes. Escrito durante um turno, roda quando o turno termina. Numa conversa vazia responde um `assistant` sintético "Error: No messages to compact" com `local_command_outcome: {kind: "failed"}` e um `result` sem erro. **Compactação automática** (Haiku, `--autocompact 100k`, quatro arquivos lidos): no meio do turno, os dois `system/status`, o `compact_boundary` com `trigger: "auto"` e o `user` sintético, sem `system/init` nem `result` próprios; o turno seguiu e terminou certo. **Pelo daemon de dev**: `bots.compact` deixou o bot `busy`, o aviso `compacted` entrou no chat, o tamanho caiu e o bot respondeu depois a palavra que tinha guardado; o log de `debug` só teve ids e números. Uma compactação que falha por erro da API não foi vista | feito |
| CLAUDE.md do dono nos bots | 5, 7.5 | Confirmado (`memory`, `settings-reference`): o Claude Code carrega `CLAUDE.md`, `CLAUDE.local.md` e `.claude/CLAUDE.md` do cwd e de toda pasta acima dele; `claudeMdExcludes` pula arquivos por caminho absoluto ou glob, vale em qualquer camada de settings e também para `AGENTS.md`; o `CLAUDE.md` gerenciado não pode ser excluído. **Testado com 2.1.284** (`-p --setting-sources project,local`, `memoryFiles` do `get_context_usage`, numa pasta de teste dentro de `%USERPROFILE%`). Sem a setting: o `%USERPROFILE%\.claude\CLAUDE.md` do dono entra como `Project`, e de cada pasta acima entram `CLAUDE.md`, `CLAUDE.local.md`, `.claude/CLAUDE.md` e `.claude/rules/` com as subpastas; as skills são só as embutidas. Forma do padrão: `C:/Users/...` e `C:\Users\...` casam, também misturadas; `/c/Users/...` e `//c/Users/...` não casam; a comparação diferencia maiúsculas, até na letra do drive, e o Claude Code monta os caminhos com o cwd como o recebeu (um cwd em minúsculas aparece em minúsculas). Numa pasta chamada `odd (x) [y] {z}`, o caminho exato de um arquivo ainda casa, mas `.../.claude/rules/**` não; com `?` no lugar de cada um desses seis caracteres, casa. Espaço, acento e ``!+@#$%^&',;=~`` casam como estão. O `AGENTS.md` de uma pasta acima só entra com `instructionFiles: claude-md-and-agents-md` (os bots sempre têm `CLAUDE.md`), e o padrão o tira. `--add-dir` carrega `CLAUDE.md` e `.claude/rules/` só da própria pasta: os das pastas acima de uma pasta de trabalho escolhida não entram, com ou sem a setting. **Pelo daemon de dev** (workspaces dentro de `%USERPROFILE%`, com `CLAUDE.md` de teste em cada pasta acima; Haiku): com o `settings.json` gerado, `memoryFiles` e o próprio bot listaram só o `CLAUDE.md` da pasta da crew, o do bot, as regras e o da `shared\` | feito |
| Memória automática do Claude Code nos bots | 7.5 | Confirmado (`memory`, `settings-reference`): ligada por padrão; o Claude Code grava `MEMORY.md` e um arquivo por nota em `%USERPROFILE%\.claude\projects\<projeto>\memory\` e carrega o índice em toda sessão; `<projeto>` vem do repositório git ou, fora de um, do caminho do cwd; `autoMemoryEnabled: false` vale em qualquer settings. **Testado com 2.1.284** (`-p --setting-sources project,local`, Haiku): ligada também assim, com cerca de 3 mil tokens a mais no prompt de sistema (6457 contra 3495). A pasta `memory\` é criada quando a sessão sobe. Pedido para lembrar de uma coisa, o bot gravou a nota e o `MEMORY.md` nessa pasta, fora do workspace, sem pedido de permissão no modo `default`, e não tocou no `CLAUDE.md` dele. O nome de `<projeto>` é o caminho com `-` no lugar de tudo que não é letra nem número (`C--Users-ana-Botloft-site-revisor`); por essa regra `site-web\dev` e `site\web-dev` dariam a mesma pasta (não testado). Com o cwd dentro de um repositório git, a sessão carregou como `AutoMem` o `MEMORY.md` que o Claude Code guarda para o dono naquele repositório. Com `autoMemoryEnabled: false` no `settings.json` do projeto, o prompt encolhe e nada disso é carregado, nem dentro do repositório. **Pelo daemon de dev** (Haiku, "Aceitar edições"): pedido para lembrar, o bot editou o `CLAUDE.md` do workspace, e nenhuma pasta `memory\` foi criada. Mover a memória automática para dentro do workspace (`autoMemoryDirectory`) não foi testado | feito |
| Telas no WebView2 do app | 22.5 | Não visto: a CSP do Tauri com `frame-src http://127.0.0.1:*` e as telas carregando no `iframe` | manual (PR): `pnpm tauri dev`, pedir uma página HTML a um bot e ver a área de design |

Itens do runtime anterior (ConPTY, hooks em exec form, `crossSessionInbound`, linha de auth do inbox, diálogo de confiança, `ESC[6n` do ConPTY, consultas do terminal no replay) foram verificados no M2–M4 e deixaram de se aplicar com a ADR 0001; o histórico está no git e na ADR.

## 20. Rotinas

Status: primeiro item depois do MVP (seção 18), em dois marcos (20.11), **implementados**.

### 20.1 O que é

Uma rotina faz um bot trabalhar sozinho num horário ou num intervalo. Na hora marcada, o daemon põe na conversa do bot uma message com o pedido que o dono escreveu, e ela segue o caminho de qualquer message (9.1): gravada antes, entregue pelo courier, com retry. Não há processo nem sessão à parte: o bot responde no mesmo chat, com a mesma memória, e o que ele faz aparece como sempre. Isso mantém o princípio 3 (uma conversa por bot).

Exemplos: "todo dia útil às 9h, resuma o que chegou em `shared/inbox`"; "a cada 2 horas, confira se o site responde e avise o @deploy se não".

### 20.2 Quando roda

O horário fica guardado como JSON estruturado (`schedule`), e não como texto cron, para o app mostrar e editar sem jargão:

| `kind` | Campos | No app |
|---|---|---|
| `weekly` | `days` (1 = segunda … 7 = domingo, ao menos um), `time` (`HH:MM`) | "Todo dia às 09:00", "Dias úteis às 09:00", "Segunda e quinta às 14:30" |
| `interval` | `minutes` (de 5 a 10 080) | "A cada 2 horas" |
| `cron` | `expr` (5 campos, sem segundos) | só em "Avançado"; o app mostra a expressão como está |
| `signal` | `name` (o aviso, normalizado) | "Quando um bot avisar “relatorio-pronto”"; roda quando um bot da crew manda o aviso (20.13) |

- Horários de calendário (`weekly`, `cron`) valem no fuso da rotina (`timezone`, nome IANA como `America/Sao_Paulo`). O app manda o fuso do sistema ao criar (`Intl.DateTimeFormat().resolvedOptions().timeZone`). Mudar o fuso do Windows depois não mexe em rotinas existentes.
- `interval` conta a partir do último horário marcado, e não do fim do trabalho: `próximo = último marcado + minutes`, ancorado na criação. Não deriva.
- Horário que não existe (o relógio pula na entrada do horário de verão): roda no primeiro instante válido depois do pulo. Horário que acontece duas vezes (saída do horário de verão): roda uma vez, na primeira.
- Espaçamento mínimo de 5 minutos, também para `cron`: o daemon confere as próximas 20 ocorrências e recusa com erro de validação. Rotina frequente demais gasta o plano do dono sem ele perceber.
- `cron`: 5 campos (minuto, hora, dia do mês, mês, dia da semana). Com dia do mês e dia da semana restritos, basta bater um dos dois, como no cron clássico.
- Fusos com `jiff`, que traz a base IANA embutida (o Windows não tem uma). O daemon avalia o próprio cron: anda pelos dias no fuso da rotina e resolve cada hora de parede pelas regras acima; no pulo, acha o instante exato da mudança de offset. Testado com o pulo e a repetição de Nova York em 2026, com o fuso de São Paulo e de Lisboa, e com `0 0 29 2 *`, que só roda em 2028.

### 20.3 Execuções

Cada disparo vira uma `routine_run`, com o horário marcado (`scheduled_for`), um `status` e a message que gerou.

| `status` | Quando |
|---|---|
| `queued` | a message foi gravada e espera a entrega ou o fim do turno dela |
| `done` | o turno que começou com essa message terminou |
| `failed` | a delivery morreu (`dead`) ou o turno terminou com erro |
| `skipped` | não disparou; `reason`: `overlap`, `bot_paused`, `missed` ou `too_soon` (um aviso menos de 5 minutos depois do anterior, 20.13) |

- **Fim de uma execução:** o daemon já sabe quando o bot pega uma message (o replay com o mesmo `uuid`, 9.1 passo 7). O `result` seguinte fecha o turno em que ela entrou, também quando outra message entrou no mesmo turno: a execução vira `done`, ou `failed` se o `result` trouxer erro. Se o processo morrer antes de ler a message, a delivery volta para a fila (9.1 passo 8) e a execução continua `queued`; se morrer no meio do turno, a execução vira `failed`, porque ninguém mais vai fechar aquele turno. Pelo mesmo motivo, ao subir, o daemon marca `failed` as execuções cujo turno começou sob o daemon anterior. Uma delivery `dead` também leva a execução a `failed`.
- **Sobreposição:** o horário chega com a execução anterior ainda `queued` (o bot está lento, parado por limite de uso, sem login ou fora do ar).
  - `skip` (padrão): registra `skipped` com `reason: overlap`. Um bot lento ou fora do ar não acumula pedidos repetidos.
  - `queue`: grava mesmo assim, mas só uma execução espera atrás da aberta; as outras viram `skipped`.
- **Bot ou crew pausados:** o horário vira `skipped` (`bot_paused`); a rotina não guarda pedidos para quando voltar. Bot ou crew arquivados: a rotina é arquivada junto. Excluídos: a rotina e as execuções são apagadas (7.6).
- **Rodar agora:** `routines.runNow` cria uma execução fora de hora (`scheduled_for` = agora), sem mexer no próximo horário, e vale a mesma regra de sobreposição.

### 20.4 Horários perdidos

Com o daemon parado, o PC dormindo ou ninguém logado, nada dispara. Na volta (no boot do daemon e a cada ciclo, comparando `next_run_at` com agora):

- `missed: run_once` (padrão): se passou algum horário, roda **uma vez** agora, com `scheduled_for` = o último horário perdido. Os outros viram uma só entrada `skipped` (`missed`) com a contagem (`skipped_count`). Uma rotina diária com o PC desligado por uma semana roda uma vez ao ligar, e não sete.
- `missed: skip`: nada roda; registra o `skipped` e segue para o próximo horário.
- Um horário conta como no horário até 2 minutos de atraso (o agendador acorda ao menos uma vez por minuto); depois disso, conta como perdido.
- Religar uma rotina desligada conta a partir de agora: o que passou com ela desligada não é perdido.
- O PC não acorda para rodar rotina (a tarefa agendada não usa `WakeToRun`), e o app diz isso nas opções.

### 20.5 O que o bot recebe

A message tem `kind: routine` e `from_kind: system`, com um envelope em inglês como o dos outros remetentes (9.3):

```
[botloft] routine "Resumo da manhã" · scheduled 2026-10-01 09:00 (America/Sao_Paulo)
Nobody is watching live: do the work, then report it in your reply.

<pedido do dono>
```

- O horário do envelope é absoluto, no fuso da rotina: o bot não sabe a hora atual.
- O pedido tem a autoridade do dono, que o escreveu, mas chega com envelope para o bot saber que é automático e que ninguém está olhando naquela hora.
- No chat, o item `inbound` mostra o nome da rotina. Na lista de conversas, a linha é `kind: message`.

### 20.6 Agendador

- Módulo `routines/` no daemon, com o relógio injetável (`Clock`) como o courier. Ele dorme até o `next_run_at` mais próximo, por no máximo 60 s (para acompanhar mudança do relógio e a volta do sono), ou até ser acordado (rotina criada, editada, ligada ou desligada; bot pausado).
- `next_run_at` fica gravado e é recalculado a cada disparo, edição e volta de horário perdido. O cálculo usa a hora de parede no fuso da rotina, nunca o relógio monotônico.
- Disparar é uma transação: `routine_run`, `message`, `delivery` e o item `inbound`, como no `messages.send`. Depois o courier é acordado.
- Testes com `FakeRuntime` e relógio manual: `weekly`, `interval` e `cron`; fuso; horário de verão (pulo e repetição); horários perdidos; sobreposição; pausa; rodar agora; fim da execução pelo `result`.

### 20.7 Dados

Migration nova:

| Tabela | Colunas |
|---|---|
| `routines` | `id` (`rtn_`), `bot_id`, `name`, `prompt`, `schedule` (JSON), `timezone`, `overlap` (`skip`, `queue`), `missed` (`run_once`, `skip`), `enabled`, `next_run_at`, `created_at`, `updated_at`, `archived_at` |
| `routine_runs` | `id` (`rrn_`), `routine_id`, `scheduled_for`, `status`, `reason`, `skipped_count`, `message_id`, `created_at`, `finished_at` |
| `messages` | `kind` ganha `routine`; coluna nova `routine_id` |
| `routine_runs` (0017) | `signal_name`, `signal_from`, `signal_note`: o aviso que fez a execução rodar (20.13) |

Índices: `routines(enabled, next_run_at)` e `routine_runs(routine_id, id)`.

### 20.8 Protocolo

| Método | Params | Result |
|---|---|---|
| `routines.list` | `botId?` | `Routine[]`, com `nextRunAt` e a última execução |
| `routines.create` | `botId, name, prompt, schedule, timezone, overlap?, missed?` | `Routine` |
| `routines.update` | `routineId` e os mesmos campos, opcionais | `Routine` |
| `routines.setEnabled` | `routineId, enabled` | `Routine` |
| `routines.runNow` | `routineId` | `RoutineRun` |
| `routines.archive` | `routineId` | `Routine` |
| `routines.runs` | `routineId, before?, limit?` | `RoutineRun[]`, mais nova primeiro |

- Notificações: `routine.changed` e `routine.run`.
- Validação (`-32004`): nome de 1 a 80 caracteres; pedido dentro do limite de uma message; `days` não vazio e de 1 a 7; `time` válido; `minutes` de 5 a 10 080; `cron` válido e com espaçamento de pelo menos 5 minutos; `timezone` conhecido. Os problemas do horário vêm com `data.reason` (`timezone_unknown`, `days_empty`, `days_range`, `time_invalid`, `interval_range`, `cron_invalid`, `too_often`, `never_runs`, `signal_invalid`), que o app escreve no idioma do dono (15.6); a mensagem em inglês fica para quem não conhece o código.

### 20.9 App

Sem jargão (15.2): o dono não vê "cron", "overlap" nem "timezone" no caminho principal.

- No bot, abas **Conversa** e **Rotinas** (com a contagem, "Rotinas (2)"). A lista: um interruptor para ligar e desligar, o nome, quando ("Dias úteis às 09:00"), a próxima vez no fuso da rotina ("amanhã às 09:00"; desligada, "Desligada"), como foi a última (rodando, rodou bem, falhou, ou pulada e por quê), "Rodar agora" e um menu com editar e apagar (com confirmação). Sem rotinas, uma explicação e "Nova rotina".
- Criar e editar: "Nome", "O que <bot> deve fazer?" e "Quando": todo dia, dias úteis, dias escolhidos (os sete dias como botões, com os nomes do idioma), ou a cada N minutos ou horas; para os três primeiros, o horário. Nomes de dias e horas vêm do `Intl`, no idioma do app.
- Em "Mais opções":
  - o fuso (o do sistema por padrão, mostrado pelo nome da cidade);
  - "Se a anterior ainda não terminou": pular ou esperar a vez;
  - "Se o computador estava desligado na hora": rodar quando ligar, ou pular;
  - "Avançado": expressão cron.
- No chat, a message da rotina aparece com a etiqueta "Rotina · <nome>".
- Na página da crew, uma aba com as rotinas de todos os bots dela.
- Uma execução `failed` entra na marca da barra de tarefas (15.2) até o dono abrir o bot. O app guarda quando o dono abriu cada bot (`localStorage`, `botloft.seen`); um bot aberto não marca.
- O store carrega `routines.list` a cada conexão e segue `routine.changed` e `routine.run`.
- Textos nos três idiomas (15.6); a frase de "quando" é montada pelo app a partir do `schedule`.

### 20.10 Fora desta etapa

- Sinais entre bots disparando rotinas (seção 18, item 7). Feito depois, em 20.13.
- Notificação do Windows quando uma rotina termina ou falha.
- Acordar o PC para uma rotina.

### 20.11 Marcos

| Marco | Entrega | Pronto quando |
|---|---|---|
| **R1** Agendador | migration, `routines/`, courier com `kind: routine`, fim de execução pelo `result`, RPC e notificações, testes com relógio manual | com `FakeRuntime`, rotinas `weekly` e `interval` disparam no horário, pulam por sobreposição e rodam uma vez depois de horário perdido |
| **R2** App | aba Rotinas no bot e na crew, editor, etiqueta no chat, marca na barra de tarefas, textos nos três idiomas | criar pelo app uma rotina "a cada 5 minutos", ver duas execuções com o Claude Code real, desligá-la e ver que para |

### 20.12 Rotinas pedidas pelos bots

Pedido para fazer algo em horário marcado ("confere o e-mail todo dia às 8"), um bot usava o agendador do próprio Claude Code (`CronCreate`), que morre com o processo e não aparece no app (7.4, 19). Agora esses agendadores ficam desligados, e o bot pede uma rotina de verdade pela tool `schedule_routine`, que o dono aprova no chat, como a sugestão de bot (10.2).

- **Entrada:** `name`, `prompt` (o pedido de cada vez, escrito para o bot que vai rodar, até 8 000 caracteres para caber no cartão), `schedule` (o mesmo JSON de 20.2) e, opcionais, `bot` (handle de outro bot da crew, para quem a rotina é) e `timezone`. Sem `timezone`, vale o fuso do computador, como no app: o daemon o lê do Windows pelo `jiff` (`UTC` se o Windows não der um nome IANA). O daemon grava na entrada o fuso usado e o handle normalizado.
- **Antes do dono:** o daemon confere tudo o que `routines.create` confere (20.8), mais o limite do pedido e que o `bot` existe na crew. Um pedido impossível (espaçamento menor que 5 minutos, fuso desconhecido, campo a mais) volta como erro ao bot, sem cartão.
- **Aprovação:** `approval` com `toolName: "mcp__botloft__schedule_routine"`, entrada até 64 KB e resumo com o nome da rotina, no chat do bot que pediu, mesmo quando a rotina é de outro bot. O bot fica `needs_approval`, e a chamada espera até `approval_timeout_minutes`. Qualquer bot pode pedir, não só o chefe.
- **Cartão:** nome, "O que <bot> deve fazer?", "Quando" e "Mais opções", os mesmos campos do editor (20.9), já preenchidos. Para outro bot, o título diz para quem é. "Criar rotina" permite; "Agora não" nega com a nota do dono. Mudado algum campo, `approvals.answer` leva `input` com a rotina como o dono deixou. O daemon confere esse `input` antes de fechar o pedido, e um problema de horário volta com o `data.reason` de 20.8: o cartão o escreve no idioma do dono e continua aberto. O bot e o fuso do pedido valem se o `input` não trouxer outros.
- **Resposta ao bot:** criada, o id, o que ficou gravado (o dono pode ter mudado), o fuso e a próxima vez na hora local, e o lembrete de que ela está na aba Rotinas. Negada ou sem resposta, que nada foi agendado e que não deve dizer que foi.
- **Bot em `bypass_permissions`:** cria na hora, sem cartão (13).
- **Regras do bot** (5.1): trabalho em horário marcado é `schedule_routine`; é a única forma de agendar, e o bot só diz que agendou depois que a tool diz que criou.
- Mudar, pausar e apagar rotinas continua sendo só do dono, pelo app.

### 20.13 Sinais entre bots

Uma rotina pode esperar um **aviso** (sinal) em vez de um horário: ela roda quando um bot da equipe diz que algo aconteceu. Exemplo: o @writer salva o relatório e avisa `relatorio-pronto`; a rotina "Revisar o relatório" do @revisor roda na hora, sem o dono no meio e sem o @writer precisar saber quem revisa.

- **Agendamento.** Um `kind` novo em `schedule` (20.2): `{"kind": "signal", "name": "relatorio-pronto"}`. O nome é guardado normalizado, como um slug (minúsculas, números e hífens, sem acento, até 32 caracteres: "Relatório Pronto" vira `relatorio-pronto`); sem letra nem número, é `validation` com `data.reason: signal_invalid`. A rotina não tem horário: `next_run_at` fica `null`, o agendador nunca a dispara, e `missed` e o fuso não valem para ela. "Rodar agora" e ligar e desligar valem como em qualquer rotina.
- **Mandar.** A tool `send_signal {name, note?}` (10): o nome é normalizado igual, e a nota (até 2 000 caracteres) vai junto para quem roda. Para cada rotina ligada de um bot ativo da **mesma crew** que espera aquele nome, o daemon tenta uma execução agora, pelas regras de 20.3: bot ou crew pausados viram `skipped` (`bot_paused`), e a sobreposição vale como sempre. O próprio bot que avisa também pode ter uma rotina esperando o aviso.
- **Espaçamento.** Uma rotina roda por aviso no máximo uma vez a cada 5 minutos, como o espaçamento dos horários (20.2): um aviso que chega antes disso vira `skipped` com `reason: too_soon`. Assim dois bots que se avisam em círculo não gastam o plano do dono.
- **Resposta ao bot.** O nome normalizado e, para cada rotina alcançada, o nome, o bot e se rodou, com o porquê quando não rodou. Nenhuma rotina esperando não é erro: a resposta diz que nada rodou.
- **O que o bot que roda recebe** (9.3): `[botloft] routine "<nome>" · signal "<aviso>" from @writer` (ou `from a deleted bot`), a linha de que ninguém está olhando, o pedido do dono e, se veio nota, a nota depois, marcada como escrita por um bot e não pelo dono.
- **Execução.** `RoutineRun.signal` guarda `{name, fromBotId, note}`; `routine_runs` ganha `signal_name`, `signal_from` (sem chave estrangeira: um bot excluído só deixa o id) e `signal_note` (migration 0017).
- **Achar os avisos.** `crew_roster` lista em `signals` cada aviso que uma rotina da crew espera, com a rotina e o bot. As regras do bot (5.1) dizem para mandar o aviso quando o que ele nomeia estiver feito. `schedule_routine` (20.12) aceita o mesmo `kind`, então um bot pode pedir ao dono uma rotina por aviso.
- **App.** Em "Quando", a opção "Quando um bot avisar", com o campo "Aviso" e a explicação de que o dono deve dizer ao outro bot, nas instruções dele, quando avisar. Em "Mais opções" sobra só a regra de sobreposição. Na lista: "Quando um bot avisar “relatorio-pronto”" e "Espera um aviso" no lugar da próxima vez; a última execução diz de quem veio o aviso ("aviso de Writer"), e um aviso cedo demais aparece como pulado.
- **Fora desta etapa:** escolher de qual bot o aviso precisa vir; avisos entre crews; o dono mandar um aviso pelo app.

## 21. Navegador

Status: **N1, N2, N3, N3.1, N3.2 e N3.3 implementados** (21.11); N4 é a seção 22.

### 21.1 O que é

Cada bot tem um navegador próprio para pesquisar e usar sites: um Microsoft Edge sem janela (`--headless=new`), com um perfil só dele, que o daemon controla pelo Chrome DevTools Protocol (CDP). O bot o usa pelas tools `browser_*` do MCP (21.4). O dono vê a tela ao vivo num painel ao lado do chat, com o cursor do bot onde ele clica (21.8).

- **Por que o Edge:** vem com o Windows 10 e 11, o Windows Update o mantém em dia, é Chromium e fala CDP. Nada para instalar (sem Node, sem Playwright). `[browser] path` na config aponta outro Chromium (Chrome, por exemplo).
- **Por que no daemon:** o navegador é do bot, não do app. Fechar o app não fecha a página em que o bot trabalha (princípio 1); o app só assiste.
- O Claude Code já tem `WebSearch` e `WebFetch` para buscas e leituras rápidas; o navegador é para o que precisa de uma página de verdade: clicar, preencher, rolar, ver como ficou.

### 21.2 Processo

- Sobe na primeira tool `browser_*` do bot e fica aberto enquanto ele o usa. Um processo por bot.
- Comando: `msedge.exe --headless=new --remote-debugging-port=0 --user-data-dir=<home>\browsers\<bot_id> --no-first-run --no-default-browser-check --mute-audio --disable-extensions --disable-sync about:blank`. Sem `--disable-extensions`, um perfil novo recebe as extensões instaladas para todo o computador, e elas abrem abas próprias no navegador do bot (visto com o Edge 154). Com a porta 0, o Edge escolhe uma porta livre em 127.0.0.1 e a escreve com o caminho do WebSocket em `DevToolsActivePort`, na pasta do perfil (visto com o Edge 154); o daemon apaga o arquivo antigo antes de subir e espera o novo por até 30 s. Costuma levar menos de 1 s, mas a primeira abertura do Edge no runner Ubuntu do CI passou de 20 s. Se o navegador fecha ou não fica pronto, o erro traz a linha do stderr dele que melhor explica (uma falha de sandbox ou de namespace, senão a primeira fatal, senão a última); o stderr não vai para o log.
- Como o `claude.exe`: sem janela (`CREATE_NO_WINDOW`), num Job Object próprio com `KILL_ON_JOB_CLOSE` (morre junto com o daemon) e com o ambiente padrão do usuário (7.4).
- Achar o Edge: `[browser] path`, senão `msedge.exe` em `%ProgramFiles(x86)%` e `%ProgramFiles%` (`Microsoft\Edge\Application`) e em `%LOCALAPPDATA%`. No macOS, Edge, Chrome ou Chromium em `/Applications` e em `~/Applications`; no Linux, o primeiro de `google-chrome-stable`, `google-chrome`, `chromium`, `chromium-browser`, `microsoft-edge-stable` e `microsoft-edge` no `PATH` do ambiente do dono (14.1). O Edge vem por último no Linux: é o menos comum lá, e no runner Ubuntu do CI ele levou segundos para abrir a primeira aba de cada perfil novo (passando dos 5 s de espera da primeira aba), onde o Chrome levou menos de 1. Fora do Windows, o grupo de processos faz o papel do Job Object (14.1). Sem navegador, a tool responde ao bot que ele não está disponível, e o painel diz o mesmo.
- **Perfil:** `<home>\browsers\<bot_id>\`, fora da pasta do bot e em `%LOCALAPPDATA%` (5): cookies e sessões são do navegador, não dos arquivos que o bot faz. O perfil fica entre aberturas, então um login feito continua valendo. Arquivar ou excluir o bot fecha o navegador e apaga o perfil.
- **Descansar:** um navegador aberto que ninguém usa não deve gastar o computador. Assim que o turno do bot termina (o estado dele deixa de ser `busy` ou `needs_approval`, 7.1), o daemon põe o navegador para descansar: tira um quadro da aba ativa, para o painel ter o que mostrar, e minimiza a janela do Edge (`Browser.setWindowBounds` com `windowState: "minimized"`; a janela já não aparecia, mas para o Edge ela existe). As páginas passam a valer como as de uma janela minimizada: `visibilityState` vira `hidden`, os timers caem para um por segundo ou menos, animações e quadros param. Medido com o Edge 154, uma página que gastava 90% de um núcleo fica abaixo de 1% (19). Nada se perde: abas, formulários e sessões continuam como estavam. Não descansa no meio de uma tool nem nas mãos do dono (21.10).
- **Acordar:** na hora, com a janela de volta a `normal`, na próxima tool `browser_*` do bot, quando o dono assume o controle e quando ele recarrega a página. Depois de algo que o dono fez com o bot fora de um turno, o navegador volta a descansar sozinho: 20 s sem uso, conferido a cada 30 s.
- **Fechar:** depois de `[browser] idle_minutes` sem uso, que é uma tool do bot ou algo que o dono fez com o navegador nas mãos; quando o bot ou a crew pausa; com `browser_close`. Assistir não é uso: o painel abre sozinho (21.8) e seguraria o navegador aberto para sempre. Nas mãos do dono ele não fecha. Se já há `[browser] max_open` navegadores abertos, abrir outro fecha o que está parado há mais tempo, mas nunca um que esteja no meio de uma tool.
- Se o processo cai ou o WebSocket fecha, o estado vira `closed` e a próxima tool sobe de novo.

### 21.3 Página

- **Abas:** o daemon se prende a toda página nova do navegador (`Target.setAutoAttach` no alvo do navegador, com `flatten` e `waitForDebuggerOnStart`) e a configura antes de ela rodar. Os comandos de preparo saem juntos com `Runtime.runIfWaitingForDebugger`, sem esperar resposta um a um: numa aba aberta pela página, `Page.enable` só responde depois que ela roda (visto com o Edge 154). O navegador só conta como aberto, e uma aba nova do dono só recebe um endereço, depois desse preparo: uma página aberta antes carregaria sem o daemon ver os eventos dela. Uma aba aberta pela página (`target=_blank`, `window.open`) vira a aba ativa, como para quem usa o navegador; quando ela fecha, a que era ativa antes dela volta (`Target.activateTarget`, para o navegador mostrar a mesma que o daemon). No máximo 6 abas: acima disso, fecha a que foi ativa há mais tempo. As tools agem sempre na ativa, e o bot não tem tool para trocar de aba; o dono, no controle, troca de aba, abre uma nova e vai a um endereço (21.10). O título de cada aba, para a lista que o dono vê (21.7), é o `document.title`, lido no mundo isolado (21.6) a cada carregamento e a cada mudança de endereço dentro da página: o Edge sem janela avisa quando a aba ganha um endereço, mas nunca o título dela (19). Até a página carregar, a aba fica sem título; um título que a página troca depois disso só aparece no carregamento seguinte.
- **Tela:** 1280 px de largura, escala 1 (`Emulation.setDeviceMetricsOverride`), e 800 px de altura enquanto ninguém assiste. Com o painel aberto, a altura acompanha o espaço que o painel tem para a página (`browser.resize`, 21.7): a mesma proporção desse espaço, entre 600 e 2000 px, para a tela ao vivo ocupar o painel em vez de deixar um vazio embaixo. A largura não muda, para os sites mostrarem sempre a versão de computador: um painel estreito encolhe a imagem, não a página. O tamanho vale para todas as abas, e volta a 1280 × 800 quando o último que assistia para. O user agent é o do Edge sem a palavra `Headless`, para os sites mostrarem a página comum; nada mais é disfarçado (`navigator.webdriver` continua `true`).
- **Diálogos** (`alert`, `confirm`, `prompt`, "sair da página?"): aceitos na hora; a resposta seguinte da tool diz ao bot o texto que apareceu.
- **Escolher arquivo:** o pedido da página é interceptado e cancelado, e o bot lê que enviar arquivos pelo navegador ainda não é possível.
- **Downloads:** vão para `<pasta do bot>\downloads\` (`Browser.setDownloadBehavior`) e aparecem no painel de arquivos (8.4); a tool diz quando um download termina.
- **Esperar a página:** depois de cada ação, o daemon espera a navegação que ela começou terminar (`Page.loadEventFired`), até 15 s, e a rede ficar quieta (nenhum request pendente por 500 ms), até mais 3 s. Uma página que nunca sossega não prende o bot: ele recebe o que já carregou.

### 21.4 Tools

Ficam no servidor `botloft` (11), então `--allowedTools mcp__botloft` as libera no Claude Code; quem decide o que pede o dono é o daemon (21.5). Todas agem no navegador do próprio bot.

| Tool | Entrada | O que faz |
|---|---|---|
| `browser_open` | `url` | abre um endereço `http` ou `https`, ou um arquivo das pastas do bot (21.5): caminho do Windows, relativo à pasta do bot, ou `file:///` |
| `browser_look` | `from?` | lê a página de novo, sem agir; `from` continua o texto de onde a leitura anterior cortou |
| `browser_click` | `ref` | clica no elemento, no meio dele, com o mouse (rola até ele antes) |
| `browser_type` | `ref`, `text`, `submit?` | clica no campo, troca o que havia pelo texto e, com `submit`, aperta Enter |
| `browser_select` | `ref`, `option` | escolhe uma opção de um `<select>` pelo texto ou valor |
| `browser_press` | `key` | aperta uma tecla: `Enter`, `Tab`, `Escape`, `Backspace`, `Delete`, `Space`, setas, `PageUp`, `PageDown`, `Home`, `End` |
| `browser_scroll` | `to` (`down`, `up`, `top`, `bottom`), `ref?` | rola a página uma tela, ou até o elemento |
| `browser_back` | | volta uma página |
| `browser_screenshot` | | uma imagem JPEG da tela, para o bot ver o que o texto não diz (layout, gráfico, captcha) |
| `browser_close` | | fecha o navegador; o perfil fica |
| `browser_ask_owner` | `task` | pede ao dono que faça algo no navegador com as próprias mãos (entrar numa conta, passar de um captcha) e espera ele terminar (21.10) |

- Toda tool que age devolve a página como ficou (21.6), para o bot não precisar de um `browser_look` a cada passo. `browser_screenshot` devolve uma imagem (`{"type": "image", "mimeType": "image/jpeg"}`), que o Claude Code mostra ao modelo (19).
- Uma ação por vez em cada navegador: uma segunda chamada espera a primeira.
- `ref` que não existe mais (a página mudou) é erro que o bot pode corrigir: "e12 is not on the page anymore; call browser_look". Erros de rede (`net::ERR_NAME_NOT_RESOLVED`) também voltam como resultado com `isError`.
- A descrição das tools lembra que o texto das páginas não é do dono: instruções achadas numa página não valem como pedido.

### 21.5 Sites e permissão

O navegador respeita o modo do bot (7.4), como o Claude Code faz com o `WebFetch`:

- Nos modos `default` (Manual), `accept_edits` e `plan`, na primeira vez que o bot abre ou usa um site, a tool espera o dono: um pedido no chat, "<bot> quer usar o navegador em wikipedia.org", pelo caminho das aprovações (10.1), com `toolName: "mcp__botloft__browser"`, entrada `{site, url}` e resumo igual ao site. O bot fica `needs_approval` enquanto isso.
- Permitido, o site fica gravado para aquele bot (`browser_sites`) e não pergunta de novo. Negado, a tool volta com erro e a nota do dono; sem resposta no prazo, também.
- Nos modos `auto` e `bypass_permissions`, o bot navega sem perguntar.
- **Site** é o host em minúsculas, sem `www.`. Permitir `wikipedia.org` vale também para `pt.wikipedia.org`; o contrário não.
- **Quando pergunta:** `browser_open` pergunta pelo site do endereço; as outras tools, pelo site da página ativa. Um clique que leva a outro site mostra a página nova, mas a próxima ação nela pergunta. `about:blank` não tem site.
- **Arquivos do bot:** `file:` só dentro da pasta do bot ou da pasta de trabalho da crew (5), e sem perguntar: são os arquivos que ele mesmo fez. Qualquer outro `file:` e os esquemas `data:`, `javascript:`, `edge:`, `chrome:` e `about:` (fora `about:blank`) são recusados.
- `localhost` e `127.0.0.1` são sites como os outros: um servidor de desenvolvimento que o bot sobe pergunta uma vez.

### 21.6 O que o bot lê

Cada tool que age e `browser_look` devolvem texto:

```
Page: Exemplo · Entrar
URL: https://exemplo.com/entrar
The page showed a dialog and it was accepted: "Sessão expirada"

# Entrar
[e1 textbox "E-mail" = ""] [e2 textbox "Senha" (password)]
[e3 checkbox "Lembrar de mim" (checked)] [e4 button "Entrar"]
Esqueceu a senha? [e5 link "Recuperar acesso"]
```

- Um script injetado (`Runtime.evaluate`) percorre o documento em ordem de leitura: texto visível, títulos com `#`, itens de lista com `-`, quebras de bloco, e cada controle entre colchetes com uma `ref` (`e<n>`), o papel (`link`, `button`, `textbox`, `checkbox`, `radio`, `select`, `clickable`...), o nome acessível (`aria-label`, `<label>`, texto, `placeholder`, `title`, `alt`) e o estado (valor, marcado, desabilitado). Senhas aparecem só como `(password)`.
- Um nó de texto só com espaços vira um espaço, e espaços seguidos viram um. Assim uma página que põe cada letra num elemento próprio, espaços incluídos (o script do example.com faz isso com o texto em inglês), não chega ao bot com as palavras grudadas.
- Entra o que está visível na página inteira, não só na tela: nada com `display: none`, `visibility: hidden`, `aria-hidden` ou tamanho zero. `iframe` do mesmo site e shadow DOM aberto entram; de outro site, só `[frame]`.
- Elementos clicáveis sem papel (um `div` com `cursor: pointer` que não herda o cursor do pai) viram `clickable`.
- A mesma ref aponta sempre para o mesmo elemento enquanto ele existir, também entre leituras.
- No máximo 20 000 caracteres por resposta; o resto fica para `browser_look {from}`, e a resposta diz quanto falta.

### 21.7 Protocolo

| Método | Params | Result |
|---|---|---|
| `browser.list` | | `BrowserState[]`: os navegadores que não estão fechados |
| `browser.watch` | `botId` | `BrowserView {state, frame}`: o estado e o último quadro. A conexão passa a receber os quadros desse bot |
| `browser.unwatch` | | `null` |
| `browser.resize` | `botId`, `width`, `height` | `null`: o espaço que o painel tem para a página, em pixels do app (de 1 a 10 000 cada). O daemon ajusta a altura da página à proporção desse espaço (21.3). Só da conexão que assiste esse bot; vale também com o navegador fechado, para quando ele abrir. Cada navegador tem um tamanho só: o último pedido vale |
| `browser.take` | `botId` | `BrowserState`: o dono assume o navegador (21.10). A conexão precisa estar assistindo esse bot, e o navegador aberto |
| `browser.release` | `botId` | `BrowserState`: devolve o navegador ao bot; um pedido de ajuda aberto recebe **Pronto** |
| `browser.input` | `botId`, `input` | `null`: um evento do dono na página. Só da conexão que controla |
| `browser.reload` | `botId` | `null`: recarrega a aba ativa. Da conexão que assiste esse bot, com o navegador aberto; não precisa do controle (21.10) |
| `browser.newTab` | `botId` | `null`: abre uma aba em branco, que vira a ativa. Só da conexão que controla |
| `browser.switchTab` | `botId`, `tabId` | `null`: essa aba vira a ativa. Só da conexão que controla; aba que não existe mais é `not_found` |
| `browser.open` | `botId`, `url` | `null`: leva a aba ativa a um endereço `http` ou `https`; sem esquema, vale `https://`. Só da conexão que controla; o que não é endereço da web é `validation` |

- O que muda o navegador responde assim que o pedido é aceito; o resultado chega em `browser.changed` e nos quadros.
- `BrowserState`: `botId`, `status` (`closed`, `starting`, `open`, `failed`), `url`, `title` e `loading` (da aba ativa), `tabs` (as abas abertas, na ordem em que abriram: `BrowserTab {id, title, url, active}`, com `id` opaco), `error` (por que não abriu, em `failed`), `control` (`bot` ou `owner`, 21.10), `resting` (o navegador descansa, 21.2), `ask` (o que o bot pediu ao dono com `browser_ask_owner`, enquanto o pedido está aberto) e `updatedAt`.
- Notificações para todos: `browser.changed` (`BrowserState`) e `browser.action {botId, kind, x, y, label, at}`, com `kind` `open`, `click`, `type`, `select`, `press`, `scroll` ou `back`; `x` e `y` em pixels da página (o tamanho que vem em `browser.frame`) quando a ação tem um ponto; `label` é o nome do elemento, a tecla ou o site, nunca o texto digitado. Clicar, digitar e escolher avisam antes de agir: o daemon acha o elemento e o traz para a tela, manda `browser.action` com o ponto e, se alguém assiste, espera o cursor deslizar até lá no painel (560 ms, o tempo da animação mais uma folga) antes do clique de verdade; sem ninguém assistindo, não espera. As outras ações avisam depois de feitas. Assim a página muda depois que o cursor chega, e não antes.
- Só para a conexão que assiste: `browser.frame {botId, data, width, height}`, um JPEG em base64 e o tamanho da página que ele mostra. Cada conexão assiste um bot por vez; `browser.watch` de outro troca, e fechar a conexão para.
- Os quadros vêm do `Page.startScreencast` (JPEG, qualidade 60, do tamanho da página, 21.3), que roda só enquanto alguém assiste, e só quando a tela muda. O daemon confirma cada quadro no máximo ~15 vezes por segundo. Um app lento não acumula quadros: a conexão manda sempre o mais novo quando consegue, e quadros não contam no limite de 1024 notificações (11.1).

### 21.8 App (N2)

- No cabeçalho do bot, o botão **Navegador** (globo), ao lado de Arquivos. Com quadros chegando, o título do painel ganha a etiqueta "Ao vivo"; com o navegador descansando (21.2), "Em descanso", e a dica diz que ele não gasta o computador e volta na hora em que o bot ou o dono precisar. A tela mostra o quadro tirado ao descansar. Com o navegador aberto e em uso e o painel fechado, o botão ganha um ponto que pulsa; descansando, não. O painel abre sozinho quando o navegador do bot sobe, quando acorda do descanso para o bot e quando ele pede uma mão (21.10); um navegador que já estava aberto quando o dono abriu o bot fica atrás do botão (15.1). Quando o navegador entra em descanso, o painel fecha na hora, deslizando para a direita, e o chat volta a ocupar o espaço; aberto de novo pelo dono, mostra "Em descanso". Nas mãos do dono, ou com um pedido de mão aberto, ele fica. As duas coisas seguem a escolha "Abrir o navegador e as telas quando um bot começar a usar" (15.1): desligada, o painel só abre e fecha pelo botão.
- O painel divide o lugar com detalhes e arquivos (15.1) e é redimensionável como eles (`botloft.panel.browser`, 560 px de início), com um botão para ocupar todo o espaço que o chat pode ceder.
- Em cima, as abas: uma por página aberta, na ordem em que abriram, com o título e, embaixo dele, o site (ou o nome do arquivo); a ativa em destaque, "Nova aba" numa aba em branco, e **+** no fim. Não cabendo, a fila rola para o lado. Sem o controle elas só mostram o que o bot abriu, e a dica diz para assumir o controle; no controle, um clique troca de aba e **+** abre uma nova, já com o cursor na barra do endereço (21.10).
- Depois, a barra do endereço: **Recarregar** (a seta circular), que vale a qualquer momento (21.10); o endereço, com o cadeado ou a roda de carregando; e "Abrir no meu navegador" (`open_url`, 15.2). O endereço é só leitura e selecionável; no controle vira um campo: Enter leva a aba até o que o dono digitou (`browser.open`), Esc desfaz. Embaixo, a tela ao vivo ocupa o espaço que sobra no painel: o app mede o painel abaixo da barra do endereço, tira o lugar da legenda e do botão de assumir (80 px) e manda o resto ao daemon (`browser.resize`, 300 ms depois da última mudança, e de novo ao voltar a assistir); cada quadro aparece no maior tamanho que cabe, na proporção da página. Arrastar a borda do painel, alargá-lo ou mudar o tamanho da janela muda a página junto. O que entra e sai do painel (a legenda, o pedido de ajuda, a faixa do controle, 21.10) não entra na conta: a página nunca muda de tamanho por causa disso, só a imagem encolhe um pouco para caber. Sobre a tela, o cursor do bot (a seta, na cor dele) indo até cada ponto de `browser.action`, um anel no clique e uma legenda curta do que ele fez ("Clicou em Entrar", "Escreveu em E-mail", "Abriu exemplo.com").
- Sem navegador aberto: o mascote e "<bot> ainda não abriu o navegador", com a explicação de que tudo que ele fizer num site aparece ali ao vivo. Depois de fechado, o último quadro fica apagado com "Navegador fechado". Em `failed`, o motivo em "Details".
- No chat, a linha de uma tool `browser_*` ganha o ícone do globo e o botão "Ver no navegador", que abre o painel.
- O pedido de site (21.5) é um cartão próprio: "<bot> quer usar o navegador em **wikipedia.org**", o endereço embaixo, Permitir e Negar. Respondido: "Você permitiu wikipedia.org".
- O dono no controle e o pedido de ajuda têm a própria parte da tela (21.10).

### 21.9 Dados e config

- Migration: `browser_sites (bot_id, host, allowed_at)`, chave `(bot_id, host)`.
- `config.toml`:

  ```toml
  [browser]
  path = ""          # vazio = Microsoft Edge do Windows
  idle_minutes = 10  # sem uso, fecha (21.2)
  max_open = 4       # navegadores abertos ao mesmo tempo
  ```

- **Privacidade:** endereço, título, texto de página, o que o bot digita e os quadros são dados pessoais como os itens do chat (8.5): nunca vão para o log em `info` ou acima. O `debug` registra métodos do CDP e ids.

### 21.10 Dono no controle (N3)

O dono pode usar o navegador do bot com as próprias mãos: clicar, arrastar, rolar e digitar na tela ao vivo. Serve para o que o bot não deve ou não consegue fazer sozinho: entrar numa conta com a senha do dono, passar de um captcha, confirmar um código que chegou no celular dele.

- **Assumir:** "Assumir o controle", no painel (21.8), com o navegador aberto. Quem controla é a conexão que assiste o bot (21.7); parar de assistir (fechar o painel, trocar de bot, fechar o app, cair a conexão) devolve o controle na hora, para o bot nunca ficar preso. Cada navegador tem um só controlador: `browser.take` de outra conexão enquanto um controla é `conflict`.
- **O bot espera:** enquanto o dono controla, cada tool `browser_*` do bot espera ele devolver, até 10 minutos. Passou disso, a tool volta com erro: o dono está usando o navegador. `browser_ask_owner` não espera, porque é ela que chama o dono. O bot continua `busy` nessa espera, e o painel diz que ele espera.
- **Devolver:** "Pronto, devolver para <bot>" no painel. Fechar o navegador (bot pausado, `browser_close`, processo que caiu) também devolve.
- **O que passa:** cliques, arrasto e roda do mouse (`Input.dispatchMouseEvent`), teclas (`Input.dispatchKeyEvent`) e texto colado ou composto (acento, IME: `Input.insertText`), sempre na aba ativa e na ordem em que o dono os fez. A conexão entrega cada evento a uma fila do navegador, que os manda um a um. Uma tecla com texto desce com o texto; Ctrl ou Meta com uma letra desce sem texto e com a tecla virtual do Windows, e o próprio Chromium faz os atalhos de edição (Ctrl+A, Ctrl+Z, Ctrl+Backspace). No macOS ele não faz os de Cmd a partir da tecla (visto no runner macOS do CI): Cmd+A, C, X, V, Z e Shift+Cmd+Z descem com o comando de edição em `commands` (`selectAll`, `copy`, `cut`, `paste`, `undo`, `redo`). AltGr (Ctrl+Alt) conta como texto. Colar usa a área de transferência do dono, lida pelo app, nunca a do navegador do bot.
- **Pontos** em pixels da página, como `browser.action`; fora do tamanho que ela tem agora (21.3) são recusados.
- **Privacidade:** o que o dono digita e cola não vai para o log, nem para `browser.action`, nem para o bot. O log registra só que um evento chegou, em `debug`. Depois, o bot lê a página como sempre (21.6), com a senha só como `(password)`.
- **Sites:** o que o dono abre com as próprias mãos não passa pela permissão de sites, e também não libera o site para o bot: a próxima tool do bot numa página de outro site pergunta como sempre (21.5).
- **Abas e endereço:** trocar de aba, abrir uma aba nova e ir a um endereço mudam onde o bot trabalha, então só valem no controle (`browser.switchTab`, `browser.newTab`, `browser.open`, 21.7). Entram na mesma fila dos eventos do dono, na ordem em que ele os fez. Pelo endereço o dono só abre `http` e `https`. O bot não troca de aba sozinho (21.3): a aba em que o dono devolve o navegador é onde o bot continua, e a faixa do controle diz isso quando há mais de uma.
- **Recarregar** não precisa do controle (`browser.reload`): não muda onde o bot está, e uma página que travou não deveria custar ao dono parar o bot. O bot lê no próximo resultado "The owner reloaded the page."; uma `ref` de antes dá o erro de sempre, e ele lê a página de novo (21.4).
- **O que o bot vê quando a aba ativa mudou nas mãos do dono** (ele trocou de aba, abriu uma nova, ou um clique dele abriu ou fechou uma): o bot nunca age numa aba que não leu. A próxima tool que age na página (`browser_click`, `browser_type`, `browser_select`, `browser_press`, `browser_scroll`, `browser_back`) não faz nada e volta com erro: o dono trocou de aba, leia com `browser_look` a que está aberta. `browser_look`, `browser_screenshot`, `browser_open` e a volta de `browser_ask_owner` funcionam, e dizem "The owner switched tabs while they had your browser. This is the tab that is open now." O aviso vale uma vez. Se o dono devolve na mesma aba em que pegou, nada disso acontece.

`BrowserInput`, com a etiqueta `kind`:

| `kind` | Campos | Vira |
|---|---|---|
| `mouse` | `action` (`move`, `down`, `up`), `x`, `y`, `button` (`left`, `middle`, `right`, `none`), `buttons` (os apertados: 1 esquerdo, 2 direito, 4 do meio), `clicks`, `modifiers` | `mouseMoved`, `mousePressed`, `mouseReleased` |
| `wheel` | `x`, `y`, `dx`, `dy` (pixels), `modifiers` | `mouseWheel` |
| `key` | `key` e `code` como no DOM, `modifiers` | a tecla descendo e subindo |
| `text` | `text`, até 10 000 caracteres | `Input.insertText` |

`modifiers` como no CDP: Alt 1, Ctrl 2, Meta 4, Shift 8.

**Pedir ajuda.** `browser_ask_owner {task}` é o bot pedindo ao dono que faça algo no navegador e esperando. `task` é uma frase para o dono, na língua dele ("Entre na sua conta do GitHub").

- Precisa do navegador aberto. Abre um pedido no chat pelo caminho das aprovações (10.1), com `toolName: "mcp__botloft__browser_help"`, entrada `{task, url, site}` e resumo igual a `task`; o bot fica `needs_approval`, com o prazo das aprovações. `BrowserState.ask` guarda `task` até o pedido fechar.
- O cartão no chat: "<bot> precisa de você no navegador", a tarefa, o site, e **Assumir o navegador** (abre o painel e assume o controle), **Pronto** e **Não vou fazer**, com nota.
- Devolver o navegador pelo painel com um pedido aberto responde **Pronto**. Qualquer resposta devolve o controle ao bot.
- **Pronto:** a tool volta dizendo que o dono terminou, com a página como ficou (21.6). **Não vou fazer:** erro com a nota. Sem resposta no prazo: erro.
- As regras do bot (5.1) dizem: nunca peça senha, código ou dado de cartão no chat. Peça com `browser_ask_owner`, e o dono digita ele mesmo.

**No app** (21.8):

- Embaixo da tela ao vivo, com o navegador aberto: "Assumir o controle" e a explicação ("Para entrar numa conta ou passar de um captcha. <bot> espera enquanto isso.").
- Com um pedido aberto, em cima da tela: "<bot> precisa de você" com a tarefa e "Assumir o controle". O botão do navegador no cabeçalho ganha o ponto de atenção.
- No controle: a tela ganha a borda de destaque, e o cursor do bot some. Uma faixa diz "Você está no controle" e, se o bot espera, que ele espera, com "Pronto, devolver para <bot>". Com mais de uma aba, diz também que o bot continua na aba que ficar aberta. As abas e o endereço passam a responder (21.8). O teclado vai para a página enquanto a tela tem o foco; sem foco, a faixa diz "Clique na tela para digitar". Esc vai para a página, não devolve.

### 21.11 Marcos

| Marco | Entrega | Pronto quando |
|---|---|---|
| **N1** Navegador no daemon | `browser/` (processo, CDP, abas, leitura da página, ações), tools `browser_*`, sites e aprovação, RPC e notificações, migration, testes com o Edge real e páginas locais | um bot abre uma página local, preenche um formulário e lê o resultado; no modo Manual, o primeiro acesso a um site espera o dono |
| **N2** Painel ao vivo | painel do navegador com quadros, cursor e legenda, botão no cabeçalho, "Ver no navegador" no chat, cartão de site, textos nos três idiomas | ver pelo app, ao vivo, um bot real pesquisar e clicar |
| **N3** Dono no controle | `browser.take`, `browser.release`, `browser.input`, `browser_ask_owner` e o pedido de ajuda, a espera das tools, o controle no painel e o cartão no chat, textos nos três idiomas, testes com o Edge real | com o Claude Code real, um bot numa página de login pede ajuda, o dono assume pelo cartão, digita, devolve, e o bot continua na página já dentro da conta |
| **N3.1** Tela do tamanho do painel | `browser.resize`, a página com a altura que o painel pede, a tela ao vivo ocupando o espaço do painel, testes com o Edge real | num painel alto, a página do bot ocupa a altura toda em vez de deixar um vazio; arrastar a borda do painel muda a página junto |
| **N3.2** Abas e recarregar | a lista de abas em `BrowserState`, `browser.reload`, `browser.newTab`, `browser.switchTab`, `browser.open`, o aviso ao bot quando o dono troca de aba, as abas e o botão de recarregar no painel, o endereço que o dono digita, textos nos três idiomas, testes com o Edge real | pelo app, o dono vê as abas que o bot abriu, recarrega a página, assume, abre uma aba nova num endereço, troca de aba e devolve; o bot lê que a aba mudou antes de agir |
| **N3.3** Descanso | o navegador descansa quando o turno do bot termina e acorda quando é usado, `resting` em `BrowserState`, fechar por falta de uso mesmo com o painel aberto, a etiqueta "Em descanso", textos nos três idiomas, testes com o Edge real | com um bot real parado numa página que se mexe, o Edge dele não gasta processador; o bot volta a usar a página sem perder nada |
| **N4** Telas | o bot desenhando telas (HTML) que aparecem lado a lado numa área de design, atualizadas enquanto ele escreve | seção 22 |

## 22. Telas

Status: **T1, T2 e T3 implementados** (22.8). É o N4 da seção 21.

### 22.1 O que é

Uma área de design ao lado do chat: cada página HTML que o bot faz aparece como uma prancheta, renderizada de verdade, e se monta na frente do dono enquanto o bot escreve o arquivo. O dono conversa com o bot e vê o resultado ao mesmo tempo.

- **Tela** é um arquivo `.html` ou `.htm` que o bot fez: a lista de arquivos (8.4) filtrada, dentro da pasta de trabalho da crew ou da pasta do bot. Não há ferramenta nova: basta o bot escrever HTML. As regras do bot (5.1) contam isso a ele e pedem uma tela por arquivo.
- **Ao vivo:** enquanto o modelo escreve um `Write` de um `.html`, o Claude Code manda a entrada da ferramenta em pedaços (`stream_event` com `content_block_delta` e `input_json_delta`; visto com 2.1.284: 379 pedaços para um arquivo de 3 KB). O daemon junta os pedaços num rascunho da tela, e o app mostra cada versão (22.3).
- Depois de um `Write` ou `Edit`, a tela recarrega do disco.
- **Aparelho:** cada tela tem o tamanho de um computador (1280 × 800), tablet (834 × 1112) ou celular (390 × 844). Vale o que o bot pede com `<meta name="botloft-device" content="mobile">` (ou `tablet`, `desktop`) nos primeiros 8 KB do arquivo, ou computador; o dono troca por tela no app.

### 22.2 Servidor das telas

- `GET /view/<chave>/<raiz>/<caminho>`, no mesmo servidor do daemon (só 127.0.0.1). A `<chave>` tem 128 bits aleatórios, é uma por bot, fica só na memória e muda a cada início do daemon; só o app a recebe, em `screens.list`. `<raiz>` é `w` (pasta de trabalho da crew) ou `b` (pasta do bot). O caminho é resolvido de verdade e precisa ficar dentro da raiz: `..`, links para fora e entradas ocultas dão 404, sem dizer por quê. CSS, imagens, fontes e scripts ao lado do HTML carregam pelo mesmo caminho, como num site.
- Toda resposta leva `Content-Security-Policy: sandbox allow-scripts allow-forms allow-popups allow-modals`: a página roda numa origem opaca, sem acesso ao daemon nem aos cookies e ao armazenamento de 127.0.0.1, também se aberta direto. E ainda `Cache-Control: no-store`, `X-Content-Type-Options: nosniff`, `Referrer-Policy: no-referrer` e, para um pedido com `Origin: null` (a própria tela buscando um arquivo dela), `Access-Control-Allow-Origin: null`. O tipo sai da extensão (8.4). Acima de 20 MiB, 404.
- Enquanto um arquivo tem rascunho, o endereço dele serve o rascunho, com o script do cursor do bot (22.3). O arquivo do disco é servido como está.
- A CSP do app ganha `frame-src http://127.0.0.1:*`: as telas abrem num `iframe` do endereço delas, com os próprios scripts e as bibliotecas que carregam da internet, o que não funcionaria num `srcdoc`, que herda a CSP do app.
- O log registra só o status, em `debug`.

### 22.3 Rascunho ao vivo

- Na leitura do stdout (8.1), o daemon acompanha, por bot, o bloco `tool_use` que o modelo está escrevendo: `content_block_start` com `name: "Write"` começa um rascunho com o `id` do bloco, cada `input_json_delta` soma ao JSON parcial, `content_block_stop` fecha a escrita. Eventos de subagentes ficam de fora, como no chat.
- Do JSON parcial, o daemon tira `file_path` (quando a string dele fecha) e o começo de `content` que já chegou, com os escapes completos desfeitos. Só vira rascunho um `.html` ou `.htm` numa das duas raízes.
- Notificação `screen.draft {botId, path, url, rev, done}`: `url` já aponta para aquela versão (`?rev=<n>`), no máximo uma a cada 250 ms e uma quando a escrita fecha. O rascunho fica até o resultado daquele `tool_use` chegar (gravado, negado ou com erro), e então sai uma última com `done: true`, e o endereço volta a servir o disco. Um arquivo que o dono negou some da área de design. O fim do turno (`result`) também fecha os rascunhos que sobraram.
- Rascunho não vai para o banco, para o disco nem para o log.
- **Cursor do bot:** cada rascunho servido leva primeiro um script pequeno do daemon (`<script data-botloft-cursor>`), logo depois do `<!doctype>`, que precisa continuar primeiro para a página manter o modo de layout; sem doctype, no começo. Um doctype ainda sendo escrito fica sem script naquela versão. Quando a página termina de ler (`DOMContentLoaded`) e de novo no `load`, o script acha onde o conteúdo acaba agora: o fim do texto mais novo, ou o centro do elemento mais novo com tamanho, fora `script`, `style`, `template` e `noscript`. Se esse ponto está abaixo de 85% da altura da tela, rola a página para ele ficar a 60%. Um `iframe` de outra origem roda em outro processo e pode ler a página antes de saber o próprio tamanho (`innerHeight` 0, visto no Edge 154); então o script espera o `resize` antes de medir. Por fim manda ao app, por `postMessage`, `{botloft: "cursor", view, point, box}`: `view` é o tamanho da tela em que mediu, e `point` e `box` vêm nos pixels dela: o ponto e a caixa da parte em volta (o elemento mais próximo que não é texto em linha, menos a página inteira). Sem nada visível ainda (o bot escrevendo o CSS), `point` e `box` vão `null`. O script não muda mais nada na página.

### 22.4 Protocolo

| Método | Params | Result |
|---|---|---|
| `screens.list` | `botId` | `Screen[]`, da mais nova para a mais antiga |

- `Screen`: `path`, `name`, `folder` (como em `BotFile`), `modifiedAt`, `url` (com `?v=<modifiedAt>`, para o app recarregar quando muda), `device` (a dica do arquivo, ou `null`) e `writing` (tem rascunho agora). Um arquivo que ainda não existe mas tem rascunho entra também. Arquivos escritos pelo bot fora das duas raízes não entram: não há como servi-los.
- Notificação `screen.draft` (22.3), para todos.

### 22.5 App

- No cabeçalho do bot, o botão **Telas**, ao lado do Navegador. Com uma tela sendo escrita e o painel fechado, o botão ganha um ponto que pulsa; o painel de telas abre sozinho quando começa uma escrita, no lugar de outro painel aberto, menos o navegador nas mãos do dono, uma vez por tela (15.1).
- O painel divide o lugar com detalhes, arquivos e navegador (15.1), redimensionável (`botloft.panel.screens`, 640 px de início) e com o botão de alargar.
- **Visão geral:** um quadro com fundo pontilhado, como nos apps de design, com as telas lado a lado, cada uma com o nome em cima e o tamanho do aparelho, reduzidas pelo zoom do quadro (menos, mais e ajustar; `botloft.screens.zoom`). A que está sendo escrita ganha a borda na cor do bot, o mascote trabalhando e "Escrevendo…", e o quadro rola até ela. Cada versão nova carrega por trás e troca quando pronta, sem piscar. Nessa visão as telas não recebem clique: um clique abre a tela.
- **Uma tela:** na largura do painel e de verdade (rolar, clicar, preencher), com Computador, Tablet e Celular (`botloft.screens.devices`, por arquivo), "Abrir" (`open_file`, 15.2, no navegador do Windows), "Mostrar na pasta" e a volta para todas.
- Até 12 telas no quadro; "Mostrar mais" traz as outras.
- Sem telas: o mascote e "<bot> ainda não fez nenhuma tela", com a explicação de que cada página HTML que ele escrever aparece ali e se monta enquanto ele escreve.
- No chat, a linha de um `Write` ou `Edit` de `.html` que não falhou ganha "Ver em telas", que abre o painel nessa tela.
- **Cursor:** na prancheta e na tela em foco, enquanto o bot escreve, o cursor dele (a seta na cor do bot, com o nome e os três pontos de quem escreve, o mesmo do navegador) desliza até o ponto que o rascunho diz (22.3), e uma moldura na cor do bot marca a parte que está nascendo. Sem ponto ainda, o cursor espera no canto de cima. O app converte as medidas pela largura de `view` (numa prévia com zoom de CSS, o `iframe` vê mais pixels que o aparelho). Cada versão nova carrega por trás e fala antes de ir para a frente: vale a marca do `iframe` que está à mostra, para o cursor e a moldura baterem com o que se vê. Perto da borda direita, o nome passa para a esquerda da seta. O cursor e a moldura ficam em cima da tela, no tamanho do app, não da prancheta, e somem quando a escrita termina.
- Textos nos três idiomas (15.6).

### 22.6 Segurança

- A tela roda HTML e JavaScript escritos pelo bot. A CSP de sandbox (22.2) a isola do app e do daemon; o `iframe` no app também usa `sandbox="allow-scripts allow-forms allow-popups allow-modals"`, sem `allow-same-origin`. Um script da tela pode acessar a internet, como a própria página abriria num navegador.
- A chave só dá leitura, só dos arquivos das duas raízes, e some quando o daemon reinicia.
- O script do cursor (22.3) roda dentro da mesma sandbox, com a página do bot, e só manda números. O app aceita a mensagem só de um dos seus próprios `iframe`s, lê só números finitos e os prende ao tamanho da página; a página não tem como mover o cursor para fora da prancheta nem mandar outra coisa ao app.

### 22.7 Fora desta etapa

- Apontar uma parte da tela e pedir a mudança ao bot.
- As telas de todos os bots da crew num quadro só.
- Exportar telas como imagem ou PDF.
- Rascunho ao vivo de um `Edit` (hoje a tela recarrega quando ele termina).

### 22.8 Marcos

| Marco | Entrega | Pronto quando |
|---|---|---|
| **T1** Telas no daemon | rascunho a partir do stream, JSON parcial, `/view` com sandbox, `screens.list`, `screen.draft`, regras do bot, testes com `FakeRuntime` | um `Write` de `.html` transmitido pelo `FakeRuntime` aparece como rascunho em `/view` e vira o arquivo depois do resultado |
| **T2** Área de design | painel de telas, quadro com zoom, tela em foco, aparelhos, escrita ao vivo, "Ver em telas" no chat, textos nos três idiomas | ver pelo app um bot real escrever uma tela e ela se montar enquanto ele escreve |
| **T3** Cursor do bot | script do cursor nos rascunhos, cursor e moldura na prancheta e na tela em foco, a prévia com o mesmo script | ver pelo app um bot real escrever uma tela com o cursor dele seguindo cada parte que nasce |

## 23. Perguntas ao dono

Status: **P1 e P2 implementados** (23.9). É o item 2 da seção 18.

### 23.1 O que é

Um bot que precisa de uma decisão ou de uma informação do dono pergunta com a tool `ask_owner` e **não espera**: a pergunta vai para a caixa de perguntas do app e para o chat do bot, e o bot segue com o que não depende dela ou termina o turno. Quando o dono responde, a resposta chega ao bot como uma mensagem do dono, num turno novo.

É diferente de uma aprovação (10.1), que segura a ferramenta até o dono decidir e expira no prazo. Uma pergunta não tem prazo: um bot de rotina (20) pergunta às 3 da manhã, e o dono responde quando acordar. É por isso que ela não usa o caminho das aprovações nem deixa o bot em `needs_approval`.

### 23.2 A tool

`ask_owner {question, options?}`:

- `question`: o que o bot quer saber, para o dono, na língua dele; até 2 000 caracteres, em markdown.
- `options`: de 2 a 5 respostas prontas, cada uma até 100 caracteres, sem repetir. O dono pode escolher uma ou escrever outra coisa.
- **Saída:** `question_id` e o lembrete de que a resposta chega depois, como mensagem que começa com `Answer to your question`, e de que o bot não deve esperar nem adivinhar a resposta: segue com o que não depende dela ou termina o turno.
- **Limite:** até 5 perguntas abertas por bot. A sexta volta como erro ao bot, que deve esperar uma resposta antes de perguntar de novo. Campo inválido também volta como erro, sem pergunta.
- Vale em qualquer modo de permissão, também em `bypass_permissions`: perguntar não faz nada no computador.
- A chamada aparece no chat como o item `tool` de sempre; o app não mostra essa linha, porque o cartão da pergunta (23.4) já diz tudo.

**Regras do bot** (5.1): quando precisar de uma decisão ou informação do dono para continuar, e principalmente numa rotina ou numa task de outro bot, em que ninguém lê o chat na hora, pergunte com `ask_owner` em vez de só escrever a pergunta na resposta. Uma pergunta por assunto, curta, com `options` quando as respostas possíveis forem poucas. Não espere: siga com o resto ou termine o turno. Nunca peça senha, código ou dado de cartão: no navegador, isso é `browser_ask_owner` (21.10).

### 23.3 Estados

`Question`: `id` (`qst_`), `crewId`, `botId`, `text`, `options`, `status`, `answer`, `createdAt` e `answeredAt`.

| `status` | Quando |
|---|---|
| `open` | o bot perguntou e o dono ainda não respondeu |
| `answered` | o dono respondeu; `answer` guarda a resposta |
| `dismissed` | o dono descartou sem responder |

- **Responder** (`questions.answer {questionId, answer}`): a resposta é validada como uma mensagem do dono (9.1) e vira, numa transação, a resposta gravada, a pergunta `answered` e uma message do dono para o bot com `question_id`, entregue pelo courier como qualquer outra (bot pausado recebe quando voltar).
- **Descartar** (`questions.dismiss {questionId}`): fecha a pergunta e **não** avisa o bot, para não gastar um turno com isso. Quem quer que o bot saiba responde, mesmo que seja "deixa pra lá".
- Só uma pergunta `open` pode ser respondida ou descartada; outra dá `conflict`. Bot ou crew arquivados também dão `conflict`.
- A conversa normal não fecha a pergunta: o dono pode falar do assunto no chat e depois descartar.

### 23.4 No chat

- **Item `question`** (8.2): `{question}`, a `Question` inteira, criado quando o bot pergunta e regravado quando ela muda. A linha da conversa (`lastActivity`) ganha o `kind` `question`, com o texto da pergunta.
- **A resposta** chega como `inbound` (a message do dono, com `questionId`). O bot a lê assim (9.3), sem o envelope `[botloft]`, porque é o dono falando:

```
Answer to your question qst_01J9Z...: "<as primeiras 300 letras da pergunta>"

<resposta>
```

### 23.5 Caixa de perguntas

`questions.list {status?}` devolve as perguntas de bots e crews ativos, da mais nova à mais velha; sem `status`, só as `open`. As de bots ou crews arquivados ficam de fora e voltam se eles forem restaurados. `question.changed {question}` avisa o app de toda pergunta nova ou que mudou, para a caixa não depender do chat de cada bot estar carregado.

### 23.6 App (P2)

- **Cartão no chat:** "<bot> pergunta", o texto da pergunta (markdown, como uma resposta do bot), as opções como botões e um campo para escrever outra resposta, com **Responder** e **Descartar**. Respondida, o cartão mostra a resposta e quando; descartada, "Descartada". Opção escolhida responde na hora, com o texto da opção.
- **Caixa:** "Perguntas", no topo da barra lateral, abaixo de "Equipes", com o número de perguntas abertas na cor de aviso. Abre no meio da janela a lista das perguntas abertas, cada uma com o rosto e o nome do bot, a crew, a hora e o mesmo cartão para responder ali mesmo, e "Abrir conversa" para ir ao chat do bot. Sem perguntas: o mascote e "Nenhuma pergunta esperando você".
- Uma pergunta aberta conta como algo que espera o dono: marca o ícone na barra de tarefas e o ponto do botão da barra lateral escondida (15.1, 15.2).
- Textos nos três idiomas (15.6).

### 23.7 Dados

Tabela `questions`: `id, crew_id, bot_id, chat_item_id, text, options (JSON), status, answer, created_at, answered_at`, com índice em `(status, created_at)` e em `bot_id`. `messages` ganha `question_id`. Excluir um bot (7.6) solta `messages.question_id` das perguntas dele e apaga as perguntas, antes dos itens do chat.

### 23.8 Fora desta etapa

- Notificação do Windows quando chega uma pergunta.
- Pergunta com prazo, ou que cai sozinha depois de um tempo.
- Responder com anexos.

### 23.9 Marcos

| Marco | Entrega | Pronto quando |
|---|---|---|
| **P1** Perguntas no daemon | migration, tool `ask_owner`, item `question`, `questions.list`, `questions.answer`, `questions.dismiss`, `question.changed`, a resposta como message, regras do bot, exclusão, testes com `FakeRuntime` | com `FakeRuntime`, um bot pergunta, a tool volta na hora, o dono responde e o bot recebe a resposta pelo stdin com a pergunta citada |
| **P2** App | cartão no chat, caixa de perguntas na barra lateral, marca de espera, textos nos três idiomas | com o Claude Code real, um bot pergunta, o dono responde pela caixa e o bot continua o trabalho com a resposta |
