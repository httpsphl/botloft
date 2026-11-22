# Botloft on a server, in a container (spec 42.8): the daemon, Claude Code
# for the agents and Chromium for their browser. Inside, the daemon and the
# agents are two users, as install.sh sets them up on a plain server (42.3,
# 42.5), and everything Botloft keeps is under /data.
#
# From the repository, building the daemon here:
#
#   docker build -f deploy/docker/Dockerfile -t botloft .
#   docker build -f deploy/docker/Dockerfile --build-arg BROWSER=no -t botloft:slim .
#
# The release takes the static daemon it already built (`DAEMON=release`,
# with `botloftd` at the top of the build context). How to run it:
# deploy/docker/README.md.

ARG DAEMON=source

FROM rust:1-alpine AS daemon-source
RUN apk add --no-cache musl-dev gcc g++ make cmake perl linux-headers git
WORKDIR /src
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    cargo build --release --locked -p botloftd \
    && strip target/release/botloftd \
    && cp target/release/botloftd /botloftd

FROM scratch AS daemon-release
COPY botloftd /botloftd

FROM daemon-${DAEMON} AS daemon

FROM debian:bookworm-slim
# "no" leaves the browser out: a smaller image, and agents without one.
ARG BROWSER=yes
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        bash ca-certificates curl git procps tini util-linux \
    && if [ "$BROWSER" = "yes" ]; then \
        apt-get install -y --no-install-recommends chromium fonts-liberation fonts-noto-color-emoji; \
    fi \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --system botloft-agents \
    && useradd --system --gid botloft-agents --home-dir /data/agents --shell /bin/bash botloft-agents \
    && useradd --system --user-group --groups botloft-agents --home-dir /data/home \
        --shell /usr/sbin/nologin botloft
# Claude Code from its official installer. The binary leaves the installer's
# home, so the data volume can sit where the agents' home is.
RUN HOME=/tmp/claude-install bash -c 'curl -fsSL https://claude.ai/install.sh | bash' \
    && cp -L /tmp/claude-install/.local/bin/claude /usr/local/bin/claude \
    && rm -rf /tmp/claude-install \
    && claude --version
COPY --from=daemon --chmod=0755 /botloftd /usr/local/bin/botloftd
COPY --chmod=0755 deploy/docker/entrypoint.sh /usr/local/bin/botloft-entrypoint
# `docker exec botloft botloftd pair` finds the data; an update comes as a
# new image, not from inside (42.11).
ENV BOTLOFT_HOME=/data/home \
    BOTLOFT_IN_CONTAINER=1
VOLUME /data
ENTRYPOINT ["tini", "--", "botloft-entrypoint"]
