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

On macOS, Botloft is not notarized by Apple: there is no Apple Developer account for it yet. The app
carries an ad-hoc signature only, so macOS blocks its first opening until you allow it in System
Settings > Privacy & Security > Open Anyway. The Linux packages (`.deb` and AppImage) are not
signed either. On every system, updates are signed with Botloft's own updater key (below).

## How releases are made

- Every release is built by GitHub Actions ([`release.yml`](../.github/workflows/release.yml)) from
  a `vX.Y.Z` tag on `main`, on GitHub's own runners. Nothing is built on a maintainer's computer.
- The workflow opens a draft release. A person reads it and publishes it: no release goes out
  without that manual approval.
- Each download (the Windows installers, the Linux packages and the macOS disk image and update)
  gets a build provenance attestation, and the release notes list its SHA-256. To
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

On Linux and macOS (preview since 0.8.0):

- **Linux:** the `.deb` installs the app in `/usr/bin` (installing it asks for your password, as any
  package does); the AppImage stays wherever you put it. Botloft's data is in
  `~/.local/share/Botloft`. A systemd user service, `botloft.service` in `~/.config/systemd/user`,
  keeps the bots running; it starts when you sign in if Botloft starts with the system. When
  Botloft opens its window at sign-in, it adds `~/.config/autostart/botloft.desktop`.
- **macOS:** the app goes wherever you drag it, usually Applications. Botloft's data is in
  `~/Library/Application Support/Botloft`. A launch agent,
  `~/Library/LaunchAgents/io.github.httpsphl.botloft.daemon.plist`, keeps the bots running; when
  Botloft opens its window at sign-in, it adds `io.github.httpsphl.botloft.app.plist` next to it.
- Bots keep their work in `~/Botloft`, or in the folder you pick for a crew. Keeping the computer
  awake while bots work is Windows-only for now.

## Uninstalling

Settings > Apps > Installed apps > Botloft > Uninstall removes the app, stops the background
service, and removes its scheduled task and the startup entry. Your bots, their conversations and
their files stay in `%LOCALAPPDATA%\Botloft` and `%USERPROFILE%\Botloft`; delete those two folders
to remove everything.

On Linux and macOS, removing the app does not stop the background service, so stop it first:

```
~/.local/share/Botloft/bin/botloftd service uninstall                    # Linux
~/"Library/Application Support/Botloft/bin/botloftd" service uninstall   # macOS
```

Then remove the app (`sudo apt remove botloft`, the AppImage file, or the app in Applications) and,
on macOS, `~/Library/LaunchAgents/io.github.httpsphl.botloft.app.plist` if it is there. Your bots
and data stay in the data folder above and in `~/Botloft`; delete those to remove everything.
