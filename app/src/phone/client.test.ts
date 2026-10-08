// @vitest-environment node
// The phone's connection and its connecting, against a computer made here
// out of the same sealing (spec 28.3, 28.4).

import { describe, expect, test, vi } from "vitest";
import type { ApprovalCard, FromPhone, QuestionCard, ToPhone } from "../lib/protocol.gen";
import { PhoneClient } from "./client";
import {
  acceptProof,
  deriveKeys,
  fromB64,
  generateKeypair,
  importKey,
  joinProof,
  open,
  seal,
  toB64,
} from "./crypto";
import type { NoticePlatform } from "./notices";
import { PairError, pairing } from "./pair";
import { memoryStore, type Session } from "./store";

const text = new TextEncoder();
const DEVICE = "dev_phone";
const PEER = "dev_computer";

class FakeSocket {
  static all: FakeSocket[] = [];
  readyState = 0;
  sent: string[] = [];
  onopen: (() => void) | null = null;
  onmessage: ((event: { data: string }) => void) | null = null;
  onclose: (() => void) | null = null;
  constructor(readonly url: string) {
    FakeSocket.all.push(this);
    queueMicrotask(() => {
      this.readyState = 1;
      this.onopen?.();
    });
  }
  send(data: string) {
    this.sent.push(data);
  }
  close() {
    this.readyState = 3;
    this.onclose?.();
  }
  serverSend(frame: unknown) {
    this.onmessage?.({ data: JSON.stringify(frame) });
  }
}

const card = (id: string, at: number): ApprovalCard => ({
  approvalId: id,
  bot: { name: "Scout", color: "#aabbcc" },
  crew: "Ops",
  createdAt: at,
  toolName: "Bash",
  summary: "git status",
  text: "git status",
  cut: false,
  atComputer: false,
});

const question = (id: string, at: number): QuestionCard => ({
  questionId: id,
  bot: { name: "Scout", color: "#aabbcc" },
  crew: "Ops",
  createdAt: at,
  text: "Which first?",
  options: ["A", "B"],
});

/** The computer's end: the same keys, the other direction. */
async function setup(notices?: NoticePlatform) {
  FakeSocket.all = [];
  const raw = (fill: number) => new Uint8Array(32).fill(fill);
  const [c2p, p2c] = [await importKey(raw(1)), await importKey(raw(2))];
  const store = memoryStore();
  const session: Session = {
    token: "tok",
    device: DEVICE,
    peer: PEER,
    name: "Phone",
    keys: { c2p, p2c },
    sent: 0,
    received: 0,
  };
  await store.save(session);
  const fetchMock = vi.fn(async (url: string, init?: RequestInit) =>
    String(url).endsWith("/v1/push/key") && init?.method === "GET"
      ? Response.json({ key: toB64(new Uint8Array(65).fill(4)) })
      : new Response(null, { status: 204 }),
  );
  const client = new PhoneClient(session, {
    origin: "http://127.0.0.1:1",
    webSocket: FakeSocket as unknown as typeof WebSocket,
    fetch: fetchMock as unknown as typeof fetch,
    store,
    retry: { min: 5, max: 20 },
    answerWait: 50,
    ...(notices ? { notices } : {}),
  });
  client.start();
  const socket = () => FakeSocket.all[FakeSocket.all.length - 1] as FakeSocket;
  await vi.waitFor(() => expect(socket().readyState).toBe(1));
  let sentByComputer = 0;
  const computer = {
    async say(message: ToPhone, to: FakeSocket = socket()) {
      sentByComputer += 1;
      await this.sayRaw(sentByComputer, message, to);
    },
    async sayRaw(seq: number, message: ToPhone, to: FakeSocket = socket()) {
      const body = await seal(
        c2p,
        "computerToPhone",
        DEVICE,
        seq,
        text.encode(JSON.stringify(message)),
      );
      to.serverSend({ t: "msg", seq, body });
    },
    /** The messages the phone sealed, opened, in order. */
    async heard(from: FakeSocket = socket()): Promise<{ seq: number; message: FromPhone }[]> {
      const out = [];
      for (const raw of from.sent) {
        const frame = JSON.parse(raw);
        if (frame.t !== "msg") continue;
        const opened = await open(p2c, "phoneToComputer", DEVICE, frame.body);
        expect(opened.seq).toBe(frame.seq);
        out.push({ seq: opened.seq, message: JSON.parse(new TextDecoder().decode(opened.plain)) });
      }
      return out;
    },
  };
  return { client, store, socket, computer, fetchMock };
}

