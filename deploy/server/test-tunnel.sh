#!/bin/sh
# The app's own tunnel (spec 42.6), end to end in a fresh container: with
# sshd there, install.sh sets up botloft-tunnel, `botloftd pair` makes a
# key, and an ssh client opens the tunnel with the options the app uses.
# Botloft answers through it; nothing else is reachable, no command runs, and
# an unknown key or host key gets nowhere. From the repository, after a
# release build:
#
#   docker run --rm -v "$PWD:/src:ro" debian:12 sh /src/deploy/server/test-tunnel.sh /src/target/release/botloftd
set -eu

fail() {
	echo "FAIL: $*" >&2
	if [ -f /tmp/daemon.log ]; then sed 's/\x1b\[[0-9;]*m//g' /tmp/daemon.log | tail -20 >&2; fi
	exit 1
}

daemon="$1"
BIN="/usr/local/bin/botloftd"

# sshd, an ssh client, curl and jq, from whatever the image has.
if command -v apt-get >/dev/null 2>&1; then
	apt-get update -qq >/dev/null
	DEBIAN_FRONTEND=noninteractive apt-get install -y -qq openssh-server openssh-client curl jq >/dev/null
elif command -v apk >/dev/null 2>&1; then
	apk add --no-cache -q openssh curl jq
elif command -v dnf >/dev/null 2>&1; then
	dnf install -y -q openssh-server openssh-clients curl jq >/dev/null
fi
ssh-keygen -A >/dev/null
mkdir -p /run/sshd
/usr/sbin/sshd || fail "sshd did not start"

out="$(BOTLOFT_FROM="$daemon" BOTLOFT_NO_SERVICE=1 BOTLOFT_NO_CLAUDE=1 sh /src/deploy/server/install.sh 2>&1)" ||
	fail "install.sh: $out"
echo "$out" | grep -q "can now open its own SSH connection" || fail "no tunnel set up: $out"
[ -f /etc/ssh/sshd_config.d/botloft.conf ] || fail "no drop-in"
[ "$(grep '^botloft-tunnel:' /etc/shadow | cut -d: -f2)" = "*" ] || fail "the tunnel user's password field is not *"
for_user() { /usr/sbin/sshd -T -C "user=$1,host=localhost,addr=127.0.0.1"; }
for_user botloft-tunnel | grep -qx "permitopen 127.0.0.1:45710" || fail "the tunnel user may open more"
for_user botloft-tunnel | grep -qx "allowtcpforwarding local" || fail "the tunnel user may forward more"
if for_user root | grep -q "botloftd tunnel keys"; then fail "the settings reach other users"; fi

# The daemon as its own user; the agents are not part of this test.
sed -i '/^agents_user/d' /var/lib/botloft/home/config.toml
su -s /bin/sh botloft -c "BOTLOFT_HOME=/var/lib/botloft/home $BIN serve" >/tmp/daemon.log 2>&1 &
tries=0
until curl -fsS http://127.0.0.1:45710/health >/dev/null 2>&1; do
	tries=$((tries + 1))
	[ "$tries" -gt 50 ] && fail "the daemon did not start"
	sleep 0.2
done

