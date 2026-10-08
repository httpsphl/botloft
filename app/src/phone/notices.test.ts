import { describe, expect, test } from "vitest";
import { toB64 } from "./crypto";
import {
  keepNotices,
  type NoticePlatform,
  type NoticeServer,
  noticeState,
  turnOff,
  turnOn,
} from "./notices";

const KEY = toB64(new Uint8Array(65).fill(4));

/** A browser that can be told how it answers, and what it was asked. */
function browser(change: Partial<NoticePlatform> = {}) {
  let permission: NotificationPermission = "default";
  let kept: { endpoint: string } | null = null;
  const asked: string[] = [];
  const platform: NoticePlatform = {
    supported: () => true,
    homeScreenNeeded: () => false,
    permission: () => permission,
    requestPermission: async () => {
      asked.push("permission");
      permission = "granted";
      return permission;
    },
    subscription: async () => kept,
    subscribe: async (key) => {
      asked.push(`subscribe:${key.length}`);
      kept = { endpoint: "https://push.example/abc" };
      return kept;
    },
    unsubscribe: async () => {
      asked.push("unsubscribe");
      kept = null;
    },
    ...change,
  };
  return {
    platform,
    asked,
    grant: () => (permission = "granted"),
    deny: () => (permission = "denied"),
  };
}

function server(key: string | null = KEY, accepts = true) {
  const told: string[] = [];
  const api: NoticeServer = {
    key: async () => key,
    subscribe: async (endpoint) => {
      told.push(endpoint);
      return accepts;
    },
    unsubscribe: async () => {
      told.push("unsubscribe");
    },
  };
  return { api, told };
}

describe("what the page can say about its notices", () => {
  test("is off until asked, and on after", async () => {
    const b = browser();
    const s = server();
    expect(await noticeState(b.platform, s.api)).toBe("off");
    expect(await turnOn(b.platform, s.api)).toBe("on");
    // The server got the address the push service gave, and the key had 65 bytes.
    expect(s.told).toEqual(["https://push.example/abc"]);
    expect(b.asked).toEqual(["permission", "subscribe:65"]);
    expect(await noticeState(b.platform, s.api)).toBe("on");
  });

  test("says why it cannot: no support, no server push, the Home Screen, or a no", async () => {
    expect(await noticeState(browser({ supported: () => false }).platform, server().api)).toBe(
      "unsupported",
    );
    expect(await noticeState(browser().platform, server(null).api)).toBe("unsupported");
    expect(
      await noticeState(browser({ homeScreenNeeded: () => true }).platform, server().api),
    ).toBe("home-screen");
    const denied = browser();
    denied.deny();
    expect(await noticeState(denied.platform, server().api)).toBe("blocked");
    // None of them asks the browser for anything.
    expect(denied.asked).toEqual([]);
  });

  test("a refusal in the browser leaves it off or blocked, and subscribes to nothing", async () => {
    const no = browser({
      requestPermission: async () => "denied",
    });
    expect(await turnOn(no.platform, server().api)).toBe("off");
    expect(no.asked).toEqual([]);
    const closed = browser({ requestPermission: async () => "default" });
    const s = server();
    expect(await turnOn(closed.platform, s.api)).toBe("off");
    expect(s.told).toEqual([]);
  });

  test("an address the server refuses is given back to the browser", async () => {
    const b = browser();
    const s = server(KEY, false);
    expect(await turnOn(b.platform, s.api)).toBe("off");
    expect(b.asked).toEqual(["permission", "subscribe:65", "unsubscribe"]);
    expect(await b.platform.subscription()).toBeNull();
  });

  test("turning off tells the server and the browser", async () => {
    const b = browser();
    const s = server();
    await turnOn(b.platform, s.api);
    expect(await turnOff(b.platform, s.api)).toBe("off");
    expect(s.told.at(-1)).toBe("unsubscribe");
    expect(b.asked.at(-1)).toBe("unsubscribe");
  });
});

describe("keeping notices working", () => {
  test("tells the server again, and subscribes again if the browser lost the address", async () => {
    const b = browser();
    const s = server();
    await keepNotices(b.platform, s.api);
    expect(s.told).toEqual([]);

    b.grant();
    await keepNotices(b.platform, s.api);
    expect(b.asked).toEqual(["subscribe:65"]);
    expect(s.told).toEqual(["https://push.example/abc"]);
    await keepNotices(b.platform, s.api);
    expect(b.asked).toEqual(["subscribe:65"]);
    expect(s.told.length).toBe(2);
  });

  test("does nothing when this server sends no notices", async () => {
    const b = browser();
    b.grant();
    const s = server(null);
    await keepNotices(b.platform, s.api);
    expect(b.asked).toEqual([]);
    expect(s.told).toEqual([]);
  });
});