describe("the phone's connection", () => {
  test("says hello with the token, asks for what waits and shows what comes", async () => {
    const { client, socket, computer } = await setup();
    expect(JSON.parse(socket().sent[0] as string)).toEqual({ t: "hello", token: "tok" });
    expect(client.getState().link).toBe("connecting");

    socket().serverSend({ t: "ready", kind: "phone", device: DEVICE, peer: PEER, online: true });
    await vi.waitFor(() => expect(client.getState().computer).toBe("online"));
    expect(client.getState().link).toBe("online");
    await vi.waitFor(async () => expect((await computer.heard()).length).toBe(1));
    expect((await computer.heard())[0]).toEqual({ seq: 1, message: { t: "sync" } });

    // The list arrives newest-last, whatever the order it was sent in.
    await computer.say({
      t: "snapshot",
      approvals: [card("apr_b", 20), card("apr_a", 10)],
      questions: [question("qst_1", 5)],
    });
    await vi.waitFor(() => expect(client.getState().loaded).toBe(true));
    expect(client.getState().approvals.map((c) => c.approvalId)).toEqual(["apr_a", "apr_b"]);
    expect(client.getState().questions).toHaveLength(1);

    await computer.say({ t: "approval.open", card: card("apr_c", 30) });
    await vi.waitFor(() => expect(client.getState().approvals).toHaveLength(3));
    await computer.say({ t: "approval.closed", approvalId: "apr_a", status: "allowed" });
    await vi.waitFor(() => expect(client.getState().ended.apr_a).toBe("allowed"));
    expect(client.getState().approvals.map((c) => c.approvalId)).toEqual(["apr_b", "apr_c"]);
    await computer.say({ t: "question.closed", questionId: "qst_1", status: "answered" });
    await vi.waitFor(() => expect(client.getState().questions).toHaveLength(0));
  });

  test("answers are sealed with counters that rise and are saved first", async () => {
    const { client, store, socket, computer } = await setup();
    socket().serverSend({ t: "ready", kind: "phone", device: DEVICE, peer: PEER, online: true });
    await vi.waitFor(async () => expect((await computer.heard()).length).toBe(1));
    await computer.say({ t: "approval.open", card: card("apr_a", 1) });
    await vi.waitFor(() => expect(client.getState().approvals).toHaveLength(1));

    // Two answers at once still go in order: the counter never goes back.
    const both = Promise.all([
      client.answerApproval("apr_a", true, "  fine  "),
      client.answerQuestion("qst_x", "A"),
    ]);
    expect(await both).toEqual([true, true]);
    const heard = await computer.heard();
    expect(heard.map((h) => h.seq)).toEqual([1, 2, 3]);
    expect(heard[1]?.message).toEqual({
      t: "approval.answer",
      approvalId: "apr_a",
      allow: true,
      note: "fine",
    });
    expect(heard[2]?.message).toEqual({ t: "question.answer", questionId: "qst_x", answer: "A" });
    expect((await store.load())?.sent).toBe(3);
    expect(client.getState().sending).toContain("apr_a");

    // The computer says it closed: the button is not waiting any more.
    await computer.say({ t: "approval.closed", approvalId: "apr_a", status: "allowed" });
    await vi.waitFor(() => expect(client.getState().sending).not.toContain("apr_a"));
    // And with no word back, the button returns by itself.
    await vi.waitFor(() => expect(client.getState().sending).toEqual([]));
  });

  test("a message that is repeated, old, changed or not for this phone does nothing", async () => {
    const { client, store, socket, computer } = await setup();
    socket().serverSend({ t: "ready", kind: "phone", device: DEVICE, peer: PEER, online: true });
    await computer.say({ t: "approval.open", card: card("apr_a", 1) });
    await vi.waitFor(() => expect(client.getState().approvals).toHaveLength(1));
    const first = JSON.parse(JSON.stringify(socket().sent));
    expect(first.length).toBeGreaterThan(0);

    // The same counter again, with another card: refused.
    await computer.sayRaw(1, { t: "approval.open", card: card("apr_old", 2) });
    // Changed on the way, and sealed for another device: refused.
    const body = await seal(
      await importKey(new Uint8Array(32).fill(1)),
      "computerToPhone",
      "dev_other",
      5,
      text.encode("{}"),
    );
    socket().serverSend({ t: "msg", seq: 5, body });
    const tampered = await seal(
      await importKey(new Uint8Array(32).fill(1)),
      "computerToPhone",
      DEVICE,
      6,
      text.encode(JSON.stringify({ t: "approval.open", card: card("apr_x", 3) })),
    );
    socket().serverSend({ t: "msg", seq: 6, body: tampered.replace('"seq":6', '"seq":7') });
    // The frame's counter is not the sealed one: refused.
    await computer.sayRaw(9, { t: "approval.open", card: card("apr_y", 4) });
    socket().serverSend({
      t: "msg",
      seq: 8,
      body: await seal(
        await importKey(new Uint8Array(32).fill(1)),
        "computerToPhone",
        DEVICE,
        9,
        text.encode("{}"),
      ),
    });
    await new Promise((resolve) => setTimeout(resolve, 50));
    expect(
      client
        .getState()
        .approvals.map((c) => c.approvalId)
        .sort(),
    ).toEqual(["apr_a", "apr_y"]);
    // What was taken is counted, and saved.
    expect((await store.load())?.received).toBe(9);
  });

  test("comes back after the connection drops and asks again for what waits", async () => {
    const { client, socket, computer } = await setup();
    const first = socket();
    first.serverSend({ t: "ready", kind: "phone", device: DEVICE, peer: PEER, online: true });
    await vi.waitFor(async () => expect((await computer.heard(first)).length).toBe(1));
    first.close();
    expect(client.getState()).toMatchObject({ link: "offline", computer: "unknown" });
    expect(await client.answerApproval("apr_a", true)).toBe(false);
    expect(client.getState().sending).toEqual([]);

    await vi.waitFor(() => expect(FakeSocket.all.length).toBe(2));
    const second = socket();
    await vi.waitFor(() => expect(second.readyState).toBe(1));
    second.serverSend({ t: "ready", kind: "phone", device: DEVICE, peer: PEER, online: false });
    await vi.waitFor(() => expect(client.getState().computer).toBe("offline"));
    // Presence of the computer: when it comes back, the phone asks again.
    second.serverSend({ t: "presence", device: PEER, online: true });
    await vi.waitFor(async () =>
      expect((await computer.heard(second)).length).toBeGreaterThanOrEqual(2),
    );
    expect((await computer.heard(second)).every((h) => h.message.t === "sync")).toBe(true);
    // The counter went on rising across connections.
    expect((await computer.heard(second)).map((h) => h.seq)).toEqual([2, 3]);
  });

  test("a token the server refuses ends the phone, and leaving forgets the keys", async () => {
    const { client, store, socket } = await setup();
    socket().serverSend({ t: "error", reason: "unauthorized" });
    await vi.waitFor(() => expect(client.getState().session).toBe("revoked"));
    expect(await store.load()).not.toBeNull();
    await client.forget();
    expect(await store.load()).toBeNull();
    // No new connection is tried.
    await new Promise((resolve) => setTimeout(resolve, 60));
    expect(FakeSocket.all.length).toBe(1);
  });

  test("leaving from the phone tells the server and clears everything", async () => {
    const { client, store, socket } = await setup();
    socket().serverSend({ t: "ready", kind: "phone", device: DEVICE, peer: PEER, online: true });
    await client.disconnect();
    expect(client.getState().session).toBe("left");
    expect(await store.load()).toBeNull();
  });
});

