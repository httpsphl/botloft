// Notices on the phone (spec 28.8): the browser's push service tells this page
// something waits, with no content, and the page's worker writes the notice.
// Here is turning that on and off, and keeping it working.

import { fromB64 } from "./crypto";

/** What the page can say about its notices. */
export type NoticeState =
  /** This browser, or this server, cannot give them. */
  | "unsupported"
  /** Safari gives them only to a page on the Home Screen. */
  | "home-screen"
  | "off"
  /** The owner said no in the browser; only its settings can change that. */
  | "blocked"
  | "on";

/** The browser's side of it, so the page can be tested without one. */
export interface NoticePlatform {
  supported(): boolean;
  homeScreenNeeded(): boolean;
  permission(): NotificationPermission;
  requestPermission(): Promise<NotificationPermission>;
  /** The address the push service gave this page, if it has one. */
  subscription(): Promise<{ endpoint: string } | null>;
  subscribe(key: Uint8Array): Promise<{ endpoint: string }>;
  unsubscribe(): Promise<void>;
}

/** The server's side of it. */
export interface NoticeServer {
  /** The key to subscribe with; `null` when this server sends no notices. */
  key(): Promise<string | null>;
  /** False if the server refused the address. */
  subscribe(endpoint: string): Promise<boolean>;
  unsubscribe(): Promise<void>;
}

export async function noticeState(
  platform: NoticePlatform,
  server: NoticeServer,
): Promise<NoticeState> {
  if (!platform.supported()) {
    return "unsupported";
  }
  if (platform.homeScreenNeeded()) {
    return "home-screen";
  }
  if (platform.permission() === "denied") {
    return "blocked";
  }
  if (platform.permission() === "granted" && (await platform.subscription())) {
    return "on";
  }
  return (await server.key()) === null ? "unsupported" : "off";
}

/** Asks for permission (this must come from a tap), subscribes and tells the server. */
export async function turnOn(platform: NoticePlatform, server: NoticeServer): Promise<NoticeState> {
  const state = await noticeState(platform, server);
  if (state !== "off") {
    return state;
  }
  if ((await platform.requestPermission()) !== "granted") {
    return platform.permission() === "denied" ? "blocked" : "off";
  }
  const key = await server.key();
  if (key === null) {
    return "unsupported";
  }
  const subscription = await platform.subscribe(fromB64(key));
  if (!(await server.subscribe(subscription.endpoint))) {
    await platform.unsubscribe();
    return "off";
  }
  return "on";
}

export async function turnOff(
  platform: NoticePlatform,
  server: NoticeServer,
): Promise<NoticeState> {
  await server.unsubscribe().catch(() => {});
  await platform.unsubscribe().catch(() => {});
  return noticeState(platform, server);
}

/**
 * When the page opens with notices on, the server is told the address again:
 * the push service may have changed it, and the server may have lost it.
 */
export async function keepNotices(platform: NoticePlatform, server: NoticeServer): Promise<void> {
  if (!platform.supported() || platform.permission() !== "granted") {
    return;
  }
  let subscription = await platform.subscription();
  if (!subscription) {
    const key = await server.key();
    if (key === null) {
      return;
    }
    subscription = await platform.subscribe(fromB64(key));
  }
  await server.subscribe(subscription.endpoint);
}

/** The browser's own push, from the page's service worker. */
export function browserPlatform(): NoticePlatform {
  const registration = () => navigator.serviceWorker.ready;
  const subscription = async () => (await registration()).pushManager.getSubscription();
  return {
    supported: () =>
      typeof navigator !== "undefined" &&
      "serviceWorker" in navigator &&
      "PushManager" in window &&
      "Notification" in window,
    homeScreenNeeded: () => {
      const ios = /iPad|iPhone|iPod/.test(navigator.userAgent);
      const standalone =
        (navigator as Navigator & { standalone?: boolean }).standalone === true ||
        window.matchMedia?.("(display-mode: standalone)").matches === true;
      return ios && !standalone;
    },
    permission: () => Notification.permission,
    requestPermission: () => Notification.requestPermission(),
    subscription,
    subscribe: async (key) =>
      (await registration()).pushManager.subscribe({
        userVisibleOnly: true,
        applicationServerKey: new Uint8Array(key),
      }),
    unsubscribe: async () => {
      await (await subscription())?.unsubscribe();
    },
  };
}
