import { describe, expect, test } from "vitest";
import type { Client } from "../../lib/client";
import { FakeBotloft } from "../../lib/fake";
import { FakeHost } from "../../lib/fakeHost";
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
    host.status = { state: "running", port: 45799, version: "0.1.0", protocol: 1 };
    await link.getState().check();
    expect(link.getState().current.step).toBe("connected");
    expect(connections).toEqual([{ port: 45799, token: "a".repeat(64) }]);
  });

  test("a stopped daemon waits for the owner to start it", async () => {
    const { host, link } = setup();
    host.status = { state: "stopped", port: 45710, home: "C:\\data\\Botloft" };
    await link.getState().check();
    expect(link.getState().current).toEqual({
      step: "stopped",
      port: 45710,
      home: "C:\\data\\Botloft",
      error: null,
    });
    await link.getState().start();
    expect(host.starts).toBe(1);
    expect(link.getState().current.step).toBe("connected");
  });

  test("a daemon that does not come up says so", async () => {
    const { host, link } = setup();
    host.status = { state: "stopped", port: 45710, home: "C:\\data" };
    host.afterStart = host.status;
    await link.getState().start();
    const current = link.getState().current;
    expect(current.step).toBe("stopped");
    expect(current.step === "stopped" && current.error).toMatch(/did not answer/);
  });

  test("another protocol version is not spoken to", async () => {
    const { host, link, connections } = setup();
    host.status = { state: "running", port: 45710, version: "9.0.0", protocol: 9 };
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
