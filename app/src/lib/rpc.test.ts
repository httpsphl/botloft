import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";
import { RpcErrorCode } from "./protocol.gen";
import { ClientErrorCode, RpcConnection, RpcError, type SocketLike } from "./rpc";

class FakeSocket implements SocketLike {
  onopen: ((event: unknown) => void) | null = null;
  onmessage: ((event: { data: unknown }) => void) | null = null;
  onclose: ((event: unknown) => void) | null = null;
  onerror: ((event: unknown) => void) | null = null;
  readonly sent: { id?: number; method?: string; params?: unknown }[] = [];
  closed = false;

  send(data: string): void {
    this.sent.push(JSON.parse(data));
  }

  close(): void {
    this.closed = true;
  }

  open(): void {
    this.onopen?.({});
  }

  reply(id: number | undefined, result: unknown): void {
    this.onmessage?.({ data: JSON.stringify({ jsonrpc: "2.0", id, result }) });
  }

  fail(id: number | undefined, code: number, message: string): void {
    this.onmessage?.({ data: JSON.stringify({ jsonrpc: "2.0", id, error: { code, message } }) });
  }

  notify(method: string, params: unknown): void {
    this.onmessage?.({ data: JSON.stringify({ jsonrpc: "2.0", method, params }) });
  }

  drop(): void {
    this.onclose?.({});
  }

  last(): { id?: number; method?: string; params?: unknown } {
    const frame = this.sent.at(-1);
    if (!frame) {
      throw new Error("nothing sent");
    }
    return frame;
  }
}

function setup(options: { requestTimeoutMs?: number } = {}) {
  const sockets: FakeSocket[] = [];
  const connection = new RpcConnection({
    url: "ws://127.0.0.1:45710/rpc",
    token: "t".repeat(64),
    client: { name: "test", version: "1" },
    socket: () => {
      const socket = new FakeSocket();
      sockets.push(socket);
      return socket;
    },
    ...options,
  });
  const socket = () => {
    const current = sockets.at(-1);
    if (!current) {
      throw new Error("no socket");
    }
    return current;
  };
  /** Opens the socket and accepts the hello. */
  const accept = () => {
    socket().open();
    socket().reply(socket().last().id, { daemonVersion: "0.1.0", protocol: 1 });
  };
  return { connection, sockets, socket, accept };
}

beforeEach(() => {
  vi.useFakeTimers();
});

afterEach(() => {
  vi.useRealTimers();
});

describe("RpcConnection", () => {
  test("says hello first and holds requests until the daemon accepts it", async () => {
    const { connection, socket } = setup();
    connection.start();
    const crews = connection.request("crews.list");
    expect(connection.state.kind).toBe("connecting");
    socket().open();
    expect(socket().sent).toEqual([
      {
        jsonrpc: "2.0",
        id: 1,
        method: "session.hello",
        params: { token: "t".repeat(64), client: { name: "test", version: "1" }, protocol: 1 },
      },
    ]);
    socket().reply(1, { daemonVersion: "0.1.0", protocol: 1 });
    expect(connection.state).toEqual({ kind: "open", daemonVersion: "0.1.0" });
    // Methods without params send no params field.
    expect(socket().last()).toEqual({ jsonrpc: "2.0", id: 2, method: "crews.list" });
    socket().reply(2, []);
    await expect(crews).resolves.toEqual([]);
  });

  test("answers carry their result or a typed error", async () => {
    const { connection, socket, accept } = setup();
    connection.start();
    accept();
    const created = connection.request("crews.create", { name: "" });
    expect(socket().last()).toMatchObject({ method: "crews.create", params: { name: "" } });
    socket().fail(socket().last().id, RpcErrorCode.validation, "name must not be empty");
    const error = await created.catch((failure: unknown) => failure);
    expect(error).toBeInstanceOf(RpcError);
    expect(error).toMatchObject({
      code: RpcErrorCode.validation,
      message: "name must not be empty",
    });
  });

  test("delivers notifications to every listener", () => {
    const { connection, socket, accept } = setup();
    const seen: string[] = [];
    connection.onNotification((event) => seen.push(event.name));
    connection.start();
    accept();
    socket().notify("bot.state", { botId: "bot_1", state: "idle", generation: 3 });
    expect(seen).toEqual(["bot.state"]);
  });

  test("a refused hello fails for good", async () => {
    const { connection, sockets, socket } = setup();
    connection.start();
    const pending = connection.request("crews.list");
    socket().open();
    socket().fail(1, RpcErrorCode.notAuthenticated, "invalid token");
    expect(connection.state).toEqual({ kind: "failed", reason: "invalid token" });
    await expect(pending).rejects.toMatchObject({ code: RpcErrorCode.notAuthenticated });
    vi.advanceTimersByTime(60_000);
    expect(sockets).toHaveLength(1);
    await expect(connection.request("crews.list")).rejects.toMatchObject({
      code: ClientErrorCode.closed,
    });
  });

  test("reconnects with backoff after losing the daemon", async () => {
    const { connection, sockets, socket, accept } = setup();
    const states: string[] = [];
    connection.onState((state) => states.push(state.kind));
    connection.start();
    accept();
    const inflight = connection.request("crews.list");
    socket().drop();
    await expect(inflight).rejects.toMatchObject({ code: ClientErrorCode.connectionLost });
    expect(connection.state.kind).toBe("waiting");

    const queued = connection.request("bots.list", {});
    vi.advanceTimersByTime(499);
    expect(sockets).toHaveLength(1);
    vi.advanceTimersByTime(1);
    expect(sockets).toHaveLength(2);
    // The next loss waits twice as long.
    socket().drop();
    vi.advanceTimersByTime(999);
    expect(sockets).toHaveLength(2);
    vi.advanceTimersByTime(1);
    accept();
    expect(socket().last()).toMatchObject({ method: "bots.list", params: {} });
    socket().reply(socket().last().id, []);
    await expect(queued).resolves.toEqual([]);
    expect(states).toEqual([
      "connecting",
      "open",
      "waiting",
      "connecting",
      "waiting",
      "connecting",
      "open",
    ]);
  });

  test("a request the daemon never answers times out", async () => {
    const { connection, accept } = setup({ requestTimeoutMs: 1000 });
    connection.start();
    accept();
    const slow = connection.request("system.status");
    vi.advanceTimersByTime(1000);
    await expect(slow).rejects.toMatchObject({ code: ClientErrorCode.timeout });
  });

  test("close stops everything", async () => {
    const { connection, sockets, socket } = setup();
    connection.start();
    const queued = connection.request("crews.list");
    connection.close();
    await expect(queued).rejects.toMatchObject({ code: ClientErrorCode.closed });
    expect(socket().closed).toBe(true);
    vi.advanceTimersByTime(60_000);
    expect(sockets).toHaveLength(1);
    expect(connection.state.kind).toBe("closed");
  });
});
