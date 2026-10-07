<p align="center">
  <img src="docs/assets/mascot.svg" width="120" alt="The Botloft mascot: a small orange flame with eyes">
</p>

<h1 align="center">Botloft</h1>

<p align="center">
  <b>A crew of Claude Code bots that stays on and works together on your computer.</b>
</p>

<p align="center">
  <a href="https://github.com/httpsphl/botloft/releases/latest"><img src="https://img.shields.io/github/v/release/httpsphl/botloft?label=release&color=ff7a59" alt="Latest release"></a>
  <a href="https://github.com/httpsphl/botloft/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/httpsphl/botloft/ci.yml?branch=main&label=CI" alt="CI status"></a>
  <img src="https://img.shields.io/badge/Windows%20%7C%20Linux%20%7C%20macOS-0078d4" alt="Windows, Linux and macOS">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-FSL--1.1--ALv2-3b3b38" alt="FSL-1.1-ALv2 license"></a>
</p>

<p align="center">
  <a href="https://github.com/httpsphl/botloft/releases/latest"><b>Download</b></a>
  &nbsp;·&nbsp;
  <a href="#what-you-can-do">Features</a>
  &nbsp;·&nbsp;
  <a href="#how-it-works">How it works</a>
  &nbsp;·&nbsp;
  <a href="#faq">FAQ</a>
  &nbsp;·&nbsp;
  <a href="#contributing">Contributing</a>
</p>

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/assets/hero-dark.png">
  <img src="docs/assets/hero-light.png" alt="Botloft with the Research crew open: seven bots, each with its own color, state and job">
</picture>

