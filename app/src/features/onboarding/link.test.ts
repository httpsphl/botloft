import { afterEach, describe, expect, test } from "vitest";
import type { Client } from "../../lib/client";
import { FakeBotloft } from "../../lib/fake";
import { FakeHost, running } from "../../lib/fakeHost";
import { type ServerProfile, server } from "../../shell/server";
import { type Connect, createLink } from "./link";

function setup() {
  const host = new FakeHost();
  const fake = new FakeBotloft();
  const connections: { port: number; token: string }[] = [];
  const connect: Connect = (port, token) => {
    connections.push({ port, token });
    return fake as Client;
  };
  const link = createLink(host, connect);
  return { host, fake, link, connections };
}

describe("link to the daemon", () => {
  test("a running daemon connects with the owner token", async () => {
    const { host, link, connections } = setup();
    host.status = running({ port: 45799 });
    await link.getState().check();
    expect(link.getState().current.step).toBe("connected");
    expect(connections).toEqual([{ port: 45799, token: "a".repeat(64) }]);
  });

  test("a stopped daemon is installed without asking", async () => {
    const { host, link } = setup();
    host.status = { state: "stopped", port: 45710, home: "C:\\data\\Botloft" };
    await link.getState().check();
    expect(host.installs).toEqual(["install"]);
    expect(link.getState().current.step).toBe("connected");
  });

  test("a daemon that does not come up says so", async () => {
    const { host, link } = setup();
    host.status = { state: "stopped", port: 45710, home: "C:\\data" };
    host.afterInstall = host.status;
    await link.getState().check();
    expect(host.installs).toEqual(["install"]);
    const current = link.getState().current;
    expect(current.step).toBe("stopped");
    expect(current.step === "stopped" && current.error).toMatch(/did not answer/);
  });

  test("a failed install shows its error once, with a way to try again", async () => {
    const { host, link } = setup();
    host.status = { state: "stopped", port: 45710, home: "C:\\data" };
    host.afterInstall = new Error("cannot register the scheduled task Botloft: access denied");
    await link.getState().check();
    expect(link.getState().current).toEqual({
      step: "stopped",
      port: 45710,
      home: "C:\\data",
      error: "cannot register the scheduled task Botloft: access denied",
    });
  });

  test("an older daemon is updated before connecting", async () => {
    const { host, link, connections } = setup();
    host.status = running({ version: "0.0.9", protocol: 1, outdated: true });
    await link.getState().check();
    expect(host.installs).toEqual(["install"]);
    expect(link.getState().current.step).toBe("connected");
    expect(connections).toHaveLength(1);
  });

  test("an update that fails is shown once, without a loop", async () => {
    const { host, link, connections } = setup();
    host.status = running({ version: "0.0.9", outdated: true });
    host.afterInstall = new Error("the daemon did not stop");
    await link.getState().check();
    expect(link.getState().current).toEqual({
      step: "outdated",
      daemonVersion: "0.0.9",
      error: "the daemon did not stop",
    });
    host.afterInstall = running({ version: "0.0.9", outdated: true });
    await link.getState().install();
    expect(host.installs).toEqual(["install", "install"]);
    expect(link.getState().current).toEqual({
      step: "outdated",
      daemonVersion: "0.0.9",
      error: "It still reports version 0.0.9.",
    });
    expect(connections).toHaveLength(0);
  });

  test("restarting the daemon reconnects", async () => {
    const { host, link, connections } = setup();
    await link.getState().check();
    await link.getState().restart();
    expect(host.installs).toEqual(["restart"]);
    expect(link.getState().current.step).toBe("connected");
    expect(connections).toHaveLength(2);
  });

  test("another protocol version is not spoken to", async () => {
    const { host, link, connections } = setup();
    host.status = running({ version: "9.0.0", protocol: 9 });
    await link.getState().check();
    expect(link.getState().current).toEqual({
      step: "mismatch",
      daemonProtocol: 9,
      daemonVersion: "9.0.0",
    });
    expect(connections).toHaveLength(0);
  });

  test("a missing token and a taken port are reported", async () => {
    const { host, link } = setup();
    host.token = new Error("owner.token does not exist yet");
    await link.getState().check();
    expect(link.getState().current).toEqual({
      step: "error",
      message: "owner.token does not exist yet",
    });
    host.status = { state: "foreign", port: 45710 };
    await link.getState().check();
    expect(link.getState().current).toEqual({ step: "foreign", port: 45710 });
  });

  test("a refused hello closes the connection", async () => {
    const { fake, link } = setup();
    fake.setConnection({ kind: "connecting" });
    await link.getState().check();
    expect(link.getState().current.step).toBe("connecting");
    fake.setConnection({ kind: "failed", reason: "invalid token" });
    expect(link.getState().current).toEqual({ step: "refused", reason: "invalid token" });
    expect(fake.closed).toBe(true);
  });
});

describe("link to a server", () => {
  const vps: ServerProfile = { name: "vps-1", port: 45710, tunnel: 45720, token: "device-token" };

  afterEach(() => server.clear());

  test("goes through the tunnel with the device's token and starts nothing here", async () => {
    const { host, link, connections } = setup();
    host.status = { state: "stopped", port: 45710, home: "C:\\data" };
    server.set(vps);
    await link.getState().check();
    expect(link.getState().current.step).toBe("connected");
    expect(connections).toEqual([{ port: 45720, token: "device-token" }]);
    expect(host.installs).toEqual([]);
  });

  test("says when nothing answers where the tunnel starts, and connects once it does", async () => {
    const { fake, link } = setup();
    fake.setConnection({ kind: "connecting" });
    server.set(vps);
    await link.getState().check();
    expect(link.getState().current).toMatchObject({ step: "connecting", server: vps });
    fake.setConnection({ kind: "waiting", retryAt: 0 });
    expect(link.getState().current).toEqual({ step: "unreachable", server: vps });
    fake.setConnection({ kind: "open", daemonVersion: "0.22.0" });
    expect(link.getState().current.step).toBe("connected");
    fake.setConnection({ kind: "waiting", retryAt: 0 });
    expect(link.getState().current.step).toBe("connected");
  });

  test("a key the server no longer takes is said as the server's", async () => {
    const { fake, link } = setup();
    fake.setConnection({ kind: "connecting" });
    server.set(vps);
    await link.getState().check();
    fake.setConnection({ kind: "failed", reason: "invalid token" });
    expect(link.getState().current).toEqual({
      step: "refused",
      reason: "invalid token",
      server: vps,
    });
  });
});
