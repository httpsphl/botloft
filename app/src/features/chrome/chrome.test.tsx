import { act, cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { setLocaleChoice } from "../../i18n";
import { en } from "../../i18n/en";
import { FakeBotloft } from "../../lib/fake";
import { crewOpened, openBot, renderApp } from "../../test/app";
import { toolAction, toolTitle } from "../chat/toolNames";

afterEach(() => {
  act(() => setLocaleChoice("en"));
  cleanup();
});

function crewOf() {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const scout = fake.addBot(ops.id, "Scout", "Finds sources");
  const writer = fake.addBot(ops.id, "Writer", "Writes");
  const cody = fake.addBot(ops.id, "Cody", "Codes");
  fake.bot(cody.id).agent = "codex";
  return { fake, scout, writer, cody };
}

/** Settings, on the Permissions page, at the "Your Chrome" tab. */
async function openChromeTab() {
  fireEvent.click(await screen.findByRole("button", { name: /Ana Lima/ }));
  fireEvent.click(screen.getByRole("menuitem", { name: "Settings" }));
  const page = screen.getByRole("region", { name: "Settings" });
  fireEvent.click(within(page).getByRole("tab", { name: "Permissions" }));
  fireEvent.click(await within(page).findByRole("tab", { name: "Chrome" }));
  return page;
}

describe("the owner's Chrome on the Permissions page (spec 37.2)", () => {
  test("is given after the risks, and taken back with its X", async () => {
    const { fake } = crewOf();
    renderApp(fake);
    const page = await openChromeTab();
    expect(await within(page).findByText(en.chrome.none)).toBeDefined();

    fireEvent.click(within(page).getByRole("button", { name: "Give your Chrome to an agent" }));
    const dialog = await screen.findByRole("dialog", { name: "Give your Chrome to an agent" });
    // Only agents on Claude Code can be chosen.
    fireEvent.click(within(dialog).getByRole("button", { name: /Agent/ }));
    expect(screen.queryByRole("option", { name: "Cody" })).toBeNull();
    fireEvent.click(screen.getByRole("option", { name: "Writer" }));
    const confirm = within(dialog).getByRole("button", { name: "Give my Chrome" });
    expect((confirm as HTMLButtonElement).disabled).toBe(true);
    fireEvent.click(within(dialog).getByRole("checkbox", { name: "I understand the risks" }));
    fireEvent.click(confirm);

    const writers = await within(page).findByRole("region", { name: "Writer" });
    expect(within(writers).getByText("Your Chrome")).toBeDefined();
    expect(within(page).getByRole("tab", { name: "Chrome (1)" })).toBeDefined();
    expect(fake.calls.find((each) => each.method === "chrome.set")?.params).toMatchObject({
      on: true,
      acceptedRisks: true,
    });

    fireEvent.click(
      within(writers).getByRole("button", { name: "Take your Chrome back from Writer" }),
    );
    await waitFor(() => expect(within(page).queryByRole("region", { name: "Writer" })).toBeNull());
  });

  test("says who is using it now, and stops them", async () => {
    const { fake, scout } = crewOf();
    renderApp(fake);
    const page = await openChromeTab();
    expect(await within(page).findByText(en.chrome.nobody)).toBeDefined();
    act(() => fake.chrome.take(scout.id));
    expect(await within(page).findByText("Scout is using your Chrome now.")).toBeDefined();
    fireEvent.click(within(page).getByRole("button", { name: "Stop" }));
    expect(await within(page).findByText(en.chrome.nobody)).toBeDefined();
    expect(fake.calls.some((each) => each.method === "chrome.stop")).toBe(true);
  });
});

describe("an agent asking for the owner's Chrome (spec 37.5)", () => {
  test("the card shows why and the risks, and gives only once they are understood", async () => {
    const { fake, scout } = crewOf();
    fake.setBotState(scout.id, "idle");
    fake.chat.ask(
      scout.id,
      "mcp__botloft__ask_chrome",
      "To extend a song on suno.com.",
      JSON.stringify({ why: "To extend a song on suno.com." }),
    );
    renderApp(fake);
    await crewOpened("Ops");
    openBot("Scout");
    const card = await screen.findByRole("region", { name: "Scout asks for your Chrome" });
    expect(within(card).getByText("Scout wants to use your Chrome")).toBeDefined();
    expect(within(card).getByText("To extend a song on suno.com.")).toBeDefined();
    const give = within(card).getByRole("button", { name: "Give my Chrome" });
    expect((give as HTMLButtonElement).disabled).toBe(true);
    fireEvent.click(within(card).getByRole("checkbox", { name: "I understand the risks" }));
    fireEvent.click(give);
    expect(await screen.findByText("You gave your Chrome to Scout")).toBeDefined();
  });
});

describe("the tools of the owner's Chrome, in words (spec 37.3)", () => {
  test("each is named, and the site rule reads as in your Chrome", () => {
    expect(toolTitle("mcp__claude-in-chrome__navigate", en.tools)).toBe(
      "Open a page in your Chrome",
    );
    expect(toolTitle("mcp__claude-in-chrome__a_new_tool", en.tools)).toBe("Use your Chrome");
    expect(toolTitle("mcp__claude-in-chrome", en.tools)).toBe("In your Chrome");
    expect(toolAction("mcp__claude-in-chrome__computer", en.tools)).toBe(
      "click and type in your Chrome",
    );
  });
});
