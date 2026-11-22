#!/bin/sh
# Starts the image the way a server would and checks it (spec 42.8): the
# daemon up as `botloft` with the agents' user set, Claude Code running for
# the agents, a device paired with `docker exec`, and the agents kept out of
# the daemon's folder.
#
#   sh deploy/docker/test-image.sh botloft:test
set -eu

image="$1"
name="botloft-test-$$"

logs() { docker logs "$name" 2>&1 | sed 's/\x1b\[[0-9;]*m//g'; }
fail() {
	echo "FAIL: $*" >&2
	logs | tail -30 >&2
	docker rm -f "$name" >/dev/null 2>&1 || true
	exit 1
}

docker run -d --name "$name" "$image" >/dev/null
tries=0
until logs | grep -q "listening on 127.0.0.1"; do
	tries=$((tries + 1))
	[ "$tries" -le 60 ] || fail "the daemon did not start"
	sleep 1
done
logs | grep -q "the agents run as their own user" || fail "the agents' user is not set"
[ "$(docker exec "$name" ps -o user= -C botloftd | head -1)" = "botloft" ] || fail "the daemon does not run as botloft"
docker exec "$name" su -s /bin/sh botloft-agents -c 'claude --version' >/dev/null || fail "Claude Code does not run for the agents"
if docker exec "$name" su -s /bin/sh botloft-agents -c 'cat /data/home/config.toml' >/dev/null 2>&1; then
	fail "the agents read the daemon's folder"
fi
paired="$(docker exec "$name" botloftd pair --name Test)" || fail "pairing failed"
echo "$paired" | grep -q '^  botloft1\.' || fail "no key: $paired"

docker rm -f "$name" >/dev/null
echo "image: ok"
