import { act, cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { FAKE_PASSPHRASE } from "../../lib/fakeBackup";
import { expireFakeCloudLink, openFakeCloudLink } from "../../lib/fakeCloud";
import { FakeHost } from "../../lib/fakeHost";
import { RpcErrorCode } from "../../lib/protocol.gen";
import { RpcError } from "../../lib/rpc";
import { crewOpened, renderApp } from "../../test/app";

afterEach(cleanup);

async function openBackup() {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  fake.addBot(ops.id, "Scout");
  fake.addBot(ops.id, "Writer");
  const host = new FakeHost();
  renderApp(fake, host);
  await crewOpened("Ops");
  fireEvent.click(await screen.findByRole("button", { name: /Ana Lima/ }));
  fireEvent.click(screen.getByRole("menuitem", { name: "Settings" }));
  const dialog = screen.getByRole("region", { name: "Settings" });
  fireEvent.click(within(dialog).getByRole("tab", { name: "Account and phone" }));
  return { fake, host, dialog };
}

const type = (field: HTMLElement, value: string) => fireEvent.change(field, { target: { value } });

/** Signs in through the screen, the way the owner does. */
async function signIn(fake: FakeBotloft, dialog: HTMLElement) {
  type(await within(dialog).findByLabelText("Your e-mail"), "Ana@Exemplo.com");
  fireEvent.click(within(dialog).getByRole("button", { name: "Send me the link" }));
  await within(dialog).findByText(/We sent a link to Ana@Exemplo.com/);
  act(() => openFakeCloudLink(fake));
  await within(dialog).findByText("Signed in as ana@exemplo.com");
}

const tab = (dialog: HTMLElement, name: string) =>
  fireEvent.click(within(dialog).getByRole("tab", { name }));

/** Types the passphrase twice in the cloud block, the first pair of fields. */
function sendCopy(dialog: HTMLElement) {
  const [first] = within(dialog).getAllByLabelText("Passphrase");
  const [again] = within(dialog).getAllByLabelText("Passphrase again");
  type(first as HTMLElement, "correct horse");
  type(again as HTMLElement, "correct horse");
  fireEvent.click(within(dialog).getByRole("button", { name: "Send a copy now" }));
}

describe("the account and the copies in the cloud", () => {
  test("a link is asked for in the app's language and signs in when it is opened", async () => {
    const { fake, dialog } = await openBackup();
    const ask = await within(dialog).findByRole("button", { name: "Send me the link" });
    expect(ask).toHaveProperty("disabled", true);
    await signIn(fake, dialog);
    const call = fake.calls.find((each) => each.method === "cloud.signin");
    expect(call?.params).toEqual({ email: "Ana@Exemplo.com", locale: "en" });
    expect(within(dialog).getByRole("button", { name: "Sign out" })).toBeDefined();
  });

  test("a link that ran out says so and can be asked for again", async () => {
    const { fake, dialog } = await openBackup();
    type(await within(dialog).findByLabelText("Your e-mail"), "ana@exemplo.com");
    fireEvent.click(within(dialog).getByRole("button", { name: "Send me the link" }));
    await within(dialog).findByText(/We sent a link/);
    act(() => expireFakeCloudLink(fake));
    expect(await within(dialog).findByText("The link ran out. Ask for a new one.")).toBeDefined();
    expect(within(dialog).getByRole("button", { name: "Send me the link" })).toBeDefined();
  });

  test("a copy goes up, is listed, and comes back with its passphrase as a light one", async () => {
    const { fake, host, dialog } = await openBackup();
    await signIn(fake, dialog);
    tab(dialog, "Backup");
    expect(await within(dialog).findByText("No copies yet.")).toBeDefined();

    sendCopy(dialog);
    expect(await within(dialog).findByText("Copy sent (47 KB).")).toBeDefined();
    const upload = fake.calls.find((each) => each.method === "cloud.upload");
    expect(upload?.params).toEqual({ passphrase: "correct horse" });
    expect(await within(dialog).findByText(/47 KB$/)).toBeDefined();
    tab(dialog, "Account and phone");
    expect(await within(dialog).findByText(/47 KB of 200 MB used/)).toBeDefined();
    tab(dialog, "Backup");

    fireEvent.click(await within(dialog).findByRole("button", { name: "Restore" }));
    const field = await within(dialog).findByLabelText("The copy's passphrase");
    type(field, FAKE_PASSPHRASE);
    fireEvent.click(within(dialog).getByRole("button", { name: "Open" }));
    expect(await within(dialog).findByText("Ops: 2 bots")).toBeDefined();
    expect(within(dialog).getByText(/The chats and files are not in it/)).toBeDefined();
    fireEvent.click(within(dialog).getByRole("button", { name: "Restore and restart" }));
    await waitFor(() => expect(host.installs).toContain("restart"));
  });

  test("a copy is deleted only after the owner confirms", async () => {
    const { fake, dialog } = await openBackup();
    await signIn(fake, dialog);
    tab(dialog, "Backup");
    await within(dialog).findByText("No copies yet.");
    sendCopy(dialog);
    await within(dialog).findByText("Copy sent (47 KB).");

    fireEvent.click(await within(dialog).findByRole("button", { name: "Delete" }));
    const ask = await screen.findByRole("dialog", { name: "Delete this copy?" });
    fireEvent.click(within(ask).getByRole("button", { name: "Cancel" }));
    await waitFor(() =>
      expect(screen.queryByRole("dialog", { name: "Delete this copy?" })).toBeNull(),
    );
    expect(fake.calls.some((each) => each.method === "cloud.delete")).toBe(false);

    fireEvent.click(await within(dialog).findByRole("button", { name: "Delete" }));
    const again = await screen.findByRole("dialog", { name: "Delete this copy?" });
    fireEvent.click(within(again).getByRole("button", { name: "Delete" }));
    expect(await within(dialog).findByText("No copies yet.")).toBeDefined();
  });

  test("signing out goes back to asking for a link, and deleting the account asks first", async () => {
    const { fake, dialog } = await openBackup();
    await signIn(fake, dialog);

    fireEvent.click(within(dialog).getByRole("button", { name: "Delete my account…" }));
    const ask = await screen.findByRole("dialog", { name: "Delete your account?" });
    fireEvent.click(within(ask).getByRole("button", { name: "Send the link" }));
    expect(
      await within(dialog).findByText(/We sent a link to your e-mail\. Open it and press/),
    ).toBeDefined();
    expect(fake.calls.some((each) => each.method === "cloud.delete_account")).toBe(true);

    fireEvent.click(within(dialog).getByRole("button", { name: "Sign out" }));
    expect(await within(dialog).findByLabelText("Your e-mail")).toBeDefined();
  });

  test("a failure is told in the owner's words", async () => {
    const { fake, dialog } = await openBackup();
    type(await within(dialog).findByLabelText("Your e-mail"), "a@b");
    fake.failNext("cloud.signin", new RpcError(RpcErrorCode.validation, "offline", "offline"));
    fireEvent.click(within(dialog).getByRole("button", { name: "Send me the link" }));
    expect(
      await within(dialog).findByText(
        "Could not reach the cloud. Check your connection and try again.",
      ),
    ).toBeDefined();
  });
});
