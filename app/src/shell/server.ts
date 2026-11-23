// The server the app connects to instead of this computer (spec 42.6): the
// key `botloftd pair` printed there, kept in this computer's storage. With
// one, the app never starts Botloft here and reaches the server through a
// tunnel that ends on this computer's 127.0.0.1: its own, when the key says
// how to open it, else one the owner opens.

import { useSyncExternalStore } from "react";
import { tunnelPort } from "./tunnel";

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
  /** How the app opens the tunnel by itself, when the server was set up for it. */
  ssh?: ServerSsh;
}

/** The app's own SSH tunnel to the server (`botloftd tunnel setup` there). */
export interface ServerSsh {
  /** The user sshd lets in, only to reach Botloft. */
  user: string;
  /** The server's address, which the owner may change. */
  host: string;
  /** sshd's port. */
  port: number;
  /** The server's host key: the tunnel opens to that server only. */
  hostKey: string;
  /** This computer's private key there. */
  key: string;
}

function isPort(value: unknown): value is number {
  return typeof value === "number" && Number.isInteger(value) && value > 0 && value < 65536;
}

/** A server's name or IP address, never something `ssh` would take for an option. */
export function isHost(value: string): boolean {
  return /^[A-Za-z0-9._:-]{1,253}$/.test(value) && !value.startsWith("-");
}

function sshFromJson(value: unknown): ServerSsh | null {
  if (!value || typeof value !== "object") {
    return null;
  }
  const { user, host, port, hostKey, key } = value as Record<string, unknown>;
  const texts = [user, host, hostKey, key];
  if (!texts.every((text) => typeof text === "string" && text !== "") || !isPort(port)) {
    return null;
  }
  const address = (host as string).trim();
  if (!isHost(address)) {
    return null;
  }
  return {
    user: user as string,
    host: address,
    port,
    hostKey: hostKey as string,
    key: key as string,
  };
}

function fromJson(value: unknown): ServerProfile | null {
  if (!value || typeof value !== "object") {
    return null;
  }
  const { name, port, tunnel, token, ssh } = value as Record<string, unknown>;
  if (typeof name !== "string" || typeof token !== "string" || token === "") {
    return null;
  }
  if (!isPort(port) || !isPort(tunnel)) {
    return null;
  }
  const profile: ServerProfile = { name: name.trim() || "Botloft", port, tunnel, token };
  if (ssh === undefined) {
    return profile;
  }
  // A key that says how to open the tunnel says it whole, or is no key.
  const own = sshFromJson(ssh);
  return own ? { ...profile, ssh: own } : null;
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
  return url.startsWith(own)
    ? `http://127.0.0.1:${tunnelPort(chosen)}/${url.slice(own.length)}`
    : url;
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
