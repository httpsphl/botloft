// The phone's session at rest (spec 28.13). Without a PIN it is kept as it
// is. With one, the token, the ids, the counters and the two keys sit in an
// AES-GCM block under a key made from the PIN (PBKDF2), so that without the
// PIN nothing on the page can use them. The key and the open session live in
// memory only, and go when the page locks.

import { fromB64, importKey, toB64 } from "./crypto";
import type { RecordStore, Session, SessionStore } from "./store";

/** How many wrong PINs the phone puts up with before it forgets the session. */
export const MAX_WRONG = 10;
const PIN = /^\d{6,10}$/;
const AAD = new TextEncoder().encode("botloft-phone-session-1");

/** What is kept, as the tokens and the keys' bytes. */
interface PlainSession {
  token: string;
  device: string;
  peer: string;
  name: string;
  sent: number;
  received: number;
  c2p: string;
  p2c: string;
}

interface PlainRecord {
  v: 2;
  plain: PlainSession;
}

interface LockedRecord {
  v: 2;
  lock: {
    salt: Uint8Array;
    iterations: number;
    iv: Uint8Array;
    ct: Uint8Array;
    /** Wrong PINs in a row: kept outside the block, so they count between visits. */
    failed: number;
    /** Not before this time, in milliseconds. */
    nextTryAt: number;
  };
}

type Stored = PlainRecord | LockedRecord | Session;

const isLocked = (record: Stored): record is LockedRecord => "v" in record && "lock" in record;
const isPlain = (record: Stored): record is PlainRecord => "v" in record && "plain" in record;

function toPlain(session: Session): PlainSession {
  if (!session.raw) {
    throw new Error("this session has no key bytes");
  }
  return {
    token: session.token,
    device: session.device,
    peer: session.peer,
    name: session.name,
    sent: session.sent,
    received: session.received,
    c2p: toB64(session.raw.c2p),
    p2c: toB64(session.raw.p2c),
  };
}

async function fromPlain(plain: PlainSession): Promise<Session> {
  const raw = { c2p: fromB64(plain.c2p), p2c: fromB64(plain.p2c) };
  return {
    token: plain.token,
    device: plain.device,
    peer: plain.peer,
    name: plain.name,
    sent: plain.sent,
    received: plain.received,
    keys: { c2p: await importKey(raw.c2p), p2c: await importKey(raw.p2c) },
    raw,
  };
}

async function keyOf(pin: string, salt: Uint8Array, iterations: number): Promise<CryptoKey> {
  const base = await crypto.subtle.importKey(
    "raw",
    new TextEncoder().encode(pin),
    "PBKDF2",
    false,
    ["deriveKey"],
  );
  return crypto.subtle.deriveKey(
    { name: "PBKDF2", hash: "SHA-256", salt: new Uint8Array(salt), iterations },
    base,
    { name: "AES-GCM", length: 256 },
    false,
    ["encrypt", "decrypt"],
  );
}

async function seal(key: CryptoKey, plain: PlainSession) {
  const iv = crypto.getRandomValues(new Uint8Array(12));
  const ct = new Uint8Array(
    await crypto.subtle.encrypt(
      { name: "AES-GCM", iv, additionalData: AAD },
      key,
      new TextEncoder().encode(JSON.stringify(plain)),
    ),
  );
  return { iv, ct };
}

async function unseal(key: CryptoKey, lock: LockedRecord["lock"]): Promise<PlainSession | null> {
  try {
    const bytes = await crypto.subtle.decrypt(
      { name: "AES-GCM", iv: new Uint8Array(lock.iv), additionalData: AAD },
      key,
      new Uint8Array(lock.ct),
    );
    return JSON.parse(new TextDecoder().decode(bytes)) as PlainSession;
  } catch {
    return null;
  }
}

/** How long to wait after `failed` wrong PINs: none for two, then 30 s, doubling. */
export function waitAfter(failed: number): number {
  return failed < 3 ? 0 : Math.min(30_000 * 2 ** (failed - 3), 3_600_000);
}

