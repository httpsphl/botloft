#!/bin/sh
# Installs Botloft with its service in a container where systemd runs, and
# checks what the service keeps Botloft and its agents to (spec 42.4, 42.5):
# the limits, a read-only system, hidden folders and closed private
# networks. A stand-in Claude Code, which Botloft runs as the agents' user
# inside the service, tries each one and writes down what happened. From the
# repository, after a release build:
#
#   docker run -d --name botloft-sandbox --privileged --cgroupns=host \
#     -v /sys/fs/cgroup:/sys/fs/cgroup:rw -v "$PWD:/src:ro" jrei/systemd-debian:12
#   docker exec botloft-sandbox sh /src/deploy/server/test-sandbox.sh /src/target/release/botloftd
#   docker rm -f botloft-sandbox
set -eu

fail() {
	echo "FAIL: $*" >&2
	exit 1
}

daemon="$1"
command -v bash >/dev/null 2>&1 || fail "no bash here"
# Open to anyone by their modes: only the service's rules keep them away.
mkdir -p /root/secret /data/coolify /opt/open
chmod 0755 /root /root/secret
chmod 0777 /opt/open
echo "mine" >/root/secret/note
echo "theirs" >/data/coolify/db.env

BOTLOFT_FROM="$daemon" BOTLOFT_NO_CLAUDE=1 BOTLOFT_NO_TUNNEL=1 BOTLOFT_NO_AUTO_UPDATE=1 \
	BOTLOFT_MEMORY=2G BOTLOFT_CPUS=1.5 BOTLOFT_AGENTS=4 BOTLOFT_REACH=10.9.9.9 \
	sh /src/deploy/server/install.sh >/tmp/install.log 2>&1 || {
	cat /tmp/install.log
	fail "install.sh failed"
}

show() { systemctl show botloft.service -p "$1" --value; }
[ "$(show MemoryMax)" = "$((2 * 1024 * 1024 * 1024))" ] || fail "memory cap: $(show MemoryMax)"
[ "$(show CPUQuotaPerSecUSec)" = "1.500000s" ] || fail "processor cap: $(show CPUQuotaPerSecUSec)"
[ "$(show ProtectSystem)" = "strict" ] || fail "the system is not read-only: $(show ProtectSystem)"
show IPAddressDeny | grep -q "172.16.0.0/12" || fail "private networks open: $(show IPAddressDeny)"
show IPAddressAllow | grep -q "10.9.9.9" || fail "the reach asked for is missing: $(show IPAddressAllow)"
botloftd limits | grep -q "at most 4" || fail "agents' cap: $(botloftd limits)"

# The stand-in Claude Code: Botloft runs it as the agents' user, in the
# service, when it checks the version.
bin="/var/lib/botloft-agents/.local/bin"
# The service's /tmp is its own; the log goes where both see it.
mkdir -p "$bin" /var/lib/botloft-agents/out
cat >"$bin/claude" <<'EOF'
#!/bin/bash
{
	echo "ran as $(id -un)"
	cat /root/secret/note 2>/dev/null && echo "read root's home"
	cat /data/coolify/db.env 2>/dev/null && echo "read the containers' data"
	touch /opt/open/was-here 2>/dev/null && echo "wrote outside its folders"
	touch "$HOME/.ok" 2>/dev/null && echo "wrote in its home"
	# Without the block, the container's gateway refuses at once; with it,
	# the packets are dropped (or refused by the kernel) and nothing answers.
	said="$(timeout 3 bash -c 'echo > /dev/tcp/172.17.0.1/9' 2>&1; echo "exit=$?")"
	case "$said" in
	*"not permitted"* | *exit=124*) echo "private network closed" ;;
	*) echo "private network open: $said" ;;
	esac
	timeout 3 bash -c 'echo > /dev/tcp/127.0.0.1/45710' 2>/dev/null && echo "reached Botloft on loopback"
} >>/var/lib/botloft-agents/out/ran.log 2>&1
echo "2.1.999 (Claude Code)"
EOF
chmod 0755 "$bin/claude"
chown -R botloft-agents:botloft-agents /var/lib/botloft-agents/.local /var/lib/botloft-agents/out
systemctl restart botloft.service
log=/var/lib/botloft-agents/out/ran.log
tries=0
until grep -q "ran as" "$log" 2>/dev/null; do
	tries=$((tries + 1))
	[ "$tries" -lt 60 ] || {
		journalctl -u botloft --no-pager | tail -20
		fail "Botloft never ran Claude Code"
	}
	sleep 1
