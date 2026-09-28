# Botloft

Always-on Claude Code crews for Windows.

Botloft keeps groups of persistent Claude Code bots ("crews") running on your machine and lets
them hand work to each other. A Rust daemon, `botloftd`, runs each bot in its own ConPTY, stores
everything in SQLite and delivers messages through Claude Code's native inbox. A Tauri + React
desktop app shows the live terminals and controls the daemon over JSON-RPC.

> **Status: pre-alpha.** The repository is at milestone M0 (foundation): the workspace, an empty
> app and CI. Nothing runs bots yet. The design lives in [`docs/spec.md`](docs/spec.md).

## How it works

- **The daemon is the source of truth.** Closing the app never stops a bot.
- **Claude Code is the runtime.** Each bot is a real `claude` session with its own workspace,
  memory (`CLAUDE.md`) and permission prompts. There is no custom agent SDK.
- **Terminal and messages are separate channels.** What you type goes through the PTY; messages
  from other bots arrive through Claude Code's inbox and are never injected into the terminal.
- **Durable before delivered.** Every message is written to SQLite before it is sent and is
  retried until it is delivered or marked dead.
- **Local only.** The daemon listens on `127.0.0.1`. No cloud services, no telemetry.

## Layout

```
crates/botloft-core    domain types, IDs, message envelope, protocol types
crates/botloft-store   SQLite storage and migrations
crates/botloftd        the daemon
app/                   Tauri v2 + React desktop app
docs/                  specification and ADRs
```

## Development

Requirements: Windows 10 or 11, Rust stable (MSVC toolchain), Node.js 24 and pnpm. Running real
bots (milestone M2 onwards) also needs Claude Code installed and signed in.

```powershell
# Daemon and libraries
cargo build -p botloftd
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check

# Desktop app
cd app
pnpm install
pnpm tauri dev
pnpm check        # typecheck + Biome + Vitest
```

Set `BOTLOFT_HOME=.dev\home` during development so the daemon never touches a real installation
in `%LOCALAPPDATA%\Botloft`.

## Security model

- The daemon binds to `127.0.0.1` only. The app authenticates with an owner token stored under
  `%LOCALAPPDATA%\Botloft\secrets`, readable only by your Windows user.
- Each bot gets its own token per process start; the database keeps only its SHA-256 hash.
- **Botloft is not a sandbox.** Every bot runs as your Windows user, so isolation between bots is
  cooperative: a bot can read other bots' workspaces if it decides to. Give bots only the
  permissions you would give Claude Code directly.

## Roadmap

| Milestone | Scope |
|---|---|
| M0 | Workspace, empty app, CI |
| M1 | Daemon base: config, storage, RPC, crews and bots |
| M2 | Runtime: ConPTY, supervisor, hooks, terminal replay |
| M3 | Messaging: durable delivery, bot tools, tasks between bots |
| M4 | Desktop app: onboarding, crews, bots, terminals, timeline |
| M5 | Distribution: start with Windows, installer, updater |

## License

Botloft is licensed under the [Apache License 2.0](LICENSE). See [NOTICE](NOTICE) for the copyright notice that goes with redistributions.
