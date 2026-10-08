// What the phone keeps between visits (spec 28.7): the token, the keys and the
// two counters, in IndexedDB. The keys are CryptoKeys that cannot be
// exported. Nothing a bot asked is ever kept: the cards live in memory only.

import type { Keys } from "./crypto";

export interface Session {
  /** What the server knows this phone by. */
  token: string;
  /** This phone's device id on the server: part of every sealed message. */
  device: string;
  /** The computer it serves. */
  peer: string;
  name: string;
  keys: Keys;
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

/** The session in IndexedDB; the page works from memory if the browser refuses. */
export function browserStore(): SessionStore {
  if (typeof indexedDB === "undefined") {
    return memoryStore();
  }
  const fallback = memoryStore();
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
    load: async () => {
      try {
        return ((await work((store) => store.get(KEY), "readonly")) as Session | undefined) ?? null;
      } catch {
        return fallback.load();
      }
    },
    save: async (session) => {
      try {
        await work((store) => store.put(session, KEY), "readwrite");
      } catch {
        await fallback.save(session);
      }
    },
    clear: async () => {
      await fallback.clear();
      try {
        await work((store) => store.delete(KEY), "readwrite");
      } catch {
        // Nothing more to forget.
      }
    },
  };
}
