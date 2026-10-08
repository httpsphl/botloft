// @vitest-environment node
// The phone's session at rest (spec 28.13): kept plain, locked under a PIN,
// opened with it, slowed by wrong tries and forgotten after ten.

import { describe, expect, test } from "vitest";
import { importKey, open, seal, toB64 } from "./crypto";
import { memoryRecords, type Session } from "./store";
import { createVault, MAX_WRONG, validPin, waitAfter } from "./vault";

const raw = () => ({ c2p: new Uint8Array(32).fill(1), p2c: new Uint8Array(32).fill(2) });

async function session(extra: Partial<Session> = {}): Promise<Session> {
  const bytes = raw();
  return {
    token: "secret-token",
    device: "dev_phone",
    peer: "dev_computer",
    name: "Phone",
    keys: { c2p: await importKey(bytes.c2p), p2c: await importKey(bytes.p2c) },
    raw: bytes,
    sent: 3,
    received: 4,
    ...extra,
  };
}

/** A vault with fast PBKDF2 and a clock the test moves. */
function setup(initial: unknown = null) {
  const records = memoryRecords(initial);
  const clock = { at: 1_000_000 };
  const vault = createVault(records, { iterations: 1000, now: () => clock.at });
  return { records, vault, clock };
}

describe("a session with no PIN", () => {
  test("is kept as it is, counters and all, and the keys still seal and open", async () => {
    const { vault } = setup();
    expect(await vault.state()).toBe("none");
    await vault.save(await session());
    expect(await vault.state()).toBe("open");
    expect(await vault.canLock()).toBe(true);
    const back = await vault.load();
    expect(back).toMatchObject({ token: "secret-token", sent: 3, received: 4 });
    const body = await seal(
      back?.keys.p2c as CryptoKey,
      "phoneToComputer",
      "dev_phone",
      1,
      new TextEncoder().encode("hi"),
    );
    const sameKey = await importKey(raw().p2c);
    expect(
      new TextDecoder().decode((await open(sameKey, "phoneToComputer", "dev_phone", body)).plain),
    ).toBe("hi");
    await vault.clear();
    expect(await vault.state()).toBe("none");
  });

  test("a session from before the PIN works but cannot take one", async () => {
    const { vault } = setup();
    const { raw: _bytes, ...old } = await session();
    await vault.save(old);
    expect(await vault.state()).toBe("open");
    expect(await vault.canLock()).toBe(false);
    expect((await vault.load())?.token).toBe("secret-token");
    await expect(vault.setPin(old, "123456")).rejects.toThrow();
  });
});

describe("a PIN", () => {
  test("has 6 to 10 digits", () => {
    for (const good of ["123456", "0000000000"]) expect(validPin(good)).toBe(true);
    for (const bad of ["12345", "12345678901", "12345a", "", "12 456"])
      expect(validPin(bad)).toBe(false);
  });

  test("locks the session: nothing readable is left, and only the PIN opens it", async () => {
    const { vault, records } = setup();
    const live = await session();
    await vault.setPin(live, "482913");
    expect(await vault.state()).toBe("locked");
    expect(await vault.load()).toBeNull();
    // What is stored holds neither the token nor the keys, in any form.
    const stored = await records.get();
    const text = JSON.stringify(stored, (_k, v) => (v instanceof Uint8Array ? Array.from(v) : v));
    expect(text).not.toContain("secret-token");
    expect(text).not.toContain("dev_phone");
    expect(text).not.toContain(toB64(raw().c2p));

    const opened = await vault.unlock("482913");
    expect("ok" in opened && opened.ok).toMatchObject({
      token: "secret-token",
      device: "dev_phone",
      sent: 3,
      received: 4,
    });
  });

  test("keeps the counters across saves while open, and locked again after lock()", async () => {
    const { vault } = setup();
    await vault.setPin(await session(), "482913");
    await vault.save(await session({ sent: 10, received: 11 }));
    vault.lock();
    // While locked nothing can be written, and nothing is lost.
    await vault.save(await session({ sent: 99, received: 99 }));
    const again = await vault.unlock("482913");
    expect("ok" in again && again.ok).toMatchObject({ sent: 10, received: 11 });
  });

  test("a wrong PIN is refused, counted, and slowed from the third", async () => {
    const { vault, clock } = setup();
    await vault.setPin(await session(), "482913");
    vault.lock();
    for (const left of [9, 8]) {
      expect(await vault.unlock("000000")).toEqual({ wrong: true, left, wait: 0 });
    }
    expect(await vault.unlock("000000")).toEqual({ wrong: true, left: 7, wait: 30_000 });
    // Too soon: even the right PIN waits, and the wait is told.
    clock.at += 10_000;
    expect(await vault.unlock("482913")).toEqual({ waiting: 20_000 });
    clock.at += 20_000;
    // The right one opens it and the count starts again.
    expect("ok" in (await vault.unlock("482913"))).toBe(true);
    vault.lock();
    expect(await vault.unlock("000000")).toEqual({ wrong: true, left: 9, wait: 0 });
  });

  test("the count survives a restart of the page, and ten wrong ones forget the session", async () => {
    const { records, vault, clock } = setup();
    await vault.setPin(await session(), "482913");
    vault.lock();
    let last: unknown;
    for (let i = 0; i < MAX_WRONG; i++) {
      // A fresh vault each time: the count lives in the record, not in memory.
      const fresh = createVault(records, { iterations: 1000, now: () => clock.at });
      last = await fresh.unlock("000000");
      clock.at += 4 * 3_600_000;
    }
    expect(last).toEqual({ wiped: true });
    expect(await records.get()).toBeNull();
    expect(await vault.state()).toBe("none");
  });

  test("turning it off needs the PIN, and the session is kept plain again", async () => {
    const { vault } = setup();
    await vault.setPin(await session(), "482913");
    vault.lock();
    expect(await vault.removePin("111111")).toBe(false);
    expect(await vault.state()).toBe("locked");
    expect(await vault.removePin("482913")).toBe(true);
    expect(await vault.state()).toBe("open");
    expect((await vault.load())?.token).toBe("secret-token");
  });

  test("a block that was changed does not open", async () => {
    const { vault, records } = setup();
    await vault.setPin(await session(), "482913");
    vault.lock();
    const stored = (await records.get()) as { lock: { ct: Uint8Array } };
    stored.lock.ct[0] = (stored.lock.ct[0] ?? 0) ^ 1;
    expect(await vault.unlock("482913")).toMatchObject({ wrong: true });
  });
});

test("the wait doubles from 30 s and stops at an hour", () => {
  expect([0, 1, 2, 3, 4, 5, 12].map(waitAfter)).toEqual([
    0, 0, 0, 30_000, 60_000, 120_000, 3_600_000,
  ]);
});
