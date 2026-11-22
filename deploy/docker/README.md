# Botloft in Docker

The image runs Botloft the way the one-line installer does on a plain Linux server (spec 42.8): the daemon, Claude Code for the agents and, in the default image, Chromium for their browser. Everything Botloft keeps (its settings, the agents' memory, their folders and their Claude sign-in) lives in the `/data` volume.

```sh
docker run -d --name botloft --restart unless-stopped \
  --network host \
  --security-opt seccomp=unconfined \
  -v botloft:/data \
  ghcr.io/httpsphl/botloft
```

- `--network host` puts Botloft on the server's own `127.0.0.1`, where an SSH tunnel from your computer reaches it. Botloft never listens on another address, and a `-p` port mapping cannot reach a container's loopback.
- `--security-opt seccomp=unconfined` is for the agents' browser: Docker's default filter blocks the sandbox Chromium runs its pages in, and Botloft does not run Chromium without it. The smaller `ghcr.io/httpsphl/botloft:slim` has no browser and needs no such option.
- `--restart unless-stopped` brings Botloft back after a restart it asks for, such as restoring a backup.
- A memory limit (`--memory 2g`) is read by Botloft: agents sleep and wake to stay within it.

Then, once:

```sh
# Sign in to Claude for the agents. It shows a link: open it on any device,
# sign in and paste the code it gives you.
docker exec -it -u botloft-agents botloft claude auth login

# A key for the Botloft app on your computer.
docker exec botloft botloftd pair
```

Paste the key in the app, under "Connect to a server", and open the tunnel it shows.

To update, pull the new image and start the container again; `/data` stays.

From the repository, the image builds with `docker build -f deploy/docker/Dockerfile -t botloft .` (add `--build-arg BROWSER=no` for the slim one), and `sh deploy/docker/test-image.sh botloft` checks it.
