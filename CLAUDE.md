# Botloft

Workspace desktop Windows-first e open-source para rodar bots Claude Code sempre ligados que colaboram entre si. Daemon Rust (`botloftd`) + app Tauri v2 + React.

## Fonte de verdade

- **`docs/spec.md`** define arquitetura, protocolo, dados, estados e marcos. Leia a seção relevante antes de implementar.
- Se o código precisar divergir da spec, atualize a spec no mesmo commit e explique o motivo na mensagem.
- A seção 19 da spec lista comportamentos do Claude Code que precisam ser verificados. Não trate esses itens como certos: confirme na documentação oficial (code.claude.com/docs) ou com teste real e registre o resultado na spec.

## Regra de autoria

Este projeto é implementação própria. Não copie nem adapte código, testes, textos, schemas ou estrutura de outros projetos de orquestração de agentes. Fontes permitidas: esta spec, documentação oficial do Claude Code e documentação das bibliotecas usadas.

## Layout

```
crates/botloft-core    tipos de domínio, IDs, envelope, tipos do protocolo (exportados via ts-rs)
crates/botloft-store   SQLite, migrations em migrations/NNNN_nome.sql
crates/botloftd        daemon: rpc, supervisor, runtime, chat, courier, tools, platform
app/                   Tauri v2 + React 19 + TS strict + Tailwind v4 + Zustand
docs/                  spec e ADRs (docs/adr/0001: bots headless com chat)
```

## Comandos

```powershell
# Daemon
cargo build -p botloftd
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
cargo run -p botloftd -- serve   # com BOTLOFT_HOME de dev (abaixo)

# Tipos TS gerados a partir do Rust
cargo test -p botloft-core export_bindings

# App
cd app
pnpm install
pnpm tauri dev    # o app acha o daemon pelo BOTLOFT_HOME e o inicia (target\debug\botloftd.exe)
pnpm dev          # só a UI num navegador comum, com o FakeBotloft (src/dev/preview.ts)
pnpm check        # tsc --noEmit + biome check + vitest run
```

Em dev, use `$env:BOTLOFT_HOME = "$PWD\.dev\home"` (caminho absoluto) no daemon e no app, para não tocar na instalação real em `%LOCALAPPDATA%\Botloft`. O `config.toml` de dev fica dentro dessa pasta: o app lê a porta de lá.

## Convenções

- Código, identificadores, comentários, commits e README em **inglês**. Docs em `docs/` podem ser em português.
- Arquivos com no máximo **~300 linhas**. Passou disso, divida por responsabilidade.
- Erros: `thiserror` nos crates de biblioteca, `anyhow` só no binário.
- IDs são ULID com prefixo (`crw_`, `bot_`, `msg_`, `dlv_`, `tsk_`, `cht_`, `apr_`, `att_`). Tempo em ms Unix.
- Tudo que é específico de Windows fica em `crates/botloftd/src/platform/`. O resto do daemon não chama Win32 direto.
- A UI depende só da interface `BotloftApi` (`app/src/lib/api.ts`). Testes de componente usam `FakeBotloft`.
- Nunca edite `app/src/lib/protocol.gen.ts` à mão.
- Commits pequenos, no formato Conventional Commits (`feat(courier): ...`).

## Windows: armadilhas conhecidas

- Cada bot é `claude -p` com `stream-json` no stdin e no stdout (spec 7.4, 8, 9.2). O processo fica calado até a primeira mensagem; fechar o stdin o encerra. O formato de entrada não é documentado: qualquer mudança passa por teste real e vai para a seção 19.
- Bots rodam com `--setting-sources project,local` e `--strict-mcp-config`: nada de hooks, skills, modo de permissão ou MCP pessoais do dono. Aprovações passam pela tool `permission_prompt` do nosso MCP.
- Servidor MCP por HTTP: o Claude Code aborta a request em 60 s e a chamada em 5 min sem resposta, a menos que o `mcp.json` tenha `timeout` no servidor. A aprovação depende disso.
- Só o `claude.exe` nativo roda. O `claude.cmd` do npm é recusado: passar por `cmd.exe` estraga o quoting dos argumentos.
- O bot recebe o ambiente padrão do usuário (`CreateEnvironmentBlock`), nunca o do daemon: em dev o daemon herda variáveis da sessão do Claude Code (`CLAUDE_CODE_MESSAGING_SOCKET`, `ANTHROPIC_BASE_URL`...).
- PATH real pode ter aspas soltas. Separe só por `;`; `std::env::split_paths` trata aspas como agrupamento e engole o resto.
- Todo processo de bot entra num Job Object com kill-on-close e roda sem console (`CREATE_NO_WINDOW`).
- Permissão de arquivo é ACL (SID do usuário atual), não modo Unix.
- Estado em `%LOCALAPPDATA%`, nunca `%APPDATA%` (roaming).
- Cuidado com o limite de 260 caracteres em caminhos de workspace.

## Segurança e privacidade

- Daemon só em `127.0.0.1`.
- Nunca logar token, corpo de mensagem, anexo ou item do chat (resposta, saída de ferramenta) em nível `info` ou acima. Bots podem lidar com dado pessoal e de saúde.
- Tokens de bot ficam no banco só como SHA-256.
- Nada de telemetria.

## Definição de pronto

Uma tarefa só está pronta quando:

1. `cargo fmt`, `clippy -D warnings` e `cargo test` passam (e `pnpm check`, se mexeu no app);
2. comportamento novo tem teste (supervisor, courier e chat testados com `FakeRuntime`);
3. a spec continua verdadeira;
4. o que depende do Claude Code real está listado no PR com o passo de teste manual.

## Ordem de trabalho

Siga os marcos da spec (seção 17): M0 -> M1 -> M2 -> M3 -> M4 -> M4.1 -> M5. Não comece rotinas, caixa de perguntas, busca ou acesso remoto antes do M5.
