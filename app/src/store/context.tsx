// React access to the daemon API, the app store and the native host.

import { createContext, type ReactNode, useContext, useEffect, useState } from "react";
import { useStore } from "zustand";
import type { BotloftApi } from "../lib/api";
import type { Host } from "../lib/host";
import { type AppState, type AppStore, createAppStore, syncStore } from "./app";

const ApiContext = createContext<BotloftApi | null>(null);
const StoreContext = createContext<AppStore | null>(null);
const HostContext = createContext<Host | null>(null);

export function HostProvider({ host, children }: { host: Host; children: ReactNode }) {
  return <HostContext.Provider value={host}>{children}</HostContext.Provider>;
}

/** Provides `api` and a store kept in sync with it. */
export function DaemonProvider({ api, children }: { api: BotloftApi; children: ReactNode }) {
  const [store] = useState(() => createAppStore(api));
  useEffect(() => syncStore(store, api), [store, api]);
  return (
    <ApiContext.Provider value={api}>
      <StoreContext.Provider value={store}>{children}</StoreContext.Provider>
    </ApiContext.Provider>
  );
}

function required<T>(value: T | null, name: string): T {
  if (value === null) {
    throw new Error(`${name} is missing; wrap the tree in its provider`);
  }
  return value;
}

export function useHost(): Host {
  return required(useContext(HostContext), "Host");
}

export function useApi(): BotloftApi {
  return required(useContext(ApiContext), "BotloftApi");
}

export function useApp<T>(selector: (state: AppState) => T): T {
  return useStore(required(useContext(StoreContext), "AppStore"), selector);
}
