// @vitest-environment node
// The phone's service worker (public-phone/sw.js), run against a stand-in for
// the worker's world: what a push shows, in which words, and where a tap goes.

import { describe, expect, test, vi } from "vitest";
import code from "../../public-phone/sw.js?raw";

type Listener = (event: Record<string, unknown>) => void;

function worker(language: string, windows: { url: string; focus: () => unknown }[] = []) {
  const listeners: Record<string, Listener> = {};
  const shown: { title: string; options: Record<string, unknown> }[] = [];
  const opened: string[] = [];
  const self = {
    addEventListener: (name: string, listener: Listener) => {
      listeners[name] = listener;
    },
    navigator: { language },
    location: { origin: "https://cloud.example.org" },
    registration: {
      showNotification: async (title: string, options: Record<string, unknown>) => {
        shown.push({ title, options });
      },
    },
    clients: {
      matchAll: async () => windows,
      openWindow: async (url: string) => {
        opened.push(url);
      },
      claim: async () => {},
    },
    skipWaiting: () => {},
  };
  new Function("self", "caches", "fetch", code)(self, {}, vi.fn());
  /** Fires `name` and waits for what the worker waited on. */
  const fire = async (name: string, event: Record<string, unknown> = {}) => {
    const waited: Promise<unknown>[] = [];
    listeners[name]?.({ ...event, waitUntil: (work: Promise<unknown>) => waited.push(work) });
    await Promise.all(waited);
  };
  return { fire, shown, opened };
}

describe("a push", () => {
  test("shows one notice in the phone's language, with nothing from the push in it", async () => {
    for (const [language, title] of [
      ["en-US", "Botloft needs you"],
      ["pt-BR", "O Botloft precisa de você"],
      ["es-ES", "Botloft te necesita"],
      ["de", "Botloft needs you"],
    ] as const) {
      const w = worker(language);
      // Even a push that carried something is not read: the page has the details.
      await w.fire("push", { data: { text: () => "a secret command" } });
      expect(w.shown).toHaveLength(1);
      expect(w.shown[0]?.title).toBe(title);
      expect(JSON.stringify(w.shown[0])).not.toContain("secret");
      // One notice stands for all that wait, and a new push brings it back up.
      expect(w.shown[0]?.options).toMatchObject({ tag: "botloft-waiting", renotify: true });
    }
  });
});

describe("a tap on the notice", () => {
  test("goes to the page that is open, or opens it", async () => {
    const focus = vi.fn();
    const closed = vi.fn();
    const open = worker("en", [
      { url: "https://cloud.example.org/other", focus: vi.fn() },
      { url: "https://cloud.example.org/m/", focus },
    ]);
    await open.fire("notificationclick", { notification: { close: closed } });
    expect(closed).toHaveBeenCalled();
    expect(focus).toHaveBeenCalled();
    expect(open.opened).toEqual([]);

    const none = worker("en", []);
    await none.fire("notificationclick", { notification: { close: vi.fn() } });
    expect(none.opened).toEqual(["/m/"]);
  });
});
