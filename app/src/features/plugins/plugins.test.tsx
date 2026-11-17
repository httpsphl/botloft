import { act, cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { setLocaleChoice } from "../../i18n";
import { FakeBotloft } from "../../lib/fake";
import { renderApp } from "../../test/app";

afterEach(() => {
  act(() => setLocaleChoice("en"));
  cleanup();
});

function crewOf() {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const scout = fake.addBot(ops.id, "Scout", "Finds sources");
  const cody = fake.addBot(ops.id, "Cody", "Codes");
  fake.bot(cody.id).agent = "codex";
  fake.plugins.add({
    name: "vercel",
    description: "Build and deploy web apps.",
    version: "0.50.0",
    skills: ["vercel:deploy", "vercel:env"],
    agents: ["vercel:deployment-expert"],
    hooks: ["SessionStart"],
    mcpServers: ["vercel"],
  });
  fake.plugins.add({
    name: "data",
    description: "Write SQL.",
    source: "claude_ai",
    skills: ["data:sql"],
  });
  return { fake, scout, cody };
}

/** Settings, on the Plugins page. */
async function openPlugins() {
  fireEvent.click(await screen.findByRole("button", { name: /Ana Lima/ }));
  fireEvent.click(screen.getByRole("menuitem", { name: "Settings" }));
  const page = screen.getByRole("region", { name: "Settings" });
  fireEvent.click(within(page).getByRole("tab", { name: "Plugins" }));
  return page;
}

describe("Settings > Plugins (spec 41.5)", () => {
  test("each plugin is a card, and its dialog says what it brings and gives it", async () => {
    const { fake, scout } = crewOf();
    renderApp(fake);
    const page = await openPlugins();
    const vercel = await within(page).findByRole("button", { name: /vercel/ });
    expect(within(vercel).getByText("Build and deploy web apps.")).toBeDefined();
    expect(within(vercel).getByText("Claude Code · 2 skills · No agent yet")).toBeDefined();
    expect(within(vercel).getByLabelText("Before giving it")).toBeDefined();
    const data = within(page).getByRole("button", { name: /data/ });
    expect(within(data).getByText("claude.ai · 1 skill · No agent yet")).toBeDefined();
    expect(within(data).queryByLabelText("Before giving it")).toBeNull();

    fireEvent.click(vercel);
    const dialog = await screen.findByRole("dialog", { name: "vercel" });
    expect(within(dialog).getByText("Claude Code · Version 0.50.0")).toBeDefined();
    const skills = within(dialog).getByRole("list", { name: "Skills and commands" });
    expect(
      within(skills)
        .getAllByRole("listitem")
        .map((item) => item.textContent),
    ).toEqual(["/vercel:deploy", "/vercel:env"]);
    expect(within(dialog).getByText("vercel:deployment-expert")).toBeDefined();
    expect(
      within(dialog).getByText(
        /It runs commands by itself, without asking you, at moments of the conversation./,
      ),
    ).toBeDefined();
    expect(within(dialog).getByText("In detail: SessionStart")).toBeDefined();
    expect(
      within(dialog).getByText(
        "Its connected tools do not come along: vercel. To give an agent a tool, use Tools.",
      ),
    ).toBeDefined();
    // Only agents on Claude Code have a switch.
    expect(within(dialog).queryByRole("switch", { name: "Cody can use it" })).toBeNull();
    fireEvent.click(within(dialog).getByRole("switch", { name: "Scout can use it" }));
    await waitFor(() => expect(fake.plugins.plugins[1]?.botIds).toEqual([scout.id]));
    expect(fake.calls.find((each) => each.method === "plugins.share")?.params).toEqual({
      name: "vercel",
      botIds: [scout.id],
    });
    fireEvent.click(within(dialog).getByRole("button", { name: "Close" }));
    expect(await within(page).findByText("Claude Code · 2 skills · 1 agent")).toBeDefined();
  });

  test("one switch gives a plugin to every agent", async () => {
    const { fake } = crewOf();
    renderApp(fake);
    const page = await openPlugins();
    fireEvent.click(await within(page).findByRole("button", { name: /data/ }));
    const dialog = await screen.findByRole("dialog", { name: "data" });
    expect(within(dialog).queryByText("Before giving it")).toBeNull();
    fireEvent.click(within(dialog).getByRole("switch", { name: "Every agent" }));
    const locked = await within(dialog).findByRole("switch", { name: "Scout can use it" });
    await waitFor(() => expect(locked.getAttribute("aria-checked")).toBe("true"));
    expect(locked).toHaveProperty("disabled", true);
    expect(fake.calls.find((each) => each.method === "plugins.everyone")?.params).toEqual({
      name: "data",
      on: true,
    });
    fireEvent.click(within(dialog).getByRole("button", { name: "Close" }));
    expect(await within(page).findByText("claude.ai · 1 skill · Every agent")).toBeDefined();
  });

  test("every plugin goes to every agent after a question that says what runs by itself", async () => {
    const { fake } = crewOf();
    renderApp(fake);
    const page = await openPlugins();
    const all = await within(page).findByRole("switch", { name: "Every plugin for every agent" });
    fireEvent.click(all);
    const ask = await screen.findByRole("dialog", { name: "Give all 2 plugins to every agent?" });
    expect(
      within(ask).getByText("1 of them runs commands by itself, without asking you."),
    ).toBeDefined();
    fireEvent.click(within(ask).getByRole("button", { name: "Give to every agent" }));
    await waitFor(() => expect(fake.plugins.everyone).toBe(true));
    expect(await within(page).findByText("claude.ai · 1 skill · Every agent")).toBeDefined();

    fireEvent.click(within(page).getByRole("button", { name: /vercel/ }));
    const dialog = await screen.findByRole("dialog", { name: "vercel" });
    expect(
      within(dialog).getByText("Every plugin is on for every agent, at the top of Plugins."),
    ).toBeDefined();
    expect(within(dialog).getByRole("switch", { name: "Every agent" })).toHaveProperty(
      "disabled",
      true,
    );
    fireEvent.click(within(dialog).getByRole("button", { name: "Close" }));

    // Off at once, without asking.
    fireEvent.click(all);
    await waitFor(() => expect(fake.plugins.everyone).toBe(false));
    expect(screen.queryByRole("dialog")).toBeNull();
    expect(await within(page).findByText("claude.ai · 1 skill · No agent yet")).toBeDefined();
  });

  test("with many plugins, a search finds one by its name or what it does", async () => {
    const { fake } = crewOf();
    for (const name of ["design", "engineering", "finance", "marketing", "productivity"]) {
      fake.plugins.add({ name, description: name === "finance" ? "Closes the books." : "" });
    }
    renderApp(fake);
    const page = await openPlugins();
    const search = await within(page).findByRole("searchbox", { name: "Search plugins" });
    fireEvent.change(search, { target: { value: "MARKET" } });
    const cards = within(page).getAllByRole("listitem");
    expect(cards.map((card) => card.textContent?.split("No description")[0])).toEqual([
      "marketing",
    ]);
    fireEvent.change(search, { target: { value: "books" } });
    expect(within(page).getByRole("button", { name: /finance/ })).toBeDefined();
    fireEvent.change(search, { target: { value: "nothing like it" } });
    expect(within(page).getByText('No plugin matches "nothing like it".')).toBeDefined();
  });

  test("with no plugins, the page says where they come from", async () => {
    const fake = new FakeBotloft();
    fake.addBot(fake.addCrew("Ops").id, "Scout", "Finds sources");
    renderApp(fake);
    const page = await openPlugins();
    expect(
      await within(page).findByText(
        "No plugins found. The plugins you install and turn on in Claude Code show up here.",
      ),
    ).toBeDefined();
    expect(within(page).queryByRole("switch")).toBeNull();
  });
});