describe("connecting this phone", () => {
  /** The computer's side of 28.3, with a server that only carries what it is given. */
  async function world(
    changes: {
      daemonPub?: "swapped";
      proof2?: "wrong";
      joinStatus?: number;
      expire?: boolean;
    } = {},
  ) {
    const secret = crypto.getRandomValues(new Uint8Array(32));
    const daemon = await generateKeypair();
    const fragment = {
      id: toB64(crypto.getRandomValues(new Uint8Array(16))),
      secret,
      daemonPub: daemon.publicRaw,
    };
    let joined: { phone_pub: string; device_name: string; proof: string } | null = null;
    let polls = 0;
    const fetchFake = (async (url: string, init?: RequestInit) => {
      const path = new URL(url).pathname;
      if (path.endsWith("/join")) {
        if (changes.joinStatus) return new Response(null, { status: changes.joinStatus });
        joined = JSON.parse(String(init?.body));
        return Response.json({ poll: "poll-secret" }, { status: 202 });
      }
      polls += 1;
      expect(new Headers(init?.headers).get("Authorization")).toBe("Bearer poll-secret");
      if (changes.expire) return Response.json({ status: "expired" });
      if (polls < 2 || !joined) return Response.json({ status: "pending" });
      const phonePub = fromB64(joined.phone_pub);
      // The computer checks the proof, then accepts.
      expect(toB64(await joinProof(secret, fragment.id, phonePub))).toBe(joined.proof);
      const other = await generateKeypair();
      const proof2 = await acceptProof(secret, fragment.id, daemon.publicRaw, phonePub);
      return Response.json({
        status: "approved",
        token: "the-token",
        device: DEVICE,
        peer: PEER,
        daemon_pub: toB64(changes.daemonPub === "swapped" ? other.publicRaw : daemon.publicRaw),
        proof2: toB64(changes.proof2 === "wrong" ? new Uint8Array(32) : proof2),
      });
    }) as unknown as typeof fetch;
    const pair = pairing(fragment, {
      origin: "http://127.0.0.1:1",
      fetch: fetchFake,
      sleep: async () => {},
    });
    return {
      pair,
      daemon,
      secret,
      fragment,
      phonePub: () => (joined ? fromB64(joined.phone_pub) : new Uint8Array()),
    };
  }

  test("joins with a proof, shows the code and collects a session that talks to the computer", async () => {
    const w = await world();
    const joined = await w.pair.join("Celular da Ana");
    expect(joined.code).toMatch(/^\d{6}$/);
    const session = await joined.collect();
    expect(session).toMatchObject({
      token: "the-token",
      device: DEVICE,
      peer: PEER,
      name: "Celular da Ana",
      sent: 0,
      received: 0,
    });
    // Both ends hold the same keys: what the computer seals, the phone opens.
    const computerKeys = await deriveKeys(w.daemon.privateKey, w.phonePub(), w.secret);
    const body = await seal(computerKeys.c2p, "computerToPhone", DEVICE, 1, text.encode("hi"));
    expect(
      new TextDecoder().decode(
        (await open(session.keys.c2p, "computerToPhone", DEVICE, body)).plain,
      ),
    ).toBe("hi");
  });

  test("refuses a key that is not the one in the QR, or an accept without the secret's proof", async () => {
    for (const changes of [{ daemonPub: "swapped" }, { proof2: "wrong" }] as const) {
      const joined = await (await world(changes)).pair.join("Phone");
      await expect(joined.collect()).rejects.toMatchObject({ reason: "failed" });
    }
  });

  test("tells a code that ran out from one that never worked", async () => {
    await expect((await world({ joinStatus: 410 })).pair.join("Phone")).rejects.toBeInstanceOf(
      PairError,
    );
    await expect((await world({ joinStatus: 410 })).pair.join("Phone")).rejects.toMatchObject({
      reason: "expired",
    });
    await expect((await world({ joinStatus: 500 })).pair.join("Phone")).rejects.toMatchObject({
      reason: "failed",
    });
    const joined = await (await world({ expire: true })).pair.join("Phone");
    await expect(joined.collect()).rejects.toMatchObject({ reason: "expired" });
  });
});

