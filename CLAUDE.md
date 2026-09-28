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
crates/botloftd        daemon: rpc, supervisor, runtime, terminal, courier, tools, hooks, platform
app/                   Tauri v2 + React 19 + TS strict + Tailwind v4 + Zustand + xterm.js
docs/                  spec e ADRs
```

## Comandos

```powershell
# Daemon
cargo build -p botloftd
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
cargo run -p botloftd -- serve --config .dev\config.toml   # daemon isolado de dev

# Tipos TS gerados a partir do Rust
cargo test -p botloft-core export_bindings

# App
cd app
pnpm install
pnpm tauri dev
pnpm check        # tsc --noEmit + biome check + vitest run
```

Em dev, use `BOTLOFT_HOME=.dev\home` para não tocar na instalação real em `%LOCALAPPDATA%\Botloft`.

## Convenções

- Código, identificadores, comentários, commits e README em **inglês**. Docs em `docs/` podem ser em português.
- Arquivos com no máximo **~300 linhas**. Passou disso, divida por responsabilidade.
- Erros: `thiserror` nos crates de biblioteca, `anyhow` só no binário.
- IDs são ULID com prefixo (`crw_`, `bot_`, `msg_`, `dlv_`, `tsk_`). Tempo em ms Unix.
- Tudo que é específico de Windows fica em `crates/botloftd/src/platform/`. O resto do daemon não chama Win32 direto.
- A UI depende só da interface `BotloftApi` (`app/src/lib/api.ts`). Testes de componente usam `FakeBotloft`.
- Nunca edite `app/src/lib/protocol.gen.ts` à mão.
- Commits pequenos, no formato Conventional Commits (`feat(courier): ...`).

## Windows: armadilhas conhecidas

- O inbox do Claude Code no Windows é um **named pipe** criado pelo próprio Claude Code. O daemon é cliente. A primeira linha tem que ser o auth com `CLAUDE_CODE_MESSAGING_TOKEN`, senão a conexão é descartada. Abra o pipe só com a mensagem pronta.
- Hooks usam **exec form** (`"args": [...]`) chamando `botloftd.exe hook <evento>`. Nada de hook em bash ou PowerShell. O subcomando sai sempre com 0 e nunca escreve no stdout.
- `claude.cmd` (npm) precisa de `cmd.exe /d /s /c`; `claude.exe` (instalador nativo) roda direto.
- ConPTY não tem fd para `poll`: leitura da PTY em thread dedicada + canal.
- Todo processo de bot entra num Job Object com kill-on-close.
- Permissão de arquivo é ACL (SID do usuário atual), não modo Unix.
- Estado em `%LOCALAPPDATA%`, nunca `%APPDATA%` (roaming).
- Cuidado com o limite de 260 caracteres em caminhos de workspace.

## Segurança e privacidade

- Daemon só em `127.0.0.1`.
- Nunca logar token, corpo de mensagem ou saída de terminal em nível `info` ou acima. Bots podem lidar com dado pessoal e de saúde.
- Tokens de bot ficam no banco só como SHA-256.
- Nada de telemetria.

## Definição de pronto

Uma tarefa só está pronta quando:

1. `cargo fmt`, `clippy -D warnings` e `cargo test` passam (e `pnpm check`, se mexeu no app);
2. comportamento novo tem teste (supervisor, courier e terminal testados com `FakeRuntime`);
3. a spec continua verdadeira;
4. o que depende do Claude Code real está listado no PR com o passo de teste manual.

## Ordem de trabalho

Siga os marcos da spec (seção 17): M0 -> M1 -> M2 -> M3 -> M4 -> M5. Não comece rotinas, caixa de perguntas, busca ou acesso remoto antes do M5.
