import { afterEach, describe, expect, test } from "vitest";
import { onChosenElsewhere, parseKey, server, throughTunnel, tunnelCommand } from "./server";

/** A key as `botloftd pair` writes it: base64url without padding. */
function keyOf(body: unknown): string {
  const base64 = btoa(JSON.stringify(body))
    .replace(/\+/g, "-")
    .replace(/\//g, "_")
    .replace(/=+$/, "");
  return `botloft1.${base64}`;
}

const body = { name: "vps-1", port: 45710, tunnel: 45720, token: "abc" };

afterEach(() => server.clear());

describe("connection keys", () => {
  test("a key from botloftd pair gives the server", () => {
    expect(parseKey(keyOf(body))).toEqual(body);
  });

  test("spaces and line breaks from the terminal do not matter", () => {
    const key = keyOf(body);
    expect(parseKey(`  ${key.slice(0, 20)}\n${key.slice(20)}  `)).toEqual(body);
  });

  test("anything else is no key", () => {
    expect(parseKey("")).toBeNull();
    expect(parseKey("hello")).toBeNull();
    expect(parseKey("botloft1.not-base64!")).toBeNull();
    expect(parseKey(keyOf({ ...body, token: "" }))).toBeNull();
    expect(parseKey(keyOf({ ...body, tunnel: 70000 }))).toBeNull();
    expect(parseKey(keyOf({ ...body, port: "45710" }))).toBeNull();
    expect(parseKey(keyOf(body).replace("botloft1.", "botloft2."))).toBeNull();
  });

  test("the tunnel goes from this computer's port to the server's", () => {
    expect(tunnelCommand(body, "you@your-server")).toBe(
      "ssh -N -L 45720:127.0.0.1:45710 you@your-server",
    );
  });

  test("a link the server made to itself goes through the tunnel", () => {
    const link = "http://127.0.0.1:45710/media/abc";
    expect(throughTunnel(link)).toBe(link);
    server.set(body);
    expect(throughTunnel(link)).toBe("http://127.0.0.1:45720/media/abc");
    expect(throughTunnel("http://127.0.0.1:4571/x")).toBe("http://127.0.0.1:4571/x");
  });
});

describe("the chosen server", () => {
  test("is kept and forgotten", () => {
    const heard: unknown[] = [];
    const off = server.subscribe(() => heard.push(server.get()));
    server.set(body);
    expect(server.get()).toEqual(body);
    expect(JSON.parse(localStorage.getItem("botloft.server") ?? "null")).toEqual(body);
    server.clear();
    expect(server.get()).toBeNull();
    expect(localStorage.getItem("botloft.server")).toBeNull();
    off();
    expect(heard).toEqual([body, null]);
  });

  test("another window choosing is heard", () => {
    const heard: string[] = [];
    const off = onChosenElsewhere(() => heard.push("chosen"));
    window.dispatchEvent(new StorageEvent("storage", { key: "botloft.other" }));
    window.dispatchEvent(new StorageEvent("storage", { key: "botloft.server" }));
    off();
    window.dispatchEvent(new StorageEvent("storage", { key: "botloft.server" }));
    expect(heard).toEqual(["chosen"]);
  });
});
