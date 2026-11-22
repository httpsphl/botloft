#!/bin/sh
# The container's start (spec 42.8): /data as install.sh lays out a server
# (42.3), the settings on the first run, then the daemon as the botloft
# user with only the three capabilities it needs to run the agents as
# botloft-agents (42.5).
set -eu

DATA=/data
HOME_DIR="$DATA/home"
WORK_DIR="$DATA/workspaces"
AGENTS_HOME="$DATA/agents"

mkdir -p "$HOME_DIR" "$WORK_DIR" "$AGENTS_HOME"
chown botloft:botloft-agents "$DATA" "$WORK_DIR"
chmod 0750 "$DATA"
chmod 2770 "$WORK_DIR"
chown botloft:botloft "$HOME_DIR"
chmod 0700 "$HOME_DIR"
chown botloft-agents:botloft-agents "$AGENTS_HOME"
chmod 0750 "$AGENTS_HOME"

if [ ! -f "$HOME_DIR/config.toml" ]; then
	cat >"$HOME_DIR/config.toml" <<EOF
# Botloft in a container (spec 42.8). After a change: docker restart botloft
server = true
port = ${BOTLOFT_PORT:-45710}
workspaces_root = "$WORK_DIR"
agents_user = "botloft-agents"
EOF
	chown botloft:botloft "$HOME_DIR/config.toml"
	chmod 0600 "$HOME_DIR/config.toml"
fi

exec setpriv --reuid=botloft --regid=botloft --init-groups \
	--inh-caps=+setuid,+setgid,+kill --ambient-caps=+setuid,+setgid,+kill \
	--bounding-set=-all,+setuid,+setgid,+kill \
	botloftd serve
