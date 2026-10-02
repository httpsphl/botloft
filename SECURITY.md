# Security policy

## Reporting a problem

Please report security problems privately, through GitHub:
[Report a vulnerability](https://github.com/httpsphl/botloft/security/advisories/new). Do not open a
public issue.

Say what you found, how to reproduce it and which version you used. You will get an answer within a
week. Once a fix is out, the advisory is published with credit to you, unless you prefer otherwise.

## Supported versions

Only the [latest release](https://github.com/httpsphl/botloft/releases/latest) gets security fixes.
The app updates itself, so staying on it is one click.

## Scope

Botloft runs Claude Code bots on your computer, with your account. Some things are by design and not
vulnerabilities:

- Bots run as your user, without a sandbox. Keeping them apart is cooperative: a bot that may run
  any command (or one set to skip permission prompts) can read what your user can read, including
  other bots' folders and Botloft's own data. The app warns about this before you turn that on.
- The background service listens on `127.0.0.1` only and asks every client for a token, which only
  your user can read. Other programs running as your user are trusted as much as you are.

A way around what a bot was allowed to do, a page or a message that gets a bot past a permission
prompt, or a program reaching the service without the token is in scope and very welcome.

Antivirus detections of the releases are not security reports: see the
[code signing policy](docs/code-signing-policy.md) and open a normal issue.
