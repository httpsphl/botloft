import { afterEach, describe, expect, test } from "vitest";
import { parseKey, type ServerProfile, server, throughTunnel } from "./server";
import { mayPass, setTunnelPort, tunnelProblem, tunnelSpec } from "./tunnel";

/** A key as `botloftd pair` writes it: base64url without padding. */
function keyOf(body: unknown): string {
  const base64 = btoa(JSON.stringify(body))
    .replace(/\+/g, "-")
    .replace(/\//g, "_")
    .replace(/=+$/, "");
  return `botloft1.${base64}`;
}

const ssh = {
  user: "botloft-tunnel",
  host: "203.0.113.10",
  port: 22,
  hostKey: "ssh-ed25519 AAAAhost",
  key: "-----BEGIN OPENSSH PRIVATE KEY-----\nAAAA\n-----END OPENSSH PRIVATE KEY-----\n",
};
const manual: ServerProfile = { name: "vps-1", port: 45710, tunnel: 45720, token: "abc" };
const vps: ServerProfile = { ...manual, ssh };

afterEach(() => {
  server.clear();
  setTunnelPort(null);
});

describe("a key that says how to open the tunnel", () => {
  test("gives the server with its SSH part", () => {
    expect(parseKey(keyOf(vps))).toEqual(vps);
    expect(tunnelSpec(vps)).toEqual({ ...ssh, remotePort: 45710, localPort: 45720 });
    expect(tunnelSpec(manual)).toBeNull();
  });

  test("says it whole, or is no key", () => {
    expect(parseKey(keyOf({ ...vps, ssh: { ...ssh, key: "" } }))).toBeNull();
    expect(parseKey(keyOf({ ...vps, ssh: { ...ssh, port: 0 } }))).toBeNull();
    expect(parseKey(keyOf({ ...vps, ssh: { ...ssh, host: "-oProxyCommand=calc" } }))).toBeNull();
    expect(parseKey(keyOf({ ...vps, ssh: { ...ssh, host: "a b" } }))).toBeNull();
    expect(parseKey(keyOf({ ...vps, ssh: "yes" }))).toBeNull();
  });

  test("links go through the port the tunnel opened on", () => {
    const link = "http://127.0.0.1:45710/media/abc";
    server.set(vps);
    expect(throughTunnel(link)).toBe("http://127.0.0.1:45720/media/abc");
    setTunnelPort(50123);
    expect(throughTunnel(link)).toBe("http://127.0.0.1:50123/media/abc");
  });
});

describe("what went wrong opening it", () => {
  test("is read from the app's own words", () => {
    expect(tunnelProblem("refused: botloft-tunnel@x: Permission denied (publickey).")).toEqual({
      reason: "refused",
      detail: "botloft-tunnel@x: Permission denied (publickey).",
    });
    expect(tunnelProblem("key_file: Bad permissions")).toEqual({
      reason: "failed",
      detail: "key_file: Bad permissions",
    });
    expect(tunnelProblem(new Error("boom"))).toEqual({ reason: "failed", detail: "boom" });
  });

  test("only a server that does not answer is tried again", () => {
    expect(mayPass({ reason: "unreachable", detail: "" })).toBe(true);
    for (const reason of ["refused", "host_key", "no_ssh", "failed"] as const) {
      expect(mayPass({ reason, detail: "" })).toBe(false);
    }
  });
});
