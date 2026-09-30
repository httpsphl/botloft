import { cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { RpcError } from "../../lib/rpc";
import { crewOpened, renderApp } from "../../test/app";

afterEach(cleanup);

/** Opens Settings on its "Archived" part. */
async function openArchived() {
  fireEvent.click(await screen.findByRole("button", { name: /Ana Lima/ }));
  fireEvent.click(screen.getByRole("menuitem", { name: "Settings" }));
  const dialog = screen.getByRole("dialog", { name: "Settings" });
  fireEvent.click(within(dialog).getByRole("tab", { name: "Archived" }));
  return dialog;
}

/** Ops with Writer and an archived Scout, and an archived crew Docs of one bot. */
async function someArchived() {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const docs = fake.addCrew("Docs");
  const scout = fake.addBot(ops.id, "Scout");
  fake.addBot(ops.id, "Writer");
  const editor = fake.addBot(docs.id, "Editor");
  await fake.call("bots.archive", { botId: scout.id });
  await fake.call("crews.archive", { crewId: docs.id });
  renderApp(fake);
  await crewOpened("Ops");
  return { fake, ops, docs, scout, editor };
}

describe("settings, archived", () => {
  test("lists what was archived, which the app shows nowhere else", async () => {
    await someArchived();
    const dialog = await openArchived();
    const list = await within(dialog).findByRole("list", { name: "Archived" });
    const rows = within(list).getAllByRole("listitem");
    expect(rows.map((row) => row.textContent)).toEqual([
      "DocsCrew · 1 bot · archived nowDelete",
      "ScoutBot in Ops · archived nowDelete",
    ]);
    // A bot of an archived crew goes with its crew, not on its own.
    expect(within(list).queryByText("Editor")).toBeNull();
    expect(dialog.textContent).toContain("Botloft still keeps their conversations");
  });

  test("deletes an archived bot, saying it leaves rather than stops", async () => {
    const { fake, scout } = await someArchived();
    const dialog = await openArchived();
    fireEvent.click(await within(dialog).findByRole("button", { name: "Delete Scout" }));
    const confirm = screen.getByRole("dialog", { name: "Delete Scout?" });
    expect(confirm.textContent).toContain("Scout leaves Botloft for good");
    expect(confirm.textContent).not.toContain("stops now");
    expect(within(confirm).getByText(scout.workspace)).toBeDefined();
    fireEvent.click(within(confirm).getByRole("button", { name: "Delete bot" }));
    await waitFor(() => expect(within(dialog).queryByText("Scout")).toBeNull());
    expect(fake.bots.has(scout.id)).toBe(false);
    expect(within(dialog).getByText("Docs")).toBeDefined();
  });

  test("deletes an archived crew with its bots, then has nothing left", async () => {
    const { fake, docs, scout, editor } = await someArchived();
    await fake.call("bots.delete", { botId: scout.id });
    const dialog = await openArchived();
    fireEvent.click(await within(dialog).findByRole("button", { name: "Delete Docs" }));
    const confirm = screen.getByRole("dialog", { name: "Delete Docs?" });
    expect(confirm.textContent).toContain("Docs and its bot leave Botloft for good");
    expect(within(confirm).getByText(docs.workFolder)).toBeDefined();
    fireEvent.click(within(confirm).getByRole("button", { name: "Delete crew" }));
    expect(await within(dialog).findByText("Nothing is archived.")).toBeDefined();
    expect(fake.crews.has(docs.id)).toBe(false);
    expect(fake.bots.has(editor.id)).toBe(false);
  });

  test("says so when nothing is archived, or when the list cannot be read", async () => {
    const fake = new FakeBotloft();
    fake.addCrew("Ops");
    renderApp(fake);
    await crewOpened("Ops");
    const dialog = await openArchived();
    expect(await within(dialog).findByText("Nothing is archived.")).toBeDefined();

    fake.failNext("archive.list", new RpcError(-32603, "internal error"));
    fireEvent.click(within(dialog).getByRole("tab", { name: "About" }));
    fireEvent.click(within(dialog).getByRole("tab", { name: "Archived" }));
    const alert = await within(dialog).findByRole("alert");
    expect(alert.textContent).toContain("Could not load what is archived");
  });
});
