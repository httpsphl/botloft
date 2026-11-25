#!/bin/sh
# Runs install.sh in a fresh container and checks what it leaves behind
# (spec 42.3, 42.5). From the repository:
#
#   docker run --rm -v "$PWD:/src:ro" debian:12 sh /src/deploy/server/test-install.sh
#
# A stand-in daemon takes the release's place, and Claude Code and the
# service are left out: a container has no systemd and the test no network.
set -eu

fail() {
	echo "FAIL: $*" >&2
	exit 1
}

INSTALL="/src/deploy/server/install.sh"
# A server has su; a minimal container image may not.
if ! command -v su >/dev/null 2>&1 && command -v dnf >/dev/null 2>&1; then
	dnf install -y -q util-linux >/dev/null
fi
stub="/tmp/botloftd"
printf '#!/bin/sh\necho "botloftd 0.0.0-test"\n' >"$stub"
chmod +x "$stub"
# A plain container may not make user namespaces: the agents' box is
# test-sandbox.sh's, under systemd.
run() { BOTLOFT_FROM="$stub" BOTLOFT_NO_SERVICE=1 BOTLOFT_NO_CLAUDE=1 BOTLOFT_NO_BOX=1 sh "$INSTALL"; }
# owner:group mode of a path.
looks() { stat -c '%U:%G %a' "$1"; }

run
id botloft >/dev/null 2>&1 || fail "no botloft user"
id botloft-agents >/dev/null 2>&1 || fail "no botloft-agents user"
id -nG botloft | tr ' ' '\n' | grep -qx botloft-agents || fail "the daemon is not in the agents' group"
if id -nG botloft-agents | tr ' ' '\n' | grep -qx botloft; then fail "the agents are in the daemon's group"; fi
[ "$(/usr/local/bin/botloftd --version)" = "botloftd 0.0.0-test" ] || fail "the daemon is not in place"
config="/var/lib/botloft/home/config.toml"
grep -q '^server = true$' "$config" || fail "the settings do not say server"
grep -q '^agents_user = "botloft-agents"$' "$config" || fail "the settings do not name the agents' user"
grep -q '^workspaces_root = "/var/lib/botloft/workspaces"$' "$config" || fail "no workspaces root"
grep -q '^agents_box = false$' "$config" || fail "the settings do not say the agents run without a box"
[ "$(looks /var/lib/botloft)" = "botloft:botloft-agents 750" ] || fail "data folder: $(looks /var/lib/botloft)"
[ "$(looks /var/lib/botloft/home)" = "botloft:botloft 700" ] || fail "the daemon's folder is open: $(looks /var/lib/botloft/home)"
[ "$(looks "$config")" = "botloft:botloft 600" ] || fail "the settings are open: $(looks "$config")"
[ "$(looks /var/lib/botloft/workspaces)" = "botloft:botloft-agents 2770" ] || fail "workspaces: $(looks /var/lib/botloft/workspaces)"
[ "$(looks /var/lib/botloft-agents)" = "botloft-agents:botloft-agents 750" ] || fail "agents' home: $(looks /var/lib/botloft-agents)"

# The agents' user can reach the workspaces and not the daemon's folder.
su -s /bin/sh botloft-agents -c 'touch /var/lib/botloft/workspaces/agent-was-here' || fail "the agents cannot write their workspaces"
if su -s /bin/sh botloft-agents -c 'cat /var/lib/botloft/home/config.toml' >/dev/null 2>&1; then
	fail "the agents can read the daemon's folder"
fi

# Again: the daemon is replaced, the settings stay.
echo "# the owner's line" >>"$config"
run
grep -q "^# the owner's line$" "$config" || fail "a second install replaced the settings"
[ -f /var/lib/botloft/workspaces/agent-was-here ] || fail "a second install lost the agents' work"

sh "$INSTALL" --uninstall
[ ! -e /usr/local/bin/botloftd ] || fail "--uninstall left the daemon"
[ -f "$config" ] || fail "--uninstall removed the data"
sh "$INSTALL" --uninstall --purge
if id botloft >/dev/null 2>&1; then fail "--purge left the botloft user"; fi
if id botloft-agents >/dev/null 2>&1; then fail "--purge left the botloft-agents user"; fi
[ ! -e /var/lib/botloft ] || fail "--purge left the data"
[ ! -e /var/lib/botloft-agents ] || fail "--purge left the agents' home"
# shellcheck source=/dev/null
echo "install.sh: ok on $(. /etc/os-release && echo "$PRETTY_NAME")"
