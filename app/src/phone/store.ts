// What the phone keeps between visits (spec 28.7): the token, the keys and the
// two counters, in IndexedDB (vault.ts decides in what form: plain, or
// encrypted under the owner's PIN, spec 28.13). Nothing a bot asked is ever
// kept: the cards live in memory only.

import type { Keys } from "./crypto";

export interface Session {
  /** What the server knows this phone by. */
  token: string;
  /** This phone's device id on the server: part of every sealed message. */
  device: string;
  /** The computer it serves. */
  peer: string;
  name: string;
  /** What the sealing uses: keys the browser cannot export. */
  keys: Keys;
  /**
   * The same two keys as bytes, which is what can be stored encrypted. A
   * session made before the PIN existed does not have them, and cannot take
   * a PIN until the phone is connected again.
   */
  raw?: { c2p: Uint8Array; p2c: Uint8Array };
  /** The last number this phone sealed with; saved before the message goes. */
  sent: number;
  /** The highest number taken from the computer; the next must be greater. */
  received: number;
}

export interface SessionStore {
  load(): Promise<Session | null>;
  save(session: Session): Promise<void>;
  clear(): Promise<void>;
}

/** For tests, and for a browser that has no IndexedDB. */
export function memoryStore(initial: Session | null = null): SessionStore {
  let kept = initial;
  return {
    load: async () => kept,
    save: async (session) => {
      kept = session;
    },
    clear: async () => {
      kept = null;
    },
  };
}

/** Where the vault keeps its one record: whatever it is, as a clone. */
export interface RecordStore {
  get(): Promise<unknown>;
  put(record: unknown): Promise<void>;
  delete(): Promise<void>;
}

export function memoryRecords(initial: unknown = null): RecordStore {
  let kept = initial;
  return {
    get: async () => kept,
    put: async (record) => {
      kept = record;
    },
    delete: async () => {
      kept = null;
    },
  };
}

const DB = "botloft-phone";
const STORE = "session";
const KEY = "session";

function request<T>(wanted: IDBRequest<T>): Promise<T> {
  return new Promise((resolve, reject) => {
    wanted.onsuccess = () => resolve(wanted.result);
    wanted.onerror = () => reject(wanted.error);
  });
}

function openDb(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const opening = indexedDB.open(DB, 1);
    opening.onupgradeneeded = () => opening.result.createObjectStore(STORE);
    opening.onsuccess = () => resolve(opening.result);
    opening.onerror = () => reject(opening.error);
  });
}

/** The record in IndexedDB; the page works from memory if the browser refuses. */
export function browserRecords(): RecordStore {
  if (typeof indexedDB === "undefined") {
    return memoryRecords();
  }
  const fallback = memoryRecords();
  const work = async <T>(
    use: (store: IDBObjectStore) => IDBRequest<T>,
    mode: IDBTransactionMode,
  ) => {
    const db = await openDb();
    try {
      return await request(use(db.transaction(STORE, mode).objectStore(STORE)));
    } finally {
      db.close();
    }
  };
  return {
    get: async () => {
      try {
        return (await work((store) => store.get(KEY), "readonly")) ?? null;
      } catch {
        return fallback.get();
      }
    },
    put: async (record) => {
      try {
        await work((store) => store.put(record, KEY), "readwrite");
      } catch {
        await fallback.put(record);
      }
    },
    delete: async () => {
      await fallback.delete();
      try {
        await work((store) => store.delete(KEY), "readwrite");
      } catch {
        // Nothing more to forget.
      }
    },
  };
}