done
# The network tries take up to 3 s each.
sleep 8
cat "$log"
grep -q "ran as botloft-agents" "$log" || fail "not as the agents' user"
grep -q "read root's home" "$log" && fail "root's home is visible"
grep -q "read the containers' data" "$log" && fail "the containers' data is visible"
grep -q "wrote outside its folders" "$log" && fail "the system is writable"
grep -q "wrote in its home" "$log" || fail "its own home is not writable"
grep -q "private network closed" "$log" || fail "the private network is open"
grep -q "reached Botloft on loopback" "$log" || fail "loopback is closed: the agents cannot reach Botloft"
# The metadata address (169.254/16) is in the same list; the container has
# nothing there to tell a block from silence.
systemctl show botloft.service -p IPAddressDeny --value | grep -q "169.254.0.0/16" || fail "the cloud's metadata is reachable"

# The agents' browser runs as a user of its own (spec 42.5): not the
# daemon's, whose secrets it cannot read, nor the agents', who cannot read
# its cookies.
id botloft-browser >/dev/null 2>&1 || fail "no browser user"
grep -q '^browser_user = "botloft-browser"' /var/lib/botloft/home/config.toml || fail "the settings do not name the browser's user"
[ "$(stat -c '%U:%G %a' /var/lib/botloft-browser)" = "botloft-browser:botloft 2770" ] || fail "browser home: $(stat -c '%U:%G %a' /var/lib/botloft-browser)"
if ! command -v chromium >/dev/null 2>&1; then
	apt-get update -qq >/dev/null && apt-get install -y -qq chromium python3 >/dev/null
fi
systemctl restart botloft.service
sleep 3
token=/var/lib/botloft/home/secrets/owner.token
# "$1.id" is rpc.py's: the first result's id, not the shell's.
# shellcheck disable=SC2016
RPC_HOLD=40 python3 /src/deploy/server/rpc.py 45710 "$token" \
	crews.create '{"name": "Ops"}' \
	bots.create '{"crewId": "$1.id", "name": "Scout", "role": "", "instructions": ""}' \
	browser.watch '{"botId": "$2.id"}' \
	browser.start '{"botId": "$2.id"}' >/tmp/rpc.out &
browsers="--user-data-dir=/var/lib/botloft-browser/profiles/"
tries=0
until pgrep -f -- "$browsers" >/dev/null; do
	tries=$((tries + 1))
	[ "$tries" -lt 30 ] || {
		cat /tmp/rpc.out
		journalctl -u botloft --no-pager | tail -20
		fail "the browser never started"
	}
	sleep 1
done
users="$(pgrep -f -- "$browsers" | xargs -I{} ps -o user:20= -p {} | sort -u | tr -d ' ')"
[ "$users" = "botloft-browser" ] || fail "the browser runs as: $users"
bot="$(sed -n 2p /tmp/rpc.out | python3 -c 'import json, sys; print(json.load(sys.stdin)["id"])')"
profile="/var/lib/botloft-browser/profiles/$bot"
[ -d "$profile" ] || fail "no profile at $profile"
su -s /bin/sh botloft-agents -c "ls /var/lib/botloft-browser" >/dev/null 2>&1 && fail "the agents see the browser's profiles"
su -s /bin/sh botloft-browser -c "cat /var/lib/botloft/home/secrets/owner.token" >/dev/null 2>&1 && fail "the browser reads the daemon's secrets"
# Archived, the bot's browser closes and its profile goes.
python3 /src/deploy/server/rpc.py 45710 "$token" bots.archive "{\"botId\": \"$bot\"}" >/dev/null
tries=0
while [ -d "$profile" ]; do
	tries=$((tries + 1))
	[ "$tries" -lt 20 ] || fail "the archived bot's profile is still there"
	sleep 1
done
wait
echo "OK: limits, read-only system, hidden folders, closed private networks and the browser apart"