Botloft keeps a crew of [Claude Code](https://code.claude.com) bots running on your computer. Each bot has
its own folder, memory and conversation. They pass work to each other, keep going after you close the
window and pick up where they left off after a restart. You talk to each one like in a chat app, and
you decide what they may do.

## What you can do

- **Start a crew from a goal.** Say what the crew is for. Its Chief plans the work and suggests the
  bots it needs; you create each one with a click, or say why not.
- **Chat with every bot.** Replies stream in as they are written. Send images and files. When a bot
  wants to run a command, edit a file or open a website, a card asks you first, unless you told that
  bot it may.
- **Let them hand work to each other.** Bots send messages and tasks to each other. Every message is
  saved before it is sent and retried until it arrives.
- **Watch them work.** See every page a bot opens in its browser, live, and take control to sign in
  for it. Sites that refuse to sign in there, like Google, take one click more: sign in in a real
  window of the bot's browser, close it, and the bot keeps the login. Watch the screens it designs
  take shape while it writes them. Open the files it made.
- **Schedule routines.** Weekday mornings, every two hours or any cron schedule, with missed runs
  and overlaps handled.
- **Leave them running.** Bots keep working after you close Botloft. They start when you sign in,
  come back soon after a crash and keep the computer awake while they work. Botloft
  waits near the clock and tells you when a bot needs you.
- **Choose per bot.** How much it asks before acting, and which Claude model it uses. See how much
  of your plan's usage is left.
- **Back up and move.** Save a sealed backup file, or sign in with your e-mail (no password) to keep
  a light copy in the cloud and restore it on another computer. The server cannot read it. Turn on
  automatic copies, every day or week and only when something changed, if you want them to go by
  themselves (Windows for now).

<table>
  <tr>
    <td width="50%"><img src="docs/assets/chat-browser.png" alt="Scout's chat with a request to use the browser, next to the live browser where Scout types in a search box"><br><sub><b>Chat and live browser.</b> Scout asks before using a new site, and you watch it browse.</sub></td>
    <td width="50%"><img src="docs/assets/screens.png" alt="Designer writing a bakery landing page that takes shape in the screens area, with its cursor on the part being written"><br><sub><b>Screens.</b> Designer's pages take shape while it writes them.</sub></td>
  </tr>
  <tr>
    <td width="50%"><img src="docs/assets/chief.png" alt="The Chief suggesting a new bot, Designer, with its name, model, role and instructions ready to edit"><br><sub><b>A Chief for each crew.</b> It suggests the bots the work needs.</sub></td>
    <td width="50%"><img src="docs/assets/routines.png" alt="The crew's routines: a morning summary on weekdays, a link check every two hours, weekly numbers and a nightly archive"><br><sub><b>Routines.</b> Work that runs on a schedule, with the last result at a glance.</sub></td>
  </tr>
</table>

Botloft speaks English, Portuguese (Brazil) and Spanish, follows your light or dark theme and updates
itself.

## Get started

You need [Claude Code](https://code.claude.com/docs/en/setup), installed and signed in once with
your Claude account. Bots run on your own Claude plan.

1. Download Botloft for your computer from the
   [latest release](https://github.com/httpsphl/botloft/releases/latest):
   - **Windows 10 or 11 (64-bit):** `Botloft_<version>_x64-setup.exe`. Run it; it installs for your
     Windows user only, with no administrator prompt. The installer is not code-signed yet, so
     Windows SmartScreen may ask you to confirm ("More info", then "Run anyway").
   - **Linux (64-bit, preview):** `Botloft_<version>_amd64.deb` for Debian and Ubuntu, or
     `Botloft_<version>_amd64.AppImage` for other distributions (make it executable first).
   - **macOS on Apple Silicon (preview):** `Botloft_<version>_aarch64.dmg`. Drag Botloft to
     Applications and open it once. Apple has not notarized it yet, so macOS blocks it the first
     time: allow it in System Settings > Privacy & Security > Open Anyway.

   The [code signing policy](docs/code-signing-policy.md) says how releases are made and how to
   check your download.
2. Open Botloft. It sets itself up; there is nothing to configure.
3. Create your first crew and tell it what it is for.

When a new version is out, an **Update available** button shows in the title bar.

On Windows, uninstalling (Settings > Apps) removes the app and stops Botloft from running in the
background. On Linux and macOS, stop it first with
`~/.local/share/Botloft/bin/botloftd service uninstall` (Linux) or
`~/"Library/Application Support/Botloft/bin/botloftd" service uninstall` (macOS), then remove the
app. Your bots and their files stay either way (see
[Where are my bots and their files?](#faq)).

## How it works

```mermaid
flowchart LR
  app["Botloft app<br/>Tauri + React"] -- "JSON-RPC over WebSocket<br/>127.0.0.1 only" --> daemon["botloftd<br/>Rust daemon"]
  daemon --> db[("SQLite")]
  daemon -- "stdin and stdout<br/>stream-json" --> bots["One claude -p<br/>per bot"]
  bots -- "Botloft's MCP tools<br/>messages, tasks, requests" --> daemon
```

- **The daemon is the source of truth.** Closing the app never stops a bot. `botloftd` runs as a
  scheduled task of your Windows user, a systemd user service on Linux or a launch agent on macOS:
  it starts when you sign in and comes back soon if it dies.
- **Claude Code is the runtime.** Each bot is a real `claude -p` session with its own workspace,
  memory (`CLAUDE.md`) and conversation. There is no custom agent SDK and no API key to manage.
- **One conversation per bot.** Your messages, messages from other bots and notices from the daemon
  reach the bot in order. What it does comes back as a chat: replies, tool use and requests you
  allow or deny.
- **Durable before delivered.** Every message is written to SQLite before it is sent and retried
  until it is delivered or marked as failed, which you can retry.

The full design, from the protocol to the states of a bot, lives in [`docs/spec.md`](docs/spec.md)
(in Portuguese).

## Privacy and security

- **Local first.** The daemon listens on `127.0.0.1`, and there is no telemetry. An account is
  optional: without one, Botloft uses the network for nothing beyond updates and what bots send to
  Claude, which goes through Claude Code as when you use it yourself. With one, the server only ever
  holds a backup sealed with a password that never leaves your computer (see the FAQ).
- **You decide what bots may do.** Per bot: ask before anything, accept edits, plan only, or decide on
  its own what needs your OK. Unless it decides on its own, a bot also asks before its first visit
  to each website.
- **Tokens stay safe.** The app authenticates with an owner token readable only by your user account.
  Each bot gets its own token per start, and the daemon keeps only its SHA-256 hash.
- **Botloft is not a sandbox.** Every bot runs as your user account, so isolation between bots is
  cooperative: a bot can read other bots' folders if it decides to. Give bots only the permissions
  you would give Claude Code directly.

What Botloft sends over the network, what it changes on your computer and how to remove it are in
the [code signing policy](docs/code-signing-policy.md).

## FAQ

<details>
<summary><b>Does Botloft cost anything?</b></summary>

No. Botloft is free to use, at home or at work, and its source code is public (see
[License](#license)). The bots run on Claude Code with
your own Claude account, so they use your plan's limits; Botloft shows how much is left.
</details>

<details>
<summary><b>Do I need to keep the window open?</b></summary>

No. The bots run in the background, even after you close Botloft, and start when you sign in if you
want.
Settings has a switch to stop them when you close the app instead.
</details>

<details>
<summary><b>Where are my bots and their files?</b></summary>

Each bot has a folder under `Botloft\<crew>\<bot>` in your home folder (`%USERPROFILE%` on Windows,
`~` on Linux and macOS), and each crew a shared work folder that you can point at a project of
yours. Botloft's own data (SQLite, logs, secrets) is in `%LOCALAPPDATA%\Botloft` on Windows,
`~/.local/share/Botloft` on Linux and `~/Library/Application Support/Botloft` on macOS.
</details>

<details>
<summary><b>What does the optional account keep, and who can read it?</b></summary>

You sign in with a link sent to your e-mail; there is no account password. The cloud copy is the
light one: your crews' structure, the bots' memory and the routines, not the chats or files. Botloft
seals it on your computer with the backup password before sending, so the server stores bytes it
cannot open. It also knows your e-mail, the computers you signed in on, and the size and date of each
copy. Lose the backup password and the cloud copies cannot be recovered. Nothing is sent unless you
send it or turn on automatic copies. Those go every day or week, only when something changed; for
them Windows keeps your backup password in its credential store, on this computer only, so use a
password just for this. You can delete the copies or the whole account from the app. The server is this
repository's `botloft-cloud` crate: you can [run your own](docs/cloud-deploy.md) and point the app at
it.
</details>

<details>
<summary><b>Why does Windows or macOS warn me when I install it?</b></summary>

The Windows installer is not code-signed yet, so SmartScreen does not know its publisher, and Apple
has not notarized the macOS app, so Gatekeeper blocks its first opening until you allow it. Each release is
built by GitHub Actions from this repository, and updates are signed and checked by the app before
they install. The [code signing policy](docs/code-signing-policy.md) says how releases are made and
how to check a download.
</details>

<details>
<summary><b>Does it run on macOS or Linux?</b></summary>

Yes, as a preview since 0.8.0: Linux (64-bit, `.deb` or AppImage) and macOS on Apple Silicon. Intel
Macs are not supported yet.
Windows came first and has had the most use; if something does not work elsewhere, please
[open an issue](https://github.com/httpsphl/botloft/issues).
</details>

<details>
<summary><b>Is Botloft made by Anthropic?</b></summary>

No. Botloft is an independent project. It uses Claude Code as installed on your computer
and is not affiliated with or endorsed by Anthropic.
</details>

## Status

Botloft is young (version 0.x) and moves fast. The daemon, the chat, crews with a Chief, messages
and tasks between bots, routines, the live browser, screens, "always allow" for requests, the
installer and updates are in place. Planned next: a question box where bots ask you things, search,
and a code-signed installer. `main` can be ahead of the latest release.

<details>
<summary><b>Development</b></summary>

Requirements: Rust stable (the MSVC toolchain on Windows), Node.js 24 and pnpm. On Linux, the app
also needs the desktop libraries Tauri uses (on Ubuntu: `libwebkit2gtk-4.1-dev`,
`libayatana-appindicator3-dev`, `librsvg2-dev`, `libxdo-dev`). Running real bots also needs Claude
Code (the native `claude` executable) installed and signed in.

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
pnpm dev          # the UI alone in a browser, with a fake daemon and sample crews
pnpm check        # typecheck + Biome + Vitest
pnpm bundle       # this system's packages with the daemon inside: NSIS, .deb and AppImage, or .dmg
```

Set `BOTLOFT_HOME` to an absolute path such as `$PWD\.dev\home` during development so the daemon
never touches a real installation in `%LOCALAPPDATA%\Botloft`.

The daemon manages its own scheduled task (a systemd user service on Linux, a launch agent on
macOS):

```powershell
botloftd service install     # copy botloftd into the data folder, start it now and at every logon
botloftd service status
botloftd service restart
botloftd service uninstall   # stop it and remove the task; bots and data stay
```

Each data folder gets its own task, so a dev daemon installed with `BOTLOFT_HOME` (or `--home`)
never replaces the real one.

```
crates/botloft-core    domain types, IDs, message envelope, protocol types
crates/botloft-store   SQLite storage and migrations
crates/botloftd        the daemon
app/                   Tauri v2 + React desktop app
docs/                  specification and ADRs
```
</details>

## Contributing

Contributions are welcome: bug reports, ideas, translations and code. Start with
[CONTRIBUTING.md](CONTRIBUTING.md); issues labeled
[`good first issue`](https://github.com/httpsphl/botloft/labels/good%20first%20issue) are a good
first step. Security problems go through [SECURITY.md](SECURITY.md), not public issues.

## License

Botloft is licensed under the [Functional Source License 1.1, Apache 2.0 future license](LICENSE)
(FSL-1.1-ALv2). In short:

- You may use Botloft for anything, including at work, read and change its code, and send
  contributions.
- You may not offer Botloft, or a copy or a change of it, as a commercial product or service that
  competes with it.
- Two years after each version is released, that version also becomes available under the
  [Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0).

Versions up to 0.9.1 were released under the Apache License 2.0 and stay under it. The
[LICENSE](LICENSE) file is what counts; this summary is not legal advice.

The screenshots show sample crews from the app's preview, not
real data.
