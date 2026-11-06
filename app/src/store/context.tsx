// React access to the daemon API, the app store and the native host.

import { createContext, type ReactNode, useContext, useEffect, useState } from "react";
import { useStore } from "zustand";
import { createStore, type StoreApi } from "zustand/vanilla";
import type { BotloftApi } from "../lib/api";
import type { Host } from "../lib/host";
import { ApiContext, ApiProvider } from "./apiContext";
import { type AppState, type AppStore, createAppStore } from "./app";
import { syncStore } from "./sync";

export { ApiProvider, useOptionalApi } from "./apiContext";

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
    <ApiProvider api={api}>
      <StoreContext.Provider value={store}>{children}</StoreContext.Provider>
    </ApiProvider>
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

const NO_STORE: StoreApi<AppState | null> = createStore<AppState | null>()(() => null);

/** From the store where there is one, else `fallback`: a mascot also shows outside the app. */
export function useOptionalApp<T>(selector: (state: AppState) => T, fallback: T): T {
  const store = useContext(StoreContext) as StoreApi<AppState | null> | null;
  return useStore(store ?? NO_STORE, (state) => (state ? selector(state) : fallback));
}

/** The store itself, for actions that set state outside the store's own. */
export function useAppStore(): AppStore {
  return required(useContext(StoreContext), "AppStore");
}
