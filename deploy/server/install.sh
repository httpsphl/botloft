#!/bin/sh
# Installs Botloft on a Linux server (spec 42.3): the daemon, a system
# service that starts at boot, a user for the daemon and another for the
# agents (42.5), Claude Code for the agents, and sshd's way in for the app's
# own tunnel (42.6). Then it says how to sign in to Claude.
#
#   curl -fsSL https://github.com/httpsphl/botloft/releases/latest/download/install.sh | sh
#
# Run it as root, or as a user with sudo. Running it again updates the daemon
# and keeps the data and the settings.
#
#   sh install.sh --uninstall           stops Botloft and removes the daemon; the data stays
#   sh install.sh --uninstall --purge   also removes the data and both users
#
# Environment, for special cases and tests:
#   BOTLOFT_VERSION=0.22.0      that release instead of the latest
#   BOTLOFT_FROM=/path/botloftd  a daemon already on disk instead of a download
#   BOTLOFT_NO_SERVICE=1         no system service (a container without systemd)
#   BOTLOFT_NO_CLAUDE=1          do not install Claude Code
#   BOTLOFT_NO_AUTO_UPDATE=1     no hourly `botloftd update` (spec 42.11)
#   BOTLOFT_NO_TUNNEL=1          the app does not open its own SSH tunnel (42.6)
#
# How much of the server Botloft may take (spec 42.4); later, sudo botloftd limits:
#   BOTLOFT_MEMORY=4G            most memory for Botloft and its agents (default 50%)
#   BOTLOFT_CPUS=2               most processors they keep busy (default none)
#   BOTLOFT_AGENTS=6             most agents awake at once (default no limit)
set -eu

REPO="httpsphl/botloft"
BIN="/usr/local/bin/botloftd"
DATA="/var/lib/botloft"
HOME_DIR="$DATA/home"
WORK_DIR="$DATA/workspaces"
# The daemon's user, which alone reads the data folder: the owner's token,
# the agents' keys, the cloud and phone secrets.
ACCOUNT="botloft"
# The agents' user: their commands run as it, and it keeps the Claude login.
AGENTS="botloft-agents"
AGENTS_HOME="/var/lib/botloft-agents"
UNIT="/etc/systemd/system/botloft.service"
# Where `systemctl set-property` keeps the limits (`botloftd limits`).
LIMITS_DIR="/etc/systemd/system.control/botloft.service.d"
# `botloftd update` every hour, as root (spec 42.11).
UPDATE_UNIT="/etc/systemd/system/botloft-update.service"
UPDATE_TIMER="/etc/systemd/system/botloft-update.timer"
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

# Runs a shell command as the agents' user, in its own login environment
# (its home, its PATH).
as_agents() {
	if command -v runuser >/dev/null 2>&1; then
		as_root runuser -l "$AGENTS" -c "$1"
	elif command -v su >/dev/null 2>&1; then
		as_root su -l "$AGENTS" -c "$1"
	else
		fail "install su or runuser (util-linux) first"
	fi
}

has_systemd() { [ -d /run/systemd/system ] && command -v systemctl >/dev/null 2>&1; }

# A system user with a group of its own: add_user <name> <home> <shell>.
add_user() {
	if id "$1" >/dev/null 2>&1; then
		return 0
	fi
	if command -v useradd >/dev/null 2>&1; then
		as_root useradd --system --user-group --home-dir "$2" --no-create-home --shell "$3" "$1"
	else
		# BusyBox (Alpine).
		as_root addgroup -S "$1"
		as_root adduser -S -D -H -h "$2" -s "$3" -G "$1" "$1"
	fi
	say "Created the $1 user."
}

remove_user() {
	if ! id "$1" >/dev/null 2>&1; then
		return 0
	fi
	if command -v userdel >/dev/null 2>&1; then
		as_root userdel "$1" || true
		as_root groupdel "$1" 2>/dev/null || true
	else
		as_root deluser "$1" || true
		as_root delgroup "$1" 2>/dev/null || true
	fi
}