export type Unlock =
  | { ok: Session }
  /** Not the PIN: how many tries are left, and how long before the next. */
  | { wrong: true; left: number; wait: number }
  /** Too soon after a wrong one: how long to wait. */
  | { waiting: number }
  /** Too many wrong ones: the session was forgotten. */
  | { wiped: true };

export interface Vault extends SessionStore {
  state(): Promise<"none" | "open" | "locked">;
  /** Whether the stored session can take a PIN (not one from before it existed). */
  canLock(): Promise<boolean>;
  unlock(pin: string): Promise<Unlock>;
  /** Locks `session` under `pin`; later saves are encrypted too. */
  setPin(session: Session, pin: string): Promise<void>;
  /** With the right PIN, goes back to keeping the session as it is. */
  removePin(pin: string): Promise<boolean>;
  /** Forgets the key and the open session in memory. */
  lock(): void;
}

export function validPin(pin: string): boolean {
  return PIN.test(pin);
}

export function createVault(
  records: RecordStore,
  options: { iterations?: number; now?: () => number } = {},
): Vault {
  const iterations = options.iterations ?? 600_000;
  const now = options.now ?? Date.now;
  let key: CryptoKey | null = null;
  const read = async () => ((await records.get()) as Stored | null) ?? null;

  const vault: Vault = {
    state: async () => {
      const record = await read();
      return record === null ? "none" : isLocked(record) ? "locked" : "open";
    },

    canLock: async () => {
      const record = await read();
      return record !== null && (isLocked(record) || isPlain(record));
    },

    load: async () => {
      const record = await read();
      if (record === null || isLocked(record)) {
        return null;
      }
      return isPlain(record) ? fromPlain(record.plain) : record;
    },

    save: async (session) => {
      const record = await read();
      if (record !== null && isLocked(record)) {
        // Only while it is open can it be written, and it stays locked.
        if (key) {
          const { iv, ct } = await seal(key, toPlain(session));
          await records.put({ v: 2, lock: { ...record.lock, iv, ct } } satisfies LockedRecord);
        }
        return;
      }
      await records.put(
        session.raw ? ({ v: 2, plain: toPlain(session) } satisfies PlainRecord) : session,
      );
    },

    clear: async () => {
      key = null;
      await records.delete();
    },

    unlock: async (pin) => {
      const record = await read();
      if (record === null || !isLocked(record)) {
        return { wiped: true };
      }
      const { lock } = record;
      const time = now();
      if (time < lock.nextTryAt) {
        return { waiting: lock.nextTryAt - time };
      }
      const candidate = await keyOf(pin, lock.salt, lock.iterations);
      const plain = await unseal(candidate, lock);
      if (plain) {
        key = candidate;
        await records.put({
          v: 2,
          lock: { ...lock, failed: 0, nextTryAt: 0 },
        } satisfies LockedRecord);
        return { ok: await fromPlain(plain) };
      }
      const failed = lock.failed + 1;
      if (failed >= MAX_WRONG) {
        key = null;
        await records.delete();
        return { wiped: true };
      }
      const wait = waitAfter(failed);
      await records.put({
        v: 2,
        lock: { ...lock, failed, nextTryAt: wait === 0 ? 0 : time + wait },
      } satisfies LockedRecord);
      return { wrong: true, left: MAX_WRONG - failed, wait };
    },

    setPin: async (session, pin) => {
      if (!validPin(pin)) {
        throw new Error("a PIN has 6 to 10 digits");
      }
      const record = await read();
      if (record !== null && isLocked(record)) {
        throw new Error("a PIN is already on");
      }
      const salt = crypto.getRandomValues(new Uint8Array(16));
      const made = await keyOf(pin, salt, iterations);
      const { iv, ct } = await seal(made, toPlain(session));
      await records.put({
        v: 2,
        lock: { salt, iterations, iv, ct, failed: 0, nextTryAt: 0 },
      } satisfies LockedRecord);
      key = made;
    },

    removePin: async (pin) => {
      const result = await vault.unlock(pin);
      if (!("ok" in result)) {
        return false;
      }
      await records.put({ v: 2, plain: toPlain(result.ok) } satisfies PlainRecord);
      key = null;
      return true;
    },

    lock: () => {
      key = null;
    },
  };
  return vault;
}
