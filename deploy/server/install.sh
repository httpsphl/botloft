#!/bin/sh
# Installs Botloft on a Linux server (spec 42.3): the daemon, a user of its
# own, a system service that starts at boot, and Claude Code for the agents.
# Then it says how to sign in to Claude.
#
#   curl -fsSL https://github.com/httpsphl/botloft/releases/latest/download/install.sh | sh
#
# Run it as root, or as a user with sudo. Running it again updates the daemon
# and keeps the data and the settings.
#
#   sh install.sh --uninstall           stops Botloft and removes the daemon; the data stays
#   sh install.sh --uninstall --purge   also removes the data and the botloft user
#
# Environment, for special cases and tests:
#   BOTLOFT_VERSION=0.22.0      that release instead of the latest
#   BOTLOFT_FROM=/path/botloftd  a daemon already on disk instead of a download
#   BOTLOFT_NO_SERVICE=1         no system service (a container without systemd)
#   BOTLOFT_NO_CLAUDE=1          do not install Claude Code
set -eu

REPO="httpsphl/botloft"
BIN="/usr/local/bin/botloftd"
DATA="/var/lib/botloft"
HOME_DIR="$DATA/home"
WORK_DIR="$DATA/workspaces"
ACCOUNT="botloft"
UNIT="/etc/systemd/system/botloft.service"
PORT="45710"

say() { printf '%s\n' "$*"; }
fail() {
	printf 'botloft: %s\n' "$*" >&2
	exit 1
}

# Root does it all; anyone else goes through sudo, command by command, so the
# script also works when it comes through a pipe.
if [ "$(id -u)" -eq 0 ]; then
	SUDO=""
elif command -v sudo >/dev/null 2>&1; then
	SUDO="sudo"
else
	fail "run this as root, or install sudo"
fi
as_root() { $SUDO "$@"; }

# Runs a shell command as the botloft user, in its own login environment
# (its home, its PATH).
as_botloft() { as_root su -l "$ACCOUNT" -c "$1"; }

has_systemd() { [ -d /run/systemd/system ] && command -v systemctl >/dev/null 2>&1; }

uninstall() {
	purge="$1"
	if has_systemd && [ -f "$UNIT" ]; then
		as_root systemctl disable --now botloft.service >/dev/null 2>&1 || true
		as_root rm -f "$UNIT"
		as_root systemctl daemon-reload
	fi
	as_root rm -f "$BIN"
	if [ "$purge" = "yes" ]; then
		if id "$ACCOUNT" >/dev/null 2>&1; then
			if command -v userdel >/dev/null 2>&1; then
				as_root userdel "$ACCOUNT" || true
			else
				as_root deluser "$ACCOUNT" || true
			fi
		fi
		as_root rm -rf "$DATA"
		say "Botloft, its data and the $ACCOUNT user were removed."
	else
		say "Botloft was removed. Its data is still in $DATA (remove it with --uninstall --purge)."
	fi
}

case "${1:-}" in
--uninstall)
	if [ "${2:-}" = "--purge" ]; then uninstall yes; else uninstall no; fi
	exit 0
	;;
"") ;;
*) fail "unknown option $1 (use --uninstall, or --uninstall --purge)" ;;
esac

[ "$(uname -s)" = "Linux" ] || fail "this installer is for Linux servers"
case "$(uname -m)" in
x86_64 | amd64) ARCH="x86_64" ;;
aarch64 | arm64) ARCH="aarch64" ;;
*) fail "no Botloft build for $(uname -m) yet: only x86_64 and aarch64" ;;
esac

fetch() {
	if command -v curl >/dev/null 2>&1; then
		curl -fsSL "$1" -o "$2"
	elif command -v wget >/dev/null 2>&1; then
		wget -qO "$2" "$1"
	else
		fail "install curl or wget first"
	fi
}

sha256_of() {
	if command -v sha256sum >/dev/null 2>&1; then
		sha256sum "$1" | cut -d' ' -f1
	else
		shasum -a 256 "$1" | cut -d' ' -f1
	fi
}

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

# 1. The daemon, checked against its SHA-256.
if [ -n "${BOTLOFT_FROM:-}" ]; then
	cp "$BOTLOFT_FROM" "$TMP/botloftd"