paired="$("$BIN" pair --name Laptop --host 127.0.0.1)" || fail "botloftd pair: $paired"
key="$(echo "$paired" | sed -n 's/^  \(botloft1\..*\)$/\1/p')"
[ -n "$key" ] || fail "no key: $paired"
body="$(echo "${key#botloft1.}" | tr '_-' '/+')"
while [ $(($(printf %s "$body" | wc -c) % 4)) -ne 0 ]; do body="$body="; done
json="$(printf %s "$body" | base64 -d)"
# One field of the key; `echo` would turn the \n in it into new lines.
field() { printf '%s\n' "$json" | jq -r "$1"; }
[ "$(field .ssh.user)" = "botloft-tunnel" ] || fail "the key has no tunnel: $paired"
[ "$(field .ssh.host)" = "127.0.0.1" ] || fail "--host was not kept"
[ "$(field .ssh.port)" = "22" ] || fail "not sshd's port"
keys="$(su -s /bin/sh botloft -c "$BIN tunnel keys botloft-tunnel")"
echo "$keys" | grep -q '^restrict,port-forwarding,permitopen="127.0.0.1:45710" ssh-ed25519 ' || fail "sshd gets no key: $keys"
[ -z "$(su -s /bin/sh botloft -c "$BIN tunnel keys root")" ] || fail "another user gets the keys"

# The app's side: its key, the server's pinned host key, and ssh with the
# options the app gives it: open_ssh <key> <known hosts> <more>...
app="/tmp/app"
mkdir -m 0700 "$app"
field .ssh.key >"$app/key"
chmod 0600 "$app/key"
echo "botloft-server $(field .ssh.hostKey)" >"$app/known_hosts"
cat >"$app/open_ssh" <<'EOF'
#!/bin/sh
identity="$1"
known="$2"
shift 2
exec ssh -F none -T -o BatchMode=yes -o IdentitiesOnly=yes -o IdentityAgent=none -i "$identity" \
	-o UserKnownHostsFile="$known" -o StrictHostKeyChecking=yes -o HostKeyAlias=botloft-server \
	-o UpdateHostKeys=no -o ExitOnForwardFailure=yes -o ConnectTimeout=10 \
	-p 22 -l botloft-tunnel "$@"
EOF
chmod 0755 "$app/open_ssh"
open_ssh() { timeout 15 "$app/open_ssh" "$@"; }

"$app/open_ssh" "$app/key" "$app/known_hosts" -N -L 127.0.0.1:45720:127.0.0.1:45710 127.0.0.1 2>/tmp/tunnel.err &
tunnel=$!
tries=0
until curl -fsS http://127.0.0.1:45720/health 2>/dev/null | grep -q '"status":"ok"'; do
	tries=$((tries + 1))
	if [ "$tries" -gt 50 ] || ! kill -0 "$tunnel" 2>/dev/null; then fail "Botloft does not answer through the tunnel: $(cat /tmp/tunnel.err)"; fi
	sleep 0.2
done

said="$(open_ssh "$app/key" "$app/known_hosts" 127.0.0.1 echo a-shell-ran 2>&1 || true)"
if echo "$said" | grep -q a-shell-ran; then fail "a command ran"; fi
said="$(open_ssh "$app/key" "$app/known_hosts" -W 127.0.0.1:22 127.0.0.1 </dev/null 2>&1 || true)"
if echo "$said" | grep -q "SSH-2.0"; then fail "the tunnel reached sshd's port"; fi
echo "$said" | grep -qi "prohibited" || fail "another port was not refused: $said"
said="$(open_ssh "$app/key" "$app/known_hosts" -W localhost:45710 127.0.0.1 </dev/null 2>&1 || true)"
echo "$said" | grep -qi "prohibited" || fail "another name for Botloft was not refused: $said"
status=0
open_ssh "$app/key" "$app/known_hosts" -N -R 127.0.0.1:45730:127.0.0.1:45710 127.0.0.1 2>/dev/null || status=$?
# 124: still connected when the time ran out, so the forward was taken.
[ "$status" -ne 0 ] && [ "$status" -ne 124 ] || fail "a forward from the server was allowed"

# Someone else's key, and a server with another host key, get nowhere.
ssh-keygen -q -t ed25519 -N "" -f "$app/other"
said="$(open_ssh "$app/other" "$app/known_hosts" -N -L 127.0.0.1:45721:127.0.0.1:45710 127.0.0.1 2>&1 || true)"
echo "$said" | grep -q "Permission denied" || fail "an unknown key was not refused: $said"
echo "botloft-server $(cut -d' ' -f1-2 "$app/other.pub")" >"$app/wrong_hosts"
said="$(open_ssh "$app/key" "$app/wrong_hosts" -N -L 127.0.0.1:45722:127.0.0.1:45710 127.0.0.1 2>&1 || true)"
echo "$said" | grep -q "Host key verification failed" || fail "another host key was not refused: $said"

kill "$tunnel" 2>/dev/null || true
"$BIN" tunnel remove >/dev/null || fail "tunnel remove"
[ ! -e /etc/ssh/sshd_config.d/botloft.conf ] || fail "remove left the drop-in"
if id botloft-tunnel >/dev/null 2>&1; then fail "remove left the user"; fi
# sshd starts again after reading its settings: until then it refuses all.
tries=0
until said="$(open_ssh "$app/key" "$app/known_hosts" -N 127.0.0.1 2>&1)"; ! echo "$said" | grep -q "Connection refused"; do
	tries=$((tries + 1))
	[ "$tries" -gt 25 ] && fail "sshd did not come back: $said"
	sleep 0.2
done
echo "$said" | grep -q "Permission denied" || fail "the key still opens a tunnel after remove: $said"
# shellcheck source=/dev/null
echo "tunnel: ok on $(. /etc/os-release && echo "$PRETTY_NAME")"
