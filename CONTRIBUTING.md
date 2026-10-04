# Contributing to Botloft

Thanks for taking a look. Botloft is young (version 0.x) and every kind of help counts: trying it and
reporting what broke, ideas, docs, translations and code.

## Ways to help

- **Try it and tell us.** A bug report with the steps to reproduce it is one of the most useful
  things you can send. Use the [bug report form](https://github.com/httpsphl/botloft/issues/new/choose).
- **Antivirus false positives.** Releases are not code-signed yet (see the
  [code signing policy](docs/code-signing-policy.md)), so an antivirus may flag them by guesswork.
  If yours does, open an issue with the file name, the detection name and the antivirus; we report
  each one to the vendor.
- **Pick an issue.** Issues labeled
  [`good first issue`](https://github.com/httpsphl/botloft/labels/good%20first%20issue) are small and
  self-contained; [`help wanted`](https://github.com/httpsphl/botloft/labels/help%20wanted) ones are
  bigger. Comment on the issue before you start so two people do not do the same work.
- **Translations.** The app speaks English, Brazilian Portuguese and Spanish. Fixes to wording are
  welcome in any of them.
- **Bigger changes.** For a new feature or a change in how something works, open an issue first and
  describe it. It saves you from writing code that does not fit the plan.

## Getting set up

The [Development section of the README](README.md#status) lists what to install and the commands to
build and test. The short version:

```powershell
cargo test --workspace
cd app
pnpm install
pnpm dev          # the UI alone in a browser, with a fake daemon and sample crews
```

`pnpm dev` needs neither Rust nor Claude Code, so it is the quickest way to work on the interface.
Running real bots needs Claude Code installed and signed in.

During development, set `BOTLOFT_HOME` to an absolute path (for example `$PWD\.dev\home`) for both
the daemon and the app, so they never touch a real Botloft installation.

## How the project is organized

```
crates/botloft-core    domain types, IDs, message envelope, protocol types (exported to TypeScript)
crates/botloft-store   SQLite storage, migrations in migrations/NNNN_name.sql
crates/botloftd        the daemon: RPC, supervisor, bot runtime, chat, message delivery, MCP tools
app/                   Tauri v2 + React 19 + TypeScript (strict) + Tailwind v4 + Zustand
docs/                  specification and architecture decision records
```

[`docs/spec.md`](docs/spec.md) is the source of truth for architecture, protocol, data and states.
It is written in Portuguese; a browser translation reads fine, and questions about it are welcome in
an issue. If your change makes the spec wrong, update the spec in the same pull request.

## Rules for code

- **Write it yourself.** Botloft is its own implementation. Do not copy or adapt code, tests, texts
  or schemas from other agent orchestration projects. The spec, the official Claude Code
  documentation and the documentation of the libraries we use are fine sources.
- **English** for code, identifiers, comments and commit messages.
- **Small files.** Keep files under about 300 lines; split by responsibility past that.
- **Errors:** `thiserror` in the library crates, `anyhow` only in the binary.
- **Platform code** (Win32, systemd, launchd) lives in `crates/botloftd/src/platform/`. The rest of
  the daemon does not call the OS directly.
- **The UI talks only to `BotloftApi`** (`app/src/lib/api.ts`). Component tests use `FakeBotloft`.
- **No interface text inside components.** Every string goes in
  `app/src/i18n/{en,pt-BR,es}/<area>.ts`, in all three languages, and is read with `useT()`. The
  TypeScript check flags a missing one. Use plain words: the app never says "daemon" or other jargon
  to the user.
- **Never edit `app/src/lib/protocol.gen.ts` by hand.** It is generated from the Rust types with
  `cargo test -p botloft-core export_bindings`.
- **Privacy.** Never log tokens, message bodies, attachments or chat items at `info` level or
  above: bots may handle personal data. No telemetry.

## Before you open a pull request

1. `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` and
   `cargo test --workspace` pass.
2. If you changed the app: `pnpm check` passes (typecheck, Biome and Vitest).
3. New behavior has a test. The supervisor, message delivery and chat are tested with
   `FakeRuntime`; components with `FakeBotloft`.
4. The spec still describes what the code does.
5. If the change depends on how the real Claude Code behaves, say in the pull request how you tested
   it by hand.

Keep pull requests small, one change each, with a title in
[Conventional Commits](https://www.conventionalcommits.org) form, such as
`fix(chat): keep the draft when a bot restarts`. CI runs on Windows, Ubuntu and macOS and must pass
before merging.

Using an AI assistant to write your contribution is fine. You are still the one who answers for it:
read every line, run the checks and make sure it follows the rules above.

## License

Botloft is licensed under the [Functional Source License 1.1, Apache 2.0 future license](LICENSE)
(FSL-1.1-ALv2). By sending a contribution, you agree that it is licensed under the same terms, and
you allow Phelipe Lorran to also distribute it under other license terms, so the project can change
its license later without asking every contributor. You keep the copyright of your contribution.

## Security

Please do not open a public issue for a security problem. See [SECURITY.md](SECURITY.md).
