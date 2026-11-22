#!/bin/sh
# Runs a built daemon the way the service does (spec 42.5): as the botloft
# user with only CAP_SETUID, CAP_SETGID and CAP_KILL, the agents' user set.
# A stand-in Claude Code, installed for the agents' user, writes down who
# runs it. From the repository, after a release build:
#
#   docker run --rm -v "$PWD:/src:ro" debian:12 sh /src/deploy/server/test-agents-user.sh /src/target/release/botloftd
set -eu

fail() {
	echo "FAIL: $*" >&2
	exit 1
}

daemon="$1"
command -v setpriv >/dev/null 2>&1 || {
	apt-get update -qq >/dev/null && apt-get install -y -qq util-linux >/dev/null
}
command -v setpriv >/dev/null 2>&1 || fail "no setpriv here"

BOTLOFT_FROM="$daemon" BOTLOFT_NO_SERVICE=1 BOTLOFT_NO_CLAUDE=1 sh /src/deploy/server/install.sh >/dev/null

# The stand-in: its version, and who it ran as with which capabilities.
bin="/var/lib/botloft-agents/.local/bin"
mkdir -p "$bin"
cat >"$bin/claude" <<'EOF'
#!/bin/sh
{
	echo "uid=$(id -u) gid=$(id -g) home=$HOME"
	grep -E '^Cap(Eff|Prm|Amb):' /proc/self/status
	cat /var/lib/botloft/home/config.toml >/dev/null 2>&1 && echo "read the daemon's folder"
} >>/tmp/claude-ran.log 2>&1
echo "2.1.999 (Claude Code)"
EOF
chmod 0755 "$bin/claude"
chown -R botloft-agents:botloft-agents /var/lib/botloft-agents/.local
: >/tmp/claude-ran.log
chmod 0666 /tmp/claude-ran.log

# As the service runs it: the botloft user, its groups, three capabilities.
setpriv --reuid=botloft --regid=botloft --init-groups \
	--inh-caps=+setuid,+setgid,+kill --ambient-caps=+setuid,+setgid,+kill \
	--bounding-set=-all,+setuid,+setgid,+kill \
	env BOTLOFT_HOME=/var/lib/botloft/home "$daemon" serve >/tmp/daemon.log 2>&1 &
pid=$!
tries=0
until grep -q "uid=" /tmp/claude-ran.log; do
	tries=$((tries + 1))
	if [ "$tries" -gt 100 ] || ! kill -0 "$pid" 2>/dev/null; then
		sed 's/\x1b\[[0-9;]*m//g' /tmp/daemon.log
		fail "the daemon never ran Claude Code"
	fi
	sleep 0.2
done
kill "$pid"
wait "$pid" 2>/dev/null || true

agents_uid="$(id -u botloft-agents)"
sed 's/\x1b\[[0-9;]*m//g' /tmp/daemon.log | grep -q "the agents run as their own user" || fail "the daemon did not take the agents' user"
grep -q "^uid=$agents_uid " /tmp/claude-ran.log || fail "Claude Code did not run as botloft-agents: $(cat /tmp/claude-ran.log)"
grep -q "home=/var/lib/botloft-agents" /tmp/claude-ran.log || fail "Claude Code got another home: $(cat /tmp/claude-ran.log)"
for cap in CapEff CapPrm CapAmb; do
	grep -q "^$cap:[[:space:]]*0000000000000000$" /tmp/claude-ran.log || fail "the agent kept capabilities: $(cat /tmp/claude-ran.log)"
done
if grep -q "read the daemon's folder" /tmp/claude-ran.log; then fail "the agent read the daemon's folder"; fi
echo "agents user: ok"
