// The server the app connects to instead of this computer (spec 42.6): the
// key `botloftd pair` printed there, kept in this computer's storage. With
// one, the app never starts Botloft here and reaches the server through a
// tunnel that ends on this computer's 127.0.0.1.

import { useSyncExternalStore } from "react";

const STORAGE_KEY = "botloft.server";
/** What a connection key starts with: its format's version is in it. */
const KEY_PREFIX = "botloft1.";

export interface ServerProfile {
  /** The server's own name. */
  name: string;
  /** Botloft's port on the server, where the tunnel ends. */
  port: number;
  /** The port on this computer where the tunnel starts. */
  tunnel: number;
  /** This computer's own token there. */
  token: string;
}

function isPort(value: unknown): value is number {
  return typeof value === "number" && Number.isInteger(value) && value > 0 && value < 65536;
}

function fromJson(value: unknown): ServerProfile | null {
  if (!value || typeof value !== "object") {
    return null;
  }
  const { name, port, tunnel, token } = value as Record<string, unknown>;
  if (typeof name !== "string" || typeof token !== "string" || token === "") {
    return null;
  }
  if (!isPort(port) || !isPort(tunnel)) {
    return null;
  }
  return { name: name.trim() || "Botloft", port, tunnel, token };
}

/** The server in a key `botloftd pair` printed, or null when it is not one. */
export function parseKey(text: string): ServerProfile | null {
  const key = text.replace(/\s+/g, "");
  if (!key.startsWith(KEY_PREFIX)) {
    return null;
  }
  const body = key.slice(KEY_PREFIX.length).replace(/-/g, "+").replace(/_/g, "/");
  try {
    return fromJson(JSON.parse(atob(body.padEnd(Math.ceil(body.length / 4) * 4, "="))));
  } catch {
    return null;
  }
}

/**
 * A link the daemon made to itself (`http://127.0.0.1:<its port>/...`, a
 * video to play), as this computer reaches it: through the tunnel when a
 * server is chosen.
 */
export function throughTunnel(url: string): string {
  const chosen = server.get();
  if (!chosen) {
    return url;
  }
  const own = `http://127.0.0.1:${chosen.port}/`;
  return url.startsWith(own) ? `http://127.0.0.1:${chosen.tunnel}/${url.slice(own.length)}` : url;
}

/** The command that opens the tunnel from this computer; `address` stands for the server's. */
export function tunnelCommand(server: ServerProfile, address: string): string {
  return `ssh -N -L ${server.tunnel}:127.0.0.1:${server.port} ${address}`;
}

const listeners = new Set<() => void>();

function read(): ServerProfile | null {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    return stored ? fromJson(JSON.parse(stored)) : null;
  } catch {
    return null;
  }
}

let current = read();

function save(next: ServerProfile | null): void {
  current = next;
  try {
    if (next) {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(next));
    } else {
      localStorage.removeItem(STORAGE_KEY);
    }
  } catch {
    // Private storage off: the choice lasts for this window.
  }
  for (const listener of listeners) {
    listener();
  }
}

/** Where the app connects: a server, or this computer when there is none. */
export const server = {
  get: (): ServerProfile | null => current,
  set: (next: ServerProfile) => save(next),
  /** Back to Botloft on this computer. */
  clear: () => save(null),
  subscribe: (listener: () => void) => {
    listeners.add(listener);
    return () => {
      listeners.delete(listener);
    };
  },
};

export function useServer(): ServerProfile | null {
  return useSyncExternalStore(server.subscribe, server.get);
}

/** Another window of the app chose a server, or this computer again. */
export function onChosenElsewhere(listener: () => void): () => void {
  const changed = (event: StorageEvent) => {
    if (event.key === STORAGE_KEY) {
      listener();
    }
  };
  window.addEventListener("storage", changed);
  return () => window.removeEventListener("storage", changed);
}
