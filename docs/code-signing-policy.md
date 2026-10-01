# Code signing policy

This page says how Botloft's releases are made, who may release them, what Botloft sends over the
network and what it changes on your computer.

## Status

Botloft's releases are **not code-signed yet**. Until they are, Windows SmartScreen may ask you to
confirm when you run the installer, and an antivirus may flag it by guesswork (a "machine-learning"
or "heuristic" detection), as happens with new programs that have no signature. If that happens to
you, please [open an issue](https://github.com/httpsphl/botloft/issues); we report every false
positive to the vendor.

The release pipeline is ready to sign `Botloft.exe`, `botloftd.exe`, the setup window, the installer
and the uninstaller as soon as a certificate is in place. All of them carry the product name
"Botloft" and the version of the release.

Botloft does not have a code signing certificate yet. When it gets one, this page will say whose
name is on it, and every release from then on will be signed with it.

## How releases are made

- Every release is built by GitHub Actions ([`release.yml`](../.github/workflows/release.yml)) from
  a `vX.Y.Z` tag on `main`, on GitHub's own runners. Nothing is built on a maintainer's computer.
- The workflow opens a draft release. A person reads it and publishes it: no release goes out
  without that manual approval.
- Each installer gets a build provenance attestation, and the release notes list its SHA-256. To
  check that a download came from this repository's workflow, with the
  [GitHub CLI](https://cli.github.com/):

  ```
  gh attestation verify Botloft_<version>_x64-setup.exe --repo httpsphl/botloft
  ```

  This applies to releases after 0.6.0.
- Updates are signed with Botloft's own updater key. The app checks the signature, and that it was
  made for the version announced, before it installs anything.

## Team roles

- **Committers and reviewers:** [Phelipe Lorran](https://github.com/httpsphl)
- **Approvers:** [Phelipe Lorran](https://github.com/httpsphl)

## Privacy

Botloft has no telemetry, no analytics and no accounts of its own. The background service listens
on `127.0.0.1` only. What goes over the network:

- **Update check.** When the app opens, and every 6 hours while it is open, it asks GitHub for the
  latest release's `latest.json`. It downloads an update only when you click **Update now**. No
  information about you or your bots is sent.
- **Your bots.** Each bot is Claude Code running on your computer with your own Claude account. What
  it sends to Anthropic, and the websites it opens when its work needs them, are the same as when
  you use Claude Code yourself.

## What Botloft changes on your computer

- It installs for your Windows user only, under `%LOCALAPPDATA%\Botloft`, with shortcuts in the
  Start menu and on the desktop. It never asks for an administrator.
- It registers a scheduled task for your user, named "Botloft", that keeps the bots running: it
  starts when you sign in and brings the background service back if it stops. Settings > General
  turns off starting with Windows, or stops the bots when you close the app.
- While the bots start with Windows and Botloft stays near the clock or opens its window at sign-in,
  the app adds itself to your user's startup programs. Settings > General turns each of these off.
- While a bot is working, it asks Windows not to put the computer to sleep. Settings > General turns
  this off.
- Bots keep their work in `%USERPROFILE%\Botloft`, or in the folder you pick for a crew.

## Uninstalling

Settings > Apps > Installed apps > Botloft > Uninstall removes the app, stops the background
service, and removes its scheduled task and the startup entry. Your bots, their conversations and
their files stay in `%LOCALAPPDATA%\Botloft` and `%USERPROFILE%\Botloft`; delete those two folders
to remove everything.