else
	if [ -n "${BOTLOFT_VERSION:-}" ]; then
		BASE="https://github.com/$REPO/releases/download/v$BOTLOFT_VERSION"
	else
		BASE="https://github.com/$REPO/releases/latest/download"
	fi
	FILE="botloftd-linux-$ARCH.tar.gz"
	say "Downloading $FILE..."
	fetch "$BASE/$FILE" "$TMP/$FILE"
	fetch "$BASE/$FILE.sha256" "$TMP/$FILE.sha256"
	expected="$(cut -d' ' -f1 <"$TMP/$FILE.sha256")"
	[ "$(sha256_of "$TMP/$FILE")" = "$expected" ] || fail "the download does not match its SHA-256; nothing was installed"
	tar -xzf "$TMP/$FILE" -C "$TMP" botloftd
fi
was_running="no"
if has_systemd && as_root systemctl is-active --quiet botloft.service 2>/dev/null; then
	was_running="yes"
fi
as_root install -m 0755 "$TMP/botloftd" "$BIN"
say "Installed $("$BIN" --version)."

# 2. A user of its own. The agents need a shell for their commands.
if ! id "$ACCOUNT" >/dev/null 2>&1; then
	shell="/bin/bash"
	[ -x "$shell" ] || shell="/bin/sh"
	if command -v useradd >/dev/null 2>&1; then
		as_root useradd --system --user-group --home-dir "$DATA" --create-home --shell "$shell" "$ACCOUNT"
	else
		# BusyBox (Alpine).
		as_root addgroup -S "$ACCOUNT"
		as_root adduser -S -D -H -h "$DATA" -s "$shell" -G "$ACCOUNT" "$ACCOUNT"
	fi
	say "Created the $ACCOUNT user."
fi
as_root mkdir -p "$HOME_DIR" "$WORK_DIR"
as_root chown -R "$ACCOUNT:$ACCOUNT" "$DATA"
as_root chmod 0700 "$HOME_DIR"

# 3. The settings, kept when they are already there.
if ! as_root test -f "$HOME_DIR/config.toml"; then
	as_root sh -c "cat >'$HOME_DIR/config.toml'" <<EOF
# Botloft on a server (spec 42). After a change: systemctl restart botloft
server = true
port = $PORT
workspaces_root = "$WORK_DIR"
EOF
	as_root chown "$ACCOUNT:$ACCOUNT" "$HOME_DIR/config.toml"
	as_root chmod 0600 "$HOME_DIR/config.toml"
fi

# 4. Claude Code, which the agents run, from its official installer.
if [ -z "${BOTLOFT_NO_CLAUDE:-}" ]; then
	# $HOME is the botloft user's, so it expands in that user's shell.
	# shellcheck disable=SC2016
	if as_botloft 'command -v claude >/dev/null 2>&1 || [ -x "$HOME/.local/bin/claude" ]'; then
		say "Claude Code is already there."
	else
		command -v curl >/dev/null 2>&1 || fail "install curl first: Claude Code's installer needs it"
		say "Installing Claude Code for the agents..."
		as_botloft 'curl -fsSL https://claude.ai/install.sh | bash'
	fi
fi

# 5. The system service: it starts at boot, comes back after a crash, and
# keeps the agents within most of the memory, so the rest of the server lives.
if [ -z "${BOTLOFT_NO_SERVICE:-}" ] && has_systemd; then
	as_root sh -c "cat >'$UNIT'" <<EOF
[Unit]
Description=Botloft, agents that keep working
After=network-online.target
Wants=network-online.target

[Service]
Type=exec
User=$ACCOUNT
Group=$ACCOUNT
Environment=BOTLOFT_HOME=$HOME_DIR
ExecStart=$BIN serve
Restart=on-failure
RestartSec=5
KillMode=control-group
TimeoutStopSec=20
MemoryMax=90%

[Install]
WantedBy=multi-user.target
EOF
	as_root systemctl daemon-reload
	if [ "$was_running" = "yes" ]; then
		as_root systemctl restart botloft.service
	else
		as_root systemctl enable --now botloft.service
	fi
	say "Botloft is running as a service (systemctl status botloft)."
else
	say "No system service was set up. Start Botloft with:"
	say "  sudo -u $ACCOUNT BOTLOFT_HOME=$HOME_DIR $BIN serve"
fi

say ""
say "Next: sign in to Claude for the agents."
say "  sudo -u $ACCOUNT -i claude auth login"
say "It shows a link. Open it on any device, sign in, and paste the code it gives you."
