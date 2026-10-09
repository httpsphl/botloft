# ADR 0003: mais de um agente de código por trás de um bot (Codex e Antigravity)

Status: proposto (2026-10-09). Estende o ADR 0001: o bot continua headless e com chat, mas o processo por trás dele deixa de ser sempre o Claude Code.

## Contexto

Todo bot roda `claude -p` com `stream-json` (ADR 0001). O dono quer escolher o agente de cada bot, começando por **Codex** (OpenAI) e **Antigravity** (`agy`, Google), usando as assinaturas e os logins que ele já tem. A meta é paridade total com o bot Claude: chat, mensagens entre bots, MCP do Botloft, aprovações, rotinas, navegador, desktop, perguntas ao dono, contexto e uso.

O mapa do acoplamento atual (código lido em 2026-10-09) mostra que o Claude não está só atrás do `Runtime`, que abstrai apenas processo, bytes e kill:

- **Entrada e saída:** o texto do dono vira uma linha `{"type":"user",...}` (courier); a saída é lida por `chat/events.rs`, `chat/control.rs`, `chat/account.rs` e `chat/session.rs`, que conhecem `system/init`, `stream_event`, `assistant`, `user` com `isReplay`, `result`, `rate_limit_event` e `control_response`. O recibo de leitura e a fila `waiting` dependem do eco `isReplay`.
- **Aprovações:** `--permission-prompt-tool` aponta para a tool MCP `permission_prompt`; `approvals/always.rs`, `core/chat.rs` e o cartão de plano (`ExitPlanMode`) conhecem os nomes e os formatos de entrada das ferramentas do Claude.
- **Isolamento entre crews:** `.claude/settings.json` (regras `deny`, `claudeMdExcludes`), `--setting-sources`, `CLAUDE.md` como memória.
- **Contexto, uso e custo:** `get_context_usage`, `rate_limit_event`, `total_cost_usd`.
- **Dados e UI:** `BotModel`, `BotEffort`, `PermissionMode`, `ClaudeAccount` no protocolo; `bots.model`, `effort`, `session_id` no banco; listas de modelos nos diálogos do app.

## O que foi visto nos dois agentes (2026-10-09)

**Antigravity `agy` 1.3.1** (já instalado aqui):

- Tem `--print`, `--input-format stream-json --output-format stream-json`, `--model`, `--effort`, `--mode` (`accept-edits`, `plan`), `--add-dir`, `--continue`, `--conversation <id>`, `--sandbox`, `agy mcp add` e `agy models`.
- O protocolo é **próprio**, não o do Claude. Saída: uma linha JSON por evento, com `"event"`: `init` (traz `conversation_id`, `model`, `cwd`, a lista de `tools` e `permission_mode`), `step_update` (`step_type` `user_input` ou `agent_response`, `state` `ACTIVE` ou `DONE`, `text_delta`, `usage`) e `result` (`status`, `response`, `usage`). Entrada: `{"event":"user","message":{"role":"user","content":[{"type":"text","text":"..."}]}}`; sem o campo `event` ou sem `content` a CLI responde com erro explícito.
- Um turno de teste (`gemini-3.8-flash-low`, "Say only: pong") fechou com `result.status` `SUCCESS` e `usage` de tokens.
- Não visto ainda: vários turnos no mesmo processo, ferramentas e o evento de pedido de aprovação (`ask_permission` está na lista de ferramentas, `permission_mode` veio `request-review`), formato de uma tool MCP por HTTP, retomada com `--conversation`.

**Codex `codex-cli` 0.148.0-alpha.9** (instalado com o app do Codex):

- `codex exec --json` roda **um turno** e emite JSONL: `thread.started`, `turn.started`, `item.completed` (`agent_message`), `turn.completed` com `usage`. Serve para tarefas isoladas, não para um bot que fica ligado.
- `codex app-server` (experimental) fala JSON-RPC e gera o esquema do protocolo (`generate-json-schema`): pedidos de aprovação tipados (`CommandExecutionRequestApprovalParams`, `FileChangeRequestApprovalParams`, `PermissionsRequestApprovalParams`, `McpServerElicitationRequestParams`) e chamadas de ferramenta dinâmicas (`DynamicToolCallParams`). É o canal que serve ao Botloft: conversa contínua e aprovações que o daemon responde.
- Tem `--add-dir`, `-s read-only|workspace-write|danger-full-access`, `codex mcp`, `codex login`, `-c model=...`, `model_reasoning_effort` no `config.toml`, `AGENTS.md` como arquivo de instruções.

