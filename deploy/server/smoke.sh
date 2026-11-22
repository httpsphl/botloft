#!/bin/sh
# Starts a built daemon as a server and checks that it answers (spec 42.3).
# Run in any Linux container, from the repository after a release build:
#
#   docker run --rm -v "$PWD:/src:ro" debian:12 sh /src/deploy/server/smoke.sh /src/target/release/botloftd
set -eu

daemon="$1"
"$daemon" --version
home="$(mktemp -d)"
cat >"$home/config.toml" <<EOF
server = true
port = 45710
workspaces_root = "$home/workspaces"
EOF
BOTLOFT_HOME="$home" "$daemon" serve >"$home/out.log" 2>&1 &
pid=$!
tries=0
until [ -f "$home/secrets/owner.token" ] && grep -q "listening on 127.0.0.1" "$home/out.log" 2>/dev/null; do
	tries=$((tries + 1))
	if [ "$tries" -gt 50 ] || ! kill -0 "$pid" 2>/dev/null; then
		cat "$home/out.log"
		echo "FAIL: the daemon did not start" >&2
		exit 1
	fi
	sleep 0.2
done
# The log has colors in it.
sed 's/\x1b\[[0-9;]*m//g' "$home/out.log" | grep -q "server=true" || {
	cat "$home/out.log"
	echo "FAIL: the daemon does not say it runs as a server" >&2
	exit 1
}
kill "$pid"
wait "$pid" 2>/dev/null || true
# shellcheck source=/dev/null
echo "smoke: ok on $(. /etc/os-release && echo "$PRETTY_NAME")"
