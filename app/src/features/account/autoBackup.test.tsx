import { act, cleanup, configure, fireEvent, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, test, vi } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { fakeAutoBackupSent, setFakeAutoBackup } from "../../lib/fakeAutoBackup";
import { openFakeCloudLink } from "../../lib/fakeCloud";
import { FakeHost } from "../../lib/fakeHost";
import { crewOpened, renderApp } from "../../test/app";

afterEach(cleanup);

// Each test signs in and opens menus: a loaded machine needs more than a second.
configure({ asyncUtilTimeout: 5000 });
vi.setConfig({ testTimeout: 20_000 });

const type = (field: HTMLElement, value: string) => fireEvent.change(field, { target: { value } });

/** Settings, Backup, signed in to the account. */
async function signedIn() {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  fake.addBot(ops.id, "Scout");
  renderApp(fake, new FakeHost());
  await crewOpened("Ops");
  fireEvent.click(await screen.findByRole("button", { name: /Ana Lima/ }));
  fireEvent.click(screen.getByRole("menuitem", { name: "Settings" }));
  const dialog = screen.getByRole("region", { name: "Settings" });
  fireEvent.click(within(dialog).getByRole("tab", { name: "Account and phone" }));
  type(await within(dialog).findByLabelText("Your e-mail"), "ana@exemplo.com");
  fireEvent.click(within(dialog).getByRole("button", { name: "Send me the link" }));
  await within(dialog).findByText(/We sent a link/);
  act(() => openFakeCloudLink(fake));
  await within(dialog).findByText("Signed in as ana@exemplo.com");
  fireEvent.click(within(dialog).getByRole("tab", { name: "Backup" }));
  // The block asks the daemon for its status, so it is there a moment later.
  await within(dialog).findByText("Automatic copies");
  return { fake, dialog };
}

/** The passphrase fields of the automatic block: the second pair, after the upload's. */
function typePassphrase(dialog: HTMLElement, value: string) {
  const first = within(dialog).getAllByLabelText("Passphrase")[1] as HTMLElement;
  const again = within(dialog).getAllByLabelText("Passphrase again")[1] as HTMLElement;
  type(first, value);
  type(again, value);
}

const calls = (fake: FakeBotloft, method: string) =>
  fake.calls.filter((call) => call.method === method);

describe("automatic copies", () => {
  test("it is off until the owner turns it on with a passphrase and how often", async () => {
    const { fake, dialog } = await signedIn();
    expect(await within(dialog).findByText("Automatic copies")).toBeDefined();
    const turnOn = within(dialog).getByRole("button", { name: "Turn on" });
    expect(turnOn).toHaveProperty("disabled", true);

    typePassphrase(dialog, "correct horse");
    expect(turnOn).toHaveProperty("disabled", false);
    fireEvent.click(within(dialog).getByRole("button", { name: "How often: Every day" }));
    fireEvent.click(await screen.findByRole("option", { name: "Every week" }));
    fireEvent.click(turnOn);

    expect(
      await within(dialog).findByText("On: a copy every week, if something changed."),
    ).toBeDefined();
    expect(calls(fake, "autobackup.enable").at(-1)?.params).toEqual({
      passphrase: "correct horse",
      every: "weekly",
    });
    // The passphrase is not kept on the screen once it is on.
    expect(within(dialog).queryByRole("button", { name: "Turn on" })).toBeNull();
  });

  test("it shows the last copy, the next look and changes how often", async () => {
    const { fake, dialog } = await signedIn();
    typePassphrase(dialog, "correct horse");
    fireEvent.click(within(dialog).getByRole("button", { name: "Turn on" }));
    await within(dialog).findByText(/On: a copy every day/);
    expect(within(dialog).getByText(/No copy sent yet\./)).toBeDefined();

    act(() => fakeAutoBackupSent(fake));
    expect(await within(dialog).findByText(/Last copy sent:/)).toBeDefined();
    expect(within(dialog).getByText(/Next look:/)).toBeDefined();

    fireEvent.click(within(dialog).getByRole("button", { name: "How often: Every day" }));
    fireEvent.click(await screen.findByRole("option", { name: "Every week" }));
    await within(dialog).findByText(/On: a copy every week/);
    expect(calls(fake, "autobackup.set_every").at(-1)?.params).toEqual({ every: "weekly" });
  });

  test("a failed try says why, in words", async () => {
    const { fake, dialog } = await signedIn();
    typePassphrase(dialog, "correct horse");
    fireEvent.click(within(dialog).getByRole("button", { name: "Turn on" }));
    await within(dialog).findByText(/On: a copy every day/);

    act(() => setFakeAutoBackup(fake, { lastError: "offline" }));
    expect(await within(dialog).findByText("The last try failed")).toBeDefined();
    expect(
      within(dialog).getByText("Could not reach the cloud. Check your connection and try again."),
    ).toBeDefined();
  });

  test("turning it off brings back the passphrase fields", async () => {
    const { fake, dialog } = await signedIn();
    typePassphrase(dialog, "correct horse");
    fireEvent.click(within(dialog).getByRole("button", { name: "Turn on" }));
    await within(dialog).findByText(/On: a copy every day/);

    fireEvent.click(within(dialog).getByRole("button", { name: "Turn off" }));
    expect(await within(dialog).findByRole("button", { name: "Turn on" })).toBeDefined();
    expect(calls(fake, "autobackup.disable")).toHaveLength(1);
  });

  test("where the computer has no credential store it says so and offers nothing", async () => {
    const { fake, dialog } = await signedIn();
    act(() => setFakeAutoBackup(fake, { available: false }));
    expect(
      await within(dialog).findByText("Automatic copies work only on Windows for now."),
    ).toBeDefined();
    expect(within(dialog).queryByRole("button", { name: "Turn on" })).toBeNull();
  });

  test("a short passphrase keeps the button off", async () => {
    const { fake, dialog } = await signedIn();
    // The screen already holds back a short one; the fake refuses it too.
    typePassphrase(dialog, "short");
    expect(within(dialog).getByRole("button", { name: "Turn on" })).toHaveProperty(
      "disabled",
      true,
    );
    expect(calls(fake, "autobackup.enable")).toHaveLength(0);
  });
});
