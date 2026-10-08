import { act, cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { openFakeCloudLink } from "../../lib/fakeCloud";
import { FakeHost } from "../../lib/fakeHost";
import { joinFakePhone } from "../../lib/fakeMobile";
import { crewOpened, renderApp } from "../../test/app";

afterEach(cleanup);

async function openBackup() {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  fake.addBot(ops.id, "Scout");
  renderApp(fake, new FakeHost());
  await crewOpened("Ops");
  fireEvent.click(await screen.findByRole("button", { name: /Ana Lima/ }));
  fireEvent.click(screen.getByRole("menuitem", { name: "Settings" }));
  const dialog = screen.getByRole("dialog", { name: "Settings" });
  fireEvent.click(within(dialog).getByRole("tab", { name: "Backup" }));
  return { fake, dialog };
}

/** Signs in through the screen, the way the owner does. */
async function signIn(fake: FakeBotloft, dialog: HTMLElement) {
  fireEvent.change(await within(dialog).findByLabelText("Your e-mail"), {
    target: { value: "ana@exemplo.com" },
  });
  fireEvent.click(within(dialog).getByRole("button", { name: "Send me the link" }));
  await within(dialog).findByText(/We sent a link to ana@exemplo.com/);
  act(() => openFakeCloudLink(fake));
  await within(dialog).findByText("Signed in as ana@exemplo.com");
}

describe("the phone block in Settings", () => {
  test("waits for the account, then shows a code, compares it and connects the phone", async () => {
    const { fake, dialog } = await openBackup();
    await within(dialog).findByText("Sign in to the account above to use the phone.");
    expect(within(dialog).queryByRole("button", { name: "Connect a phone" })).toBeNull();
    await signIn(fake, dialog);

    expect(await within(dialog).findByText("No phone connected yet.")).toBeTruthy();
    fireEvent.click(within(dialog).getByRole("button", { name: "Connect a phone" }));
    // The code to scan is a picture, with the time it still works.
    expect(
      await within(dialog).findByRole("img", { name: "Code to scan with your phone" }),
    ).toBeTruthy();
    expect(within(dialog).getByText(/The code works for 5:00 more/)).toBeTruthy();

    // A phone scans it: its name, and the code to compare with the phone's screen.
    act(() => joinFakePhone(fake, "Celular da Ana", "482913"));
    expect(await within(dialog).findByText("Celular da Ana wants to connect.")).toBeTruthy();
    expect(within(dialog).getByText("482 913", { selector: "p.font-mono" })).toBeTruthy();
    expect(within(dialog).queryByRole("img", { name: "Code to scan with your phone" })).toBeNull();

    fireEvent.click(within(dialog).getByRole("button", { name: "Connect" }));
    const list = await within(dialog).findByRole("list");
    expect(within(list).getByText("Celular da Ana")).toBeTruthy();
    expect(within(list).getByText("Online now")).toBeTruthy();
    expect(within(dialog).getByRole("button", { name: "Connect a phone" })).toBeTruthy();

    // Disconnecting asks first, and the phone is gone.
    fireEvent.click(within(list).getByRole("button", { name: "Disconnect" }));
    const confirm = await screen.findByRole("dialog", { name: "Disconnect Celular da Ana?" });
    fireEvent.click(within(confirm).getByRole("button", { name: "Disconnect" }));
    await within(dialog).findByText("No phone connected yet.");
    expect(fake.calls.some((call) => call.method === "mobile.revoke")).toBe(true);
  });

  test("cancelling or refusing leaves no phone and brings the button back", async () => {
    const { fake, dialog } = await openBackup();
    await signIn(fake, dialog);
    fireEvent.click(await within(dialog).findByRole("button", { name: "Connect a phone" }));
    await within(dialog).findByRole("img", { name: "Code to scan with your phone" });
    fireEvent.click(within(dialog).getByRole("button", { name: "Cancel" }));
    await within(dialog).findByRole("button", { name: "Connect a phone" });
    expect((await fake.call("mobile.status")).pending).toBeUndefined();

    fireEvent.click(within(dialog).getByRole("button", { name: "Connect a phone" }));
    await within(dialog).findByRole("img", { name: "Code to scan with your phone" });
    act(() => joinFakePhone(fake));
    fireEvent.click(await within(dialog).findByRole("button", { name: "Not this one" }));
    await within(dialog).findByRole("button", { name: "Connect a phone" });
    await waitFor(async () => expect((await fake.call("mobile.status")).phones).toEqual([]));
  });
});
