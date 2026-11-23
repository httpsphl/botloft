import { afterEach, describe, expect, test, vi } from "vitest";
import type { Client } from "../../lib/client";
import { FakeBotloft } from "../../lib/fake";
import { FakeHost } from "../../lib/fakeHost";
import { type ServerProfile, server } from "../../shell/server";
import { type Connect, createLink } from "./link";
import { TUNNEL_RETRY_MS } from "./toServer";

const ssh = {
  user: "botloft-tunnel",
  host: "203.0.113.10",
  port: 22,
  hostKey: "ssh-ed25519 AAAAhost",
  key: "-----BEGIN OPENSSH PRIVATE KEY-----\nAAAA\n-----END OPENSSH PRIVATE KEY-----\n",
};
/** A server whose tunnel the owner opens. */
const manual: ServerProfile = { name: "vps-1", port: 45710, tunnel: 45720, token: "device-token" };
const vps: ServerProfile = { ...manual, ssh };

function setup() {
  const host = new FakeHost();
  const fake = new FakeBotloft();
  const connections: { port: number; token: string }[] = [];
  const connect: Connect = (port, token) => {
    connections.push({ port, token });
    return fake as Client;
  };
  return { host, fake, link: createLink(host, connect), connections };
}

afterEach(() => {
  server.clear();
  vi.useRealTimers();
});

describe("the app's own tunnel to a server", () => {
  test("opens first, then goes through it with the device's token", async () => {
    const { host, link, connections } = setup();
    host.tunnel.port = 50123;
    server.set(vps);
    await link.getState().check();
    expect(host.tunnel.opened).toEqual([{ ...ssh, remotePort: 45710, localPort: 45720 }]);
    expect(connections).toEqual([{ port: 50123, token: "device-token" }]);
    expect(link.getState().current.step).toBe("connected");
    expect(host.installs).toEqual([]);
  });

  test("a server that does not answer is said, and tried again", async () => {
    vi.useFakeTimers();
    const { host, link, connections } = setup();
    host.tunnel.failure =
      "unreachable: ssh: connect to host 203.0.113.10 port 22: Connection timed out";
    server.set(vps);
    await link.getState().check();
    expect(link.getState().current).toEqual({
      step: "unreachable",
      server: vps,
      problem: {
        reason: "unreachable",
        detail: "ssh: connect to host 203.0.113.10 port 22: Connection timed out",
      },
    });
    expect(connections).toEqual([]);
    host.tunnel.failure = null;
    await vi.advanceTimersByTimeAsync(TUNNEL_RETRY_MS);
    expect(link.getState().current.step).toBe("connected");
  });

  test("a key the server refuses waits for the owner", async () => {
    vi.useFakeTimers();
    const { host, link } = setup();
    host.tunnel.failure = "refused: botloft-tunnel@203.0.113.10: Permission denied (publickey).";
    server.set(vps);
    await link.getState().check();
    expect(link.getState().current).toMatchObject({
      step: "unreachable",
      problem: { reason: "refused" },
    });
    await vi.advanceTimersByTimeAsync(TUNNEL_RETRY_MS * 3);
    expect(host.tunnel.opened).toHaveLength(1);
  });

  test("when the connection drops, the tunnel opens again", async () => {
    const { host, fake, link } = setup();
    server.set(vps);
    await link.getState().check();
    fake.setConnection({ kind: "waiting", retryAt: 0 });
    await Promise.resolve();
    expect(host.tunnel.opened).toHaveLength(2);
    expect(link.getState().current.step).toBe("connected");
  });

  test("back on this computer, the tunnel closes", async () => {
    const { host, link, connections } = setup();
    server.set(vps);
    await link.getState().check();
    server.clear();
    await link.getState().check();
    expect(host.tunnel.closed).toBe(1);
    expect(connections.at(-1)?.port).toBe(45710);
  });

  test("a tunnel that opens after the owner went back to this computer is closed", async () => {
    const { host, link, connections } = setup();
    host.tunnel.hold = true;
    server.set(vps);
    const opening = link.getState().check();
    server.clear();
    await link.getState().check();
    host.tunnel.release();
    await opening;
    expect(host.tunnel.closed).toBe(1);
    expect(connections.map((connection) => connection.port)).toEqual([45710]);
  });

  test("a tunnel the owner opens is left to them", async () => {
    const { host, link, connections } = setup();
    server.set(manual);
    await link.getState().check();
    expect(host.tunnel.opened).toEqual([]);
    expect(connections).toEqual([{ port: 45720, token: "device-token" }]);
  });
});