## Decisão

1. **Um bot tem um agente** (`claude`, `codex`, `agy`), escolhido ao criar e gravado em `bots.agent` (padrão `claude`). Trocar de agente num bot que já existe fica fora da primeira versão: a sessão e o histórico de cada agente são dele.
2. **Uma interface `Agent` no daemon** é dona de tudo que é específico de um agente, e o resto do daemon só vê eventos normalizados:
   - **Descoberta e saúde:** onde está o binário, versão mínima, estado do login (`auth_status`) e o comando de entrar.
   - **Lançamento:** argumentos, ambiente e os arquivos que o agente lê na pasta do bot (instruções, regras, configuração de MCP, regras de isolamento).
   - **Entrada:** como um turno do dono, de outro bot ou do daemon entra, com imagens, e como o recibo de leitura é obtido.
   - **Saída:** um decodificador com estado, de bytes para o enum normalizado: turno começou, resposta, ferramenta (início e fim), turno acabou (tokens, custo se houver), contexto, limite de uso, falha de login, sessão iniciada ou perdida, subagentes, modelo em uso.
   - **Aprovações:** converte o pedido do agente para o `{tool_name, input, tool_use_id}` comum e leva a resposta de volta.
   - **Capacidades:** modelos e esforços que oferece, modos de permissão, vocabulário de ferramentas (comando, edição de arquivo, saída do plano, web) e o que ele não faz.
3. **O Claude vira a primeira implementação**, sem mudar de comportamento. O `Runtime` continua sendo a camada de processo, e o `FakeRuntime` passa a ser dirigido por eventos do agente em teste.
4. **Modelo e esforço viram texto validado pelas capacidades do agente** (hoje são enums do Claude no protocolo e no banco). O app monta os seletores a partir do que `agents.list` informa, não de listas fixas.
5. **O que um agente não faz aparece desligado, não escondido.** Contexto exato, custo e uso do plano são opcionais por agente; o app mostra o que houver e diz o que falta.
6. **Isolamento é requisito de lançamento.** Um agente só roda bots quando o Botloft consegue impor o que o `settings.json` impõe no Claude (spec 7.5): sem ler as outras crews nem os dados do Botloft. Cada agente usa o seu mecanismo (sandbox do Codex com `--add-dir`; `--sandbox` e `--add-dir` no `agy`) e a spec registra o que foi testado. Sem isso comprovado, o agente fica marcado como experimental e desligado por padrão.
7. **Protocolo do Codex:** `app-server` por JSON-RPC, não `exec --json` por turno. **Protocolo do AGY:** `stream-json` próprio do `agy`.
8. **Nada copiado de outros projetos.** Fontes: a documentação e o `--help` de cada CLI, o esquema gerado por `codex app-server generate-json-schema` e testes reais, registrados na seção 30 da spec.

## Ordem do trabalho

1. **A0 (este PR):** ADR, seção 30 da spec, `bots.agent` no banco e no protocolo (só `claude` roda; os outros dois são recusados com mensagem clara).
2. **A1:** a interface `Agent`, com o Claude por trás dela e os testes de hoje passando sem mudança.
3. **A2:** AGY (o protocolo mais próximo do atual), com modelos, esforço, MCP, aprovações e isolamento testados com o `agy` real.
4. **A3:** Codex por `app-server`.
5. **A4:** o seletor de agente no app, a conta e o login de cada agente, e a paridade (rotinas, navegador, desktop, perguntas) conferida agente por agente.

## Consequências

- O trabalho é grande e atravessa protocolo, banco, daemon e app. Cada passo entra junto com código e testes, nunca como PR só de documento.
- O contador de contexto exato, o custo e a fatia do plano são do Claude hoje. Nos outros agentes ficam aproximados ou ausentes até haver fonte oficial.
- O texto da interface fala do "agente" do bot pelo nome do bot, como já fazemos, e não de "Claude".
- Os pontos acima marcados como "não visto" entram na seção 30 e precisam de teste real antes de o agente correspondente ser liberado.