describe("notices", () => {
  const granted = (): NoticePlatform => ({
    supported: () => true,
    homeScreenNeeded: () => false,
    permission: () => "granted",
    requestPermission: async () => "granted",
    subscription: async () => ({ endpoint: "https://push.example/abc" }),
    subscribe: async () => ({ endpoint: "https://push.example/abc" }),
    unsubscribe: async () => {},
  });

  test("a phone with notices on tells the server its address each time it connects", async () => {
    const { socket, fetchMock } = await setup(granted());
    socket().serverSend({ t: "ready", kind: "phone", device: DEVICE, peer: PEER, online: true });
    await vi.waitFor(() =>
      expect(
        fetchMock.mock.calls.some(
          ([url, init]) =>
            String(url).endsWith("/v1/push/subscribe") &&
            init?.method === "POST" &&
            init.body === JSON.stringify({ endpoint: "https://push.example/abc" }) &&
            new Headers(init.headers).get("Authorization") === "Bearer tok",
        ),
      ).toBe(true),
    );
  });

  test("without the browser's push there is nothing to turn on", async () => {
    const { client, socket, fetchMock } = await setup();
    socket().serverSend({ t: "ready", kind: "phone", device: DEVICE, peer: PEER, online: true });
    expect(await client.noticeState()).toBe("unsupported");
    expect(await client.turnOnNotices()).toBe("unsupported");
    expect(fetchMock.mock.calls.some(([url]) => String(url).includes("/v1/push/"))).toBe(false);
  });

  test("turning them off asks the server to forget the address", async () => {
    const { client, fetchMock } = await setup(granted());
    await client.turnOffNotices();
    expect(
      fetchMock.mock.calls.some(
        ([url, init]) => String(url).endsWith("/v1/push/subscribe") && init?.method === "DELETE",
      ),
    ).toBe(true);
  });
});