uninstall() {
	purge="$1"
	if has_systemd && [ -f "$UPDATE_TIMER" ]; then
		as_root systemctl disable --now botloft-update.timer >/dev/null 2>&1 || true
		as_root rm -f "$UPDATE_TIMER" "$UPDATE_UNIT"
	fi
	if has_systemd && [ -f "$UNIT" ]; then
		as_root systemctl disable --now botloft.service >/dev/null 2>&1 || true
		as_root rm -f "$UNIT"
		as_root rm -rf "$LIMITS_DIR"
		as_root systemctl daemon-reload
	fi
	if [ -x "$BIN" ]; then
		as_root "$BIN" tunnel remove >/dev/null 2>&1 || true
	fi
	as_root rm -f "$BIN"
	if [ "$purge" = "yes" ]; then
		remove_user "$ACCOUNT"
		remove_user "$AGENTS"
		as_root rm -rf "$DATA" "$AGENTS_HOME"
		say "Botloft, its data and the $ACCOUNT and $AGENTS users were removed."
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
# BusyBox's ps cannot show a process's whole command, which the daemon reads
# to end what a crash left running (spec 14.1).
if command -v apk >/dev/null 2>&1 && ! ps -ww -o command= -p 1 >/dev/null 2>&1; then
	as_root apk add --no-cache -q procps
fi

# 2. The two users. The agents need a shell for their commands; the daemon
# none. The daemon joins the agents' group to reach their folders.
nologin="/usr/sbin/nologin"
[ -x "$nologin" ] || nologin="/sbin/nologin"
[ -x "$nologin" ] || nologin="/bin/false"
shell="/bin/bash"
[ -x "$shell" ] || shell="/bin/sh"
add_user "$ACCOUNT" "$DATA" "$nologin"
add_user "$AGENTS" "$AGENTS_HOME" "$shell"
if command -v usermod >/dev/null 2>&1; then
	as_root usermod -aG "$AGENTS" "$ACCOUNT"
else
	as_root addgroup "$ACCOUNT" "$AGENTS" 2>/dev/null || true
fi

# 3. The folders. The data folder is the daemon's alone; the agents only
# pass through it to the workspaces, which both write (setgid: what either
# makes there stays in the agents' group).
as_root mkdir -p "$HOME_DIR" "$WORK_DIR" "$AGENTS_HOME"
as_root chown "$ACCOUNT:$AGENTS" "$DATA"
as_root chmod 0750 "$DATA"
as_root chown "$ACCOUNT:$ACCOUNT" "$HOME_DIR"
as_root chmod 0700 "$HOME_DIR"
as_root chown "$ACCOUNT:$AGENTS" "$WORK_DIR"
as_root chmod 2770 "$WORK_DIR"
as_root chown "$AGENTS:$AGENTS" "$AGENTS_HOME"
as_root chmod 0750 "$AGENTS_HOME"

# 4. The settings, kept when they are already there.
if ! as_root test -f "$HOME_DIR/config.toml"; then
	as_root sh -c "cat >'$HOME_DIR/config.toml'" <<EOF
# Botloft on a server (spec 42). After a change: systemctl restart botloft
server = true
agents_user = "$AGENTS"
port = $PORT
workspaces_root = "$WORK_DIR"
EOF
	as_root chown "$ACCOUNT:$ACCOUNT" "$HOME_DIR/config.toml"
	as_root chmod 0600 "$HOME_DIR/config.toml"
fi

# 5. Claude Code, which the agents run, from its official installer.
if [ -z "${BOTLOFT_NO_CLAUDE:-}" ]; then
	# $HOME is the agents' user's, so it expands in that user's shell.
	# shellcheck disable=SC2016
	if as_agents 'command -v claude >/dev/null 2>&1 || [ -x "$HOME/.local/bin/claude" ]'; then
		say "Claude Code is already there."
	else
		command -v curl >/dev/null 2>&1 || fail "install curl first: Claude Code's installer needs it"
		say "Installing Claude Code for the agents..."
		as_agents 'curl -fsSL https://claude.ai/install.sh | bash'
	fi
fi

# 6. The system service: it starts at boot and comes back after a crash. It
# may only switch to the agents' user and stop their processes, and keeps
# the agents within half the memory, so the rest of the server lives; the
# owner's own limits (`botloftd limits`) are kept apart and win.
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
UMask=0007
AmbientCapabilities=CAP_SETUID CAP_SETGID CAP_KILL
CapabilityBoundingSet=CAP_SETUID CAP_SETGID CAP_KILL
NoNewPrivileges=yes
Restart=on-failure
RestartSec=5
KillMode=control-group
TimeoutStopSec=20
MemoryMax=50%

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

	# The limits asked for, once Botloft answers (the agents' cap is its own
	# setting).
	if [ -n "${BOTLOFT_MEMORY:-}${BOTLOFT_CPUS:-}${BOTLOFT_AGENTS:-}" ]; then
		set --
		[ -n "${BOTLOFT_MEMORY:-}" ] && set -- "$@" --memory "$BOTLOFT_MEMORY"
		[ -n "${BOTLOFT_CPUS:-}" ] && set -- "$@" --cpus "$BOTLOFT_CPUS"
		[ -n "${BOTLOFT_AGENTS:-}" ] && set -- "$@" --agents "$BOTLOFT_AGENTS"
		tries=0
		until as_root env BOTLOFT_HOME="$HOME_DIR" "$BIN" limits "$@"; do
			tries=$((tries + 1))
			[ "$tries" -lt 10 ] || fail "the limits were not set; try: sudo botloftd limits $*"
			sleep 1
		done
	else
		say "Botloft and its agents may use up to half the memory (change it: sudo botloftd limits --memory 4G --cpus 2 --agents 6)."
	fi

	# 7. Updates: every hour, the newest release's daemon, checked against the
	# updater's signature, once no agent is working (or after a day).
	if [ -z "${BOTLOFT_NO_AUTO_UPDATE:-}" ]; then
		as_root sh -c "cat >'$UPDATE_UNIT'" <<EOF
[Unit]
Description=Botloft update
After=network-online.target
Wants=network-online.target

[Service]
Type=oneshot
Environment=BOTLOFT_HOME=$HOME_DIR
ExecStart=$BIN update
EOF
		as_root sh -c "cat >'$UPDATE_TIMER'" <<EOF
[Unit]
Description=Looks for a Botloft update every hour

[Timer]
OnBootSec=15min
OnUnitActiveSec=1h
RandomizedDelaySec=10min
Persistent=true

[Install]
WantedBy=timers.target
EOF
		as_root systemctl daemon-reload
		as_root systemctl enable --now botloft-update.timer >/dev/null
		say "Botloft updates itself when a new version is out (turn it off: sudo systemctl disable --now botloft-update.timer)."
	fi
else
	say "No system service was set up. Botloft needs CAP_SETUID, CAP_SETGID and CAP_KILL"
	say "to run the agents as $AGENTS; as root, start it with:"
	say "  BOTLOFT_HOME=$HOME_DIR $BIN serve"
fi

# 8. The app's own tunnel (spec 42.6): a user that can only reach Botloft,
# whom sshd lets in with the key of a paired device. No new port opens.
if [ -z "${BOTLOFT_NO_TUNNEL:-}" ]; then
	as_root "$BIN" tunnel setup || say "So the app reaches Botloft through an SSH tunnel of yours (sudo botloftd pair says how)."
fi

say ""
say "Next: sign in to Claude for the agents."
say "  sudo -u $AGENTS -i claude auth login"
say "It shows a link. Open it on any device, sign in, and paste the code it gives you."
say ""
say "Then connect the Botloft app on your computer. Here, run:"
say "  sudo botloftd pair"
say "and paste the key it shows in the app, under \"Connect to a server\"."
