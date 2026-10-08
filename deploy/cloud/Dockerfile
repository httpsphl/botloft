# The Botloft cloud server (spec 27): accounts and sealed copies.
#
#   docker build -f deploy/cloud/Dockerfile -t botloft-cloud .
#
# The build context is the repository, because the crate lives in a workspace.
# See docs/cloud-deploy.md.

# The phone's page (spec 28.7): app/src/phone, built with Vite.
FROM node:22-bookworm-slim AS phone
RUN npm install --global pnpm@12.4.1
WORKDIR /src/app
COPY app/package.json app/pnpm-lock.yaml ./
RUN pnpm install --frozen-lockfile
COPY app/ ./
COPY docs/test-vectors/ /src/docs/test-vectors/
RUN pnpm build:phone

FROM rust:1-bookworm AS build
# aws-lc, which the bucket client's TLS uses, builds with cmake.
RUN apt-get update \
    && apt-get install -y --no-install-recommends cmake \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /src
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    cargo build --release --locked -p botloft-cloud \
    && cp target/release/botloft-cloud /botloft-cloud

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --home-dir /data --shell /usr/sbin/nologin botloft \
    && mkdir /data && chown botloft /data
COPY --from=build /botloft-cloud /usr/local/bin/botloft-cloud
COPY --from=phone /src/app/dist-phone /srv/phone
USER botloft
WORKDIR /data
VOLUME /data
EXPOSE 8787
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s \
    CMD curl -fsS http://127.0.0.1:8787/health || exit 1
ENTRYPOINT ["botloft-cloud"]
CMD ["serve", "--config", "/config/cloud.toml"]
