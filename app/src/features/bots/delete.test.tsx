import { cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { RpcError } from "../../lib/rpc";
import { crewOpened, openBot, openTab, renderApp, sidebar } from "../../test/app";

/** The last call that is not the Bot agency loading, which an empty crew page does. */
const lastCall = (fake: FakeBotloft) =>
  fake.calls.filter((call) => call.method !== "catalog.list").at(-1);

afterEach(cleanup);

/** A crew "Ops" with Scout and Writer, open on the crew's page. */
async function twoBots() {
  const fake = new FakeBotloft();
  const crew = fake.addCrew("Ops");
  const scout = fake.addBot(crew.id, "Scout");
  const writer = fake.addBot(crew.id, "Writer");
  renderApp(fake);
  await crewOpened("Ops");
  return { fake, crew, scout, writer };
}

const inSidebar = (name: string) =>
  within(sidebar()).queryByRole("button", { name: new RegExp(name) });

/** Picks Delete in the open bot's menu and returns the confirmation. */
function askToDelete(name: string) {
  fireEvent.click(screen.getByRole("button", { name: "More agent actions" }));
  fireEvent.click(screen.getByRole("menuitem", { name: "Delete agent" }));
  return screen.getByRole("dialog", { name: `Delete ${name}?` });
}

describe("deleting an agent", () => {
  test("says what goes and what stays, then removes the agent", async () => {
    const { fake, scout } = await twoBots();
    openBot("Scout");
    await screen.findByRole("heading", { level: 1, name: "Scout" });
    const dialog = askToDelete("Scout");
    expect(dialog.textContent).toContain(
      "Scout stops now and leaves Botloft for good, with its conversation, its routines and the tasks it was part of. This can't be undone.",
    );
    expect(dialog.textContent).toContain("Scout's folder stays on your computer");
    expect(within(dialog).getByText(scout.workspace)).toBeDefined();
    expect(dialog.textContent).not.toContain("without a chief");

    fireEvent.click(within(dialog).getByRole("button", { name: "Delete agent" }));
    // Back on the crew's page, without the bot.
    await waitFor(() => expect(inSidebar("Scout")).toBeNull());
    expect(screen.getByRole("heading", { level: 1, name: "Ops" })).toBeDefined();
    expect(lastCall(fake)).toEqual({ method: "bots.delete", params: { botId: scout.id } });
    expect(fake.bots.has(scout.id)).toBe(false);
    expect(inSidebar("Writer")).not.toBeNull();
  });

  test("cancelling keeps the agent", async () => {
    const { fake, scout } = await twoBots();
    openBot("Scout");
    await screen.findByRole("heading", { level: 1, name: "Scout" });
    const dialog = askToDelete("Scout");
    fireEvent.click(within(dialog).getByRole("button", { name: "Cancel" }));
    expect(screen.queryByRole("dialog")).toBeNull();
    expect(fake.bots.has(scout.id)).toBe(true);
    expect(fake.calls.some((call) => call.method === "bots.delete")).toBe(false);
  });

  test("works from the right-click menu, for an agent that is not open", async () => {
    const { fake, scout } = await twoBots();
    openBot("Writer");
    await screen.findByRole("heading", { level: 1, name: "Writer" });
    fireEvent.contextMenu(inSidebar("Scout") as HTMLElement, { clientX: 80, clientY: 120 });
    const menu = screen.getByRole("menu", { name: "Actions for Scout" });
    // Delete comes last, after Archive.
    expect(within(menu).getAllByRole("menuitem").at(-1)?.textContent).toBe("Delete agent");
    fireEvent.click(within(menu).getByRole("menuitem", { name: "Delete agent" }));
    const dialog = screen.getByRole("dialog", { name: "Delete Scout?" });
    fireEvent.click(within(dialog).getByRole("button", { name: "Delete agent" }));
    await waitFor(() => expect(inSidebar("Scout")).toBeNull());
    expect(fake.bots.has(scout.id)).toBe(false);
    expect(screen.getByRole("heading", { level: 1, name: "Writer" })).toBeDefined();
  });

  test("warns that the crew loses its chief", async () => {
    const { fake, crew, scout } = await twoBots();
    await fake.call("crews.setLead", { crewId: crew.id, botId: scout.id });
    openBot("Scout");
    await screen.findByRole("heading", { level: 1, name: "Scout" });
    const dialog = askToDelete("Scout");
    expect(dialog.textContent).toContain("Ops will be left without a chief.");
  });

  test("a delete that fails says so and keeps the agent", async () => {
    const { fake, scout } = await twoBots();
    openBot("Scout");
    await screen.findByRole("heading", { level: 1, name: "Scout" });
    const dialog = askToDelete("Scout");
    fake.failNext("bots.delete", new RpcError(-32603, "internal error"));
    fireEvent.click(within(dialog).getByRole("button", { name: "Delete agent" }));
    await screen.findByText(/Could not delete the agent/);
    expect(fake.bots.has(scout.id)).toBe(true);
    expect(inSidebar("Scout")).not.toBeNull();
  });

  test("what it sent stays in the timeline, from an agent no longer in the crew", async () => {
    const { fake, scout, writer } = await twoBots();
    fake.conversation.say({ from: scout.id, to: writer.id, body: "The draft is in shared/." });
    await fake.call("bots.delete", { botId: scout.id });
    openTab("Timeline");
    const timeline = screen.getByRole("tabpanel", { name: "Timeline" });
    const message = (await within(timeline).findByText("The draft is in shared/.")).closest("li");
    expect(message?.textContent).toContain("an agent no longer in the crew");
  });
});

describe("deleting a crew", () => {
  function askToDeleteCrew(name: string) {
    fireEvent.click(screen.getByRole("button", { name: "More crew actions" }));
    fireEvent.click(screen.getByRole("menuitem", { name: "Delete crew" }));
    return screen.getByRole("dialog", { name: `Delete ${name}?` });
  }

  test("says what goes and what stays, then removes the crew and its agents", async () => {
    const { fake, crew, scout } = await twoBots();
    const docs = fake.addCrew("Docs");
    await waitFor(() =>
      expect(within(sidebar()).getByRole("button", { name: "Docs" })).toBeDefined(),
    );
    const dialog = askToDeleteCrew("Ops");
    expect(dialog.textContent).toContain(
      "Ops and its 2 agents stop now and leave Botloft for good, with their conversations, routines and tasks. This can't be undone.",
    );
    expect(dialog.textContent).toContain("The folders stay on your computer");
    expect(within(dialog).getByText(crew.workFolder)).toBeDefined();

    fireEvent.click(within(dialog).getByRole("button", { name: "Delete crew" }));
    await waitFor(() =>
      expect(within(sidebar()).queryByRole("button", { name: "Ops" })).toBeNull(),
    );
    expect(lastCall(fake)).toEqual({ method: "crews.delete", params: { crewId: crew.id } });
    expect(fake.bots.has(scout.id)).toBe(false);
    expect(inSidebar("Scout")).toBeNull();
    expect(fake.crews.has(docs.id)).toBe(true);
    expect(within(sidebar()).getByRole("button", { name: "Docs" })).toBeDefined();
  });

  test("an empty crew and a crew of one read right", async () => {
    const fake = new FakeBotloft();
    const crew = fake.addCrew("Ops");
    renderApp(fake);
    await crewOpened("Ops");
    const empty = askToDeleteCrew("Ops");
    expect(empty.textContent).toContain("Ops leaves Botloft for good. This can't be undone.");
    fireEvent.click(within(empty).getByRole("button", { name: "Cancel" }));

    fake.addBot(crew.id, "Scout");
    await waitFor(() => expect(inSidebar("Scout")).not.toBeNull());
    expect(askToDeleteCrew("Ops").textContent).toContain("Ops and its agent stop now");
  });

  test("the last crew gone, the app is back at the start", async () => {
    const { fake } = await twoBots();
    const dialog = askToDeleteCrew("Ops");
    fireEvent.click(within(dialog).getByRole("button", { name: "Delete crew" }));
    expect(await screen.findByRole("heading", { name: "Welcome to Botloft" })).toBeDefined();
    expect(fake.crews.size).toBe(0);
  });
});

describe("sending the folder to the Recycle Bin", () => {
  const box = (dialog: HTMLElement, name: string) =>
    within(dialog).getByRole("checkbox", { name }) as HTMLInputElement;

  test("is off until the owner asks, then the agent's folder goes with it", async () => {
    const { fake, scout } = await twoBots();
    openBot("Scout");
    await screen.findByRole("heading", { level: 1, name: "Scout" });
    const dialog = askToDelete("Scout");
    const recycle = box(dialog, "Move this folder to the Recycle Bin");
    expect(recycle.checked).toBe(false);

    fireEvent.click(recycle);
    expect(dialog.textContent).toContain(
      "Scout's folder goes to the Recycle Bin, where you can still get it back:",
    );
    expect(dialog.textContent).not.toContain("stays on your computer");
    expect(within(dialog).getByText(scout.workspace)).toBeDefined();

    fireEvent.click(within(dialog).getByRole("button", { name: "Delete agent" }));
    await waitFor(() => expect(inSidebar("Scout")).toBeNull());
    expect(lastCall(fake)).toEqual({
      method: "bots.delete",
      params: { botId: scout.id, recycleFolder: true },
    });
    expect(fake.recycled).toEqual([scout.workspace]);
  });

  test("says so when the folder could not go, and where it still is", async () => {
    const { fake, scout } = await twoBots();
    fake.recycleError = "Windows cannot put this folder in the Recycle Bin";
    openBot("Scout");
    await screen.findByRole("heading", { level: 1, name: "Scout" });
    const dialog = askToDelete("Scout");
    fireEvent.click(box(dialog, "Move this folder to the Recycle Bin"));
    fireEvent.click(within(dialog).getByRole("button", { name: "Delete agent" }));

    const notice = await screen.findByText(/did not go to the Recycle Bin/);
    expect(notice.textContent).toBe(
      `The folder ${scout.workspace} did not go to the Recycle Bin and is still there: Windows cannot put this folder in the Recycle Bin`,
    );
    // The bot is deleted all the same.
    expect(fake.bots.has(scout.id)).toBe(false);
    expect(fake.recycled).toEqual([]);
  });

  test("a crew's own folder goes whole, and a work folder the owner chose never does", async () => {
    const fake = new FakeBotloft();
    const ops = fake.addCrew("Ops");
    const site = await fake.call("crews.create", {
      name: "Site",
      workFolder: "C:\\Projects\\Site",
    });
    fake.addBot(ops.id, "Scout");
    renderApp(fake);
    await crewOpened("Ops");

    fireEvent.click(screen.getByRole("button", { name: "More crew actions" }));
    fireEvent.click(screen.getByRole("menuitem", { name: "Delete crew" }));
    const dialog = screen.getByRole("dialog", { name: "Delete Ops?" });
    fireEvent.click(box(dialog, "Move the crew's folders to the Recycle Bin"));
    expect(dialog.textContent).toContain("The crew's folder goes to the Recycle Bin");
    // The crew's folder itself, not only `shared` inside it.
    expect(within(dialog).getByText("C:\\Users\\owner\\Botloft\\ops")).toBeDefined();
    fireEvent.click(within(dialog).getByRole("button", { name: "Delete crew" }));
    await waitFor(() => expect(fake.recycled).toEqual(["C:\\Users\\owner\\Botloft\\ops"]));
    expect(lastCall(fake)?.params).toEqual({ crewId: ops.id, recycleFolder: true });

    fireEvent.click(within(sidebar()).getByRole("button", { name: "Site" }));
    await crewOpened("Site");
    fireEvent.click(screen.getByRole("button", { name: "More crew actions" }));
    fireEvent.click(screen.getByRole("menuitem", { name: "Delete crew" }));
    const chosen = screen.getByRole("dialog", { name: "Delete Site?" });
    fireEvent.click(box(chosen, "Move the crew's folders to the Recycle Bin"));
    expect(chosen.textContent).toContain("The work folder you chose stays where it is:");
    expect(within(chosen).getByText(site.workFolder)).toBeDefined();
  });
});
