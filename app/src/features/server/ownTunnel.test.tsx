import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, test, vi } from "vitest";
import type { Client } from "../../lib/client";
import { FakeBotloft } from "../../lib/fake";
import { FakeHost } from "../../lib/fakeHost";
import { type ServerProfile, server } from "../../shell/server";
import { createLink } from "../onboarding/link";
import { Onboarding } from "../onboarding/Onboarding";
import { ServerKeyForm } from "./ServerKeyForm";

const ssh = {
  user: "botloft-tunnel",
  host: "203.0.113.10",
  port: 22,
  hostKey: "ssh-ed25519 AAAAhost",
  key: "-----BEGIN OPENSSH PRIVATE KEY-----\nAAAA\n-----END OPENSSH PRIVATE KEY-----\n",
};
const vps: ServerProfile = {
  name: "vps-1",
  port: 45710,
  tunnel: 45720,
  token: "device-token",
  ssh,
};

/** The key `botloftd pair` prints for `vps`. */
function keyOf(body: unknown): string {
  const base64 = btoa(JSON.stringify(body))
    .replace(/\+/g, "-")
    .replace(/\//g, "_")
    .replace(/=+$/, "");
  return `botloft1.${base64}`;
}

afterEach(() => {
  cleanup();
  server.clear();
  vi.useRealTimers();
});

/** The opening screen after the app's own tunnel failed with `failure`. */
async function trouble(failure: string) {
  vi.useFakeTimers({ toFake: ["setTimeout", "clearTimeout"] });
  const host = new FakeHost();
  host.tunnel.failure = failure;
  const link = createLink(host, () => new FakeBotloft() as Client);
  server.set(vps);
  await link.getState().check();
  render(<Onboarding link={link} />);
}

describe("the app's own tunnel, on screen", () => {
  test("a key that says how connects by itself, at an address the owner may change", () => {
    render(<ServerKeyForm />);
    fireEvent.change(screen.getByLabelText("Connection key"), { target: { value: keyOf(vps) } });
    expect(screen.getByText(/connects to it by itself, at 203\.0\.113\.10/)).toBeTruthy();
    expect(screen.queryByText(/ssh -N -L/)).toBeNull();
    const connect = screen.getByRole("button", { name: "Connect" }) as HTMLButtonElement;
    const address = screen.getByLabelText("Server address");
    fireEvent.change(address, { target: { value: "-oProxyCommand=calc" } });
    expect(connect.disabled).toBe(true);
    fireEvent.change(address, { target: { value: " vps.example.com " } });
    fireEvent.click(connect);
    expect(server.get()).toEqual({ ...vps, ssh: { ...ssh, host: "vps.example.com" } });
  });

  test("a server that does not answer shows its address, to change", async () => {
    await trouble("unreachable: ssh: connect to host 203.0.113.10 port 22: Connection timed out");
    expect(screen.getByText("Can't reach vps-1")).toBeTruthy();
    expect(screen.getByText(/couldn't open its connection to 203\.0\.113\.10/)).toBeTruthy();
    fireEvent.change(screen.getByLabelText("Server address"), { target: { value: "100.64.0.7" } });
    fireEvent.click(screen.getByRole("button", { name: "Use this address" }));
    expect(server.get()?.ssh?.host).toBe("100.64.0.7");
  });

  test("a refused key says to pair again, with nothing to change", async () => {
    await trouble("refused: botloft-tunnel@203.0.113.10: Permission denied (publickey).");
    expect(screen.getByText("vps-1 didn't accept this computer")).toBeTruthy();
    expect(screen.getByText(/run sudo botloftd pair and paste the new key/)).toBeTruthy();
    expect(screen.queryByLabelText("Server address")).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "Use this computer instead" }));
    expect(server.get()).toBeNull();
  });

  test("no OpenSSH on this computer says where to add it", async () => {
    await trouble("no_ssh: OpenSSH's ssh is not on this computer");
    expect(screen.getByText("To reach vps-1, this computer needs OpenSSH")).toBeTruthy();
    expect(screen.getByText(/Optional features > OpenSSH Client/)).toBeTruthy();
  });
});
