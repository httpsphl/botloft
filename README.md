# Botloft

Always-on Claude Code crews for Windows.

Botloft keeps groups of persistent Claude Code bots ("crews") running on your machine and lets
them hand work to each other. A Rust daemon, `botloftd`, runs each bot as headless Claude Code,
stores everything in SQLite and delivers messages between bots durably. A Tauri + React desktop
app shows each bot as a chat: you talk to it, send it images and files, and approve what it asks
to do.

> **Status: pre-alpha.** Milestones M0 to M4.1 are done: the daemon, the chat, messages and
> tasks between bots. M5 (distribution) is in progress. The design lives in
> [`docs/spec.md`](docs/spec.md).

## Install

1. Install [Claude Code](https://code.claude.com/docs/en/setup) and sign in once.
2. Download `Botloft_<version>_x64-setup.exe` from
   [Releases](https://github.com/httpsphl/botloft/releases) and run it. It installs for your
   Windows user only, with no administrator prompt. The installer is not code-signed yet, so
   Windows SmartScreen may ask you to confirm.
3. Open Botloft. It sets itself up; there is nothing to configure.

Botloft updates itself: when a new version is out, an "Update available" button shows in the
title bar. Uninstalling (Settings > Apps) removes the app and stops Botloft from running in the
background; your bots and their files stay in `%LOCALAPPDATA%\Botloft` and
`%USERPROFILE%\Botloft`.

## How it works

- **The daemon is the source of truth.** Closing the app never stops a bot. The daemon runs as a
  scheduled task of your Windows user: it starts when you sign in and comes back within a minute
  if it dies.
- **Claude Code is the runtime.** Each bot is a real `claude -p` session with its own workspace,
  memory (`CLAUDE.md`) and conversation, talking `stream-json` over stdin and stdout. There is no
  custom agent SDK.
- **One conversation per bot.** Your messages, messages from other bots and notices from the
  daemon all reach the bot in order. What it does comes back as a chat: replies, tool use and
  permission requests you allow or deny.
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
bots also needs Claude Code (the native `claude.exe`) installed and signed in.

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
pnpm bundle       # NSIS installer with the daemon inside
```

Set `BOTLOFT_HOME` to an absolute path such as `$PWD\.dev\home` during development so the daemon
never touches a real installation in `%LOCALAPPDATA%\Botloft`.

The daemon manages its own scheduled task:

```powershell
botloftd service install     # copy botloftd into the data folder, start it now and at every logon
botloftd service status
botloftd service restart
botloftd service uninstall   # stop it and remove the task; bots and data stay
```

Each data folder gets its own task, so a dev daemon installed with `BOTLOFT_HOME` (or `--home`)
never replaces the real one.

## Security model

- The daemon binds to `127.0.0.1` only. The app authenticates with an owner token stored under
  `%LOCALAPPDATA%\Botloft\secrets`, readable only by your Windows user.
- Each bot gets its own token per process start; the daemon keeps only its SHA-256 hash.
- Bots ask in their chat before they run commands or edit files, and wait for you to allow or
  deny each request.
- **Botloft is not a sandbox.** Every bot runs as your Windows user, so isolation between bots is
  cooperative: a bot can read other bots' workspaces if it decides to. Give bots only the
  permissions you would give Claude Code directly.

## Roadmap

| Milestone | Scope |
|---|---|
| M0 | Workspace, empty app, CI |
| M1 | Daemon base: config, storage, RPC, crews and bots |
| M2 | Runtime: supervisor, restarts with backoff, Job Objects |
| M3 | Messaging: durable delivery, bot tools, tasks between bots |
| M4 | Desktop app: onboarding, crews, bots, timeline |
| M4.1 | Headless bots with a chat, approvals, attachments |
| M5 | Distribution: start with Windows, keep awake, installer, updater |

## License

Botloft is licensed under the [Apache License 2.0](LICENSE). See [NOTICE](NOTICE) for the copyright notice that goes with redistributions.
