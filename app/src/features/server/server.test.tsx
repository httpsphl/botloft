import { act, cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import type { Client } from "../../lib/client";
import { FakeBotloft } from "../../lib/fake";
import { pairFakeDevice } from "../../lib/fakeDevices";
import { FakeHost } from "../../lib/fakeHost";
import { server } from "../../shell/server";
import { crewOpened, renderApp } from "../../test/app";
import { createLink } from "../onboarding/link";
import { Onboarding } from "../onboarding/Onboarding";

const vps = { name: "vps-1", port: 45710, tunnel: 45720, token: "device-token" };

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
});

async function openGeneral(fake: FakeBotloft) {
  const ops = fake.addCrew("Ops");
  fake.addBot(ops.id, "Scout");
  renderApp(fake, new FakeHost());
  await crewOpened("Ops");
  fireEvent.click(await screen.findByRole("button", { name: /Ana Lima/ }));
  fireEvent.click(screen.getByRole("menuitem", { name: "Settings" }));
  const dialog = screen.getByRole("region", { name: "Settings" });
  fireEvent.click(within(dialog).getByRole("tab", { name: "General" }));
  return dialog;
}

describe("where the agents run, in Settings", () => {
  test("a key from the server connects to it, with the tunnel to open", async () => {
    const dialog = await openGeneral(new FakeBotloft());
    expect(within(dialog).getByText("On this computer")).toBeTruthy();
    fireEvent.click(within(dialog).getByRole("button", { name: "Connect to a server" }));
    const form = await screen.findByRole("dialog", { name: "Connect to a server" });
    const field = within(form).getByLabelText("Connection key");
    const connect = within(form).getByRole("button", { name: "Connect" }) as HTMLButtonElement;
    expect(connect.disabled).toBe(true);

    fireEvent.change(field, { target: { value: "hello" } });
    expect(within(form).getByText(/That isn't a Botloft key/)).toBeTruthy();

    fireEvent.change(field, { target: { value: keyOf(vps) } });
    expect(within(form).getByText("Server: vps-1")).toBeTruthy();
    expect(within(form).getByText("ssh -N -L 45720:127.0.0.1:45710 you@your-server")).toBeTruthy();
    fireEvent.click(connect);
    expect(server.get()).toEqual(vps);
  });

  test("on a server, what needs this computer's screen or power stays out", async () => {
    const fake = new FakeBotloft();
    fake.system = { ...fake.system, server: true };
    const dialog = await openGeneral(fake);
    await within(dialog).findByText("Idle agents sleep after");
    expect(within(dialog).queryByText("Keep the computer awake while agents work")).toBeNull();
    expect(within(dialog).queryByText(/^Start with/)).toBeNull();
    expect(within(dialog).queryByText(/^Keep working after/)).toBeNull();
  });

  test("the devices with a key are listed, and removing one asks first", async () => {
    const fake = new FakeBotloft();
    fake.system = { ...fake.system, server: true };
    pairFakeDevice(fake, "Laptop", fake.now - 60_000);
    const dialog = await openGeneral(fake);
    const row = (await within(dialog).findByText("Laptop")).closest("li");
    if (!row) {
      throw new Error("no row for Laptop");
    }
    expect(within(row).getByText(/Last connected/)).toBeTruthy();
    fireEvent.click(within(row).getByRole("button", { name: "Remove" }));
    const confirm = await screen.findByRole("dialog", { name: "Remove Laptop?" });
    fireEvent.click(within(confirm).getByRole("button", { name: "Remove" }));
    expect(await within(dialog).findByText("No other device has a key.")).toBeTruthy();
  });

  test("a computer that never had a device shows no list", async () => {
    const dialog = await openGeneral(new FakeBotloft());
    await within(dialog).findByText("On this computer");
    expect(within(dialog).queryByText("Devices")).toBeNull();
  });
});

describe("the opening screen with a server", () => {
  test("says how to open the tunnel while the server does not answer", async () => {
    const fake = new FakeBotloft();
    fake.setConnection({ kind: "connecting" });
    const link = createLink(new FakeHost(), () => fake as Client);
    server.set(vps);
    await link.getState().check();
    act(() => fake.setConnection({ kind: "waiting", retryAt: 0 }));
    render(<Onboarding link={link} />);
    expect(screen.getByText("Can't reach vps-1")).toBeTruthy();
    expect(screen.getByText("ssh -N -L 45720:127.0.0.1:45710 you@your-server")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Use this computer instead" }));
    expect(server.get()).toBeNull();
  });
});
