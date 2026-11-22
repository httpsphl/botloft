#!/bin/sh
# Runs install.sh in a fresh container and checks what it leaves behind
# (spec 42.3). From the repository:
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
stub="/tmp/botloftd"
printf '#!/bin/sh\necho "botloftd 0.0.0-test"\n' >"$stub"
chmod +x "$stub"
run() { BOTLOFT_FROM="$stub" BOTLOFT_NO_SERVICE=1 BOTLOFT_NO_CLAUDE=1 sh "$INSTALL"; }

run
id botloft >/dev/null 2>&1 || fail "no botloft user"
[ "$(/usr/local/bin/botloftd --version)" = "botloftd 0.0.0-test" ] || fail "the daemon is not in place"
config="/var/lib/botloft/home/config.toml"
grep -q '^server = true$' "$config" || fail "the settings do not say server"
grep -q '^workspaces_root = "/var/lib/botloft/workspaces"$' "$config" || fail "no workspaces root"
[ "$(stat -c %U /var/lib/botloft/home)" = "botloft" ] || fail "the data is not botloft's"
[ "$(stat -c %a /var/lib/botloft/home)" = "700" ] || fail "the data folder is open to others"
[ "$(stat -c %a "$config")" = "600" ] || fail "the settings are open to others"
[ -d /var/lib/botloft/workspaces ] || fail "no workspaces"

# Again: the daemon is replaced, the settings stay.
echo "# the owner's line" >>"$config"
run
grep -q "^# the owner's line$" "$config" || fail "a second install replaced the settings"

sh "$INSTALL" --uninstall
[ ! -e /usr/local/bin/botloftd ] || fail "--uninstall left the daemon"
[ -f "$config" ] || fail "--uninstall removed the data"
sh "$INSTALL" --uninstall --purge
if id botloft >/dev/null 2>&1; then fail "--purge left the user"; fi
[ ! -e /var/lib/botloft ] || fail "--purge left the data"
# shellcheck source=/dev/null
echo "install.sh: ok on $(. /etc/os-release && echo "$PRETTY_NAME")"
