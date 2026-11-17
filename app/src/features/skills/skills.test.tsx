import { act, cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { setLocaleChoice } from "../../i18n";
import { FakeBotloft } from "../../lib/fake";
import { crewOpened, openBot, renderApp } from "../../test/app";

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
  fake.skills.add({
    name: "deploy",
    description: "Ships the app.",
    allowedTools: ["Bash(npm run build *)"],
    hasFiles: true,
  });
  fake.skills.add({ name: "pdf", description: "Reads PDFs.", source: "claude_ai" });
  return { fake, scout, cody };
}

/** Settings, on the Skills page. */
async function openSkills() {
  fireEvent.click(await screen.findByRole("button", { name: /Ana Lima/ }));
  fireEvent.click(screen.getByRole("menuitem", { name: "Settings" }));
  const page = screen.getByRole("region", { name: "Settings" });
  fireEvent.click(within(page).getByRole("tab", { name: "Skills" }));
  return page;
}

describe("Settings > Skills (spec 38.4)", () => {
  test("each skill is a card, and its dialog gives it to agents on Claude Code", async () => {
    const { fake, scout } = crewOf();
    renderApp(fake);
    const page = await openSkills();
    const deploy = await within(page).findByRole("button", { name: /deploy/ });
    expect(within(deploy).getByText("Ships the app.")).toBeDefined();
    expect(within(deploy).getByText("Yours · No agent yet")).toBeDefined();
    expect(within(page).getByText("claude.ai · No agent yet")).toBeDefined();

    fireEvent.click(deploy);
    const dialog = await screen.findByRole("dialog", { name: "deploy" });
    expect(
      within(dialog).getByText(
        "While it is used, it runs these without asking you: Bash(npm run build *)",
      ),
    ).toBeDefined();
    expect(within(dialog).getByText("It brings other files, such as scripts.")).toBeDefined();
    // Only agents on Claude Code have a switch.
    expect(within(dialog).queryByRole("switch", { name: "Cody can use it" })).toBeNull();
    fireEvent.click(within(dialog).getByRole("switch", { name: "Scout can use it" }));
    await waitFor(() => expect(fake.skills.skills[0]?.botIds).toEqual([scout.id]));
    expect(fake.calls.find((each) => each.method === "skills.share")?.params).toEqual({
      name: "deploy",
      botIds: [scout.id],
    });
    fireEvent.click(within(dialog).getByRole("button", { name: "Close" }));
    expect(await within(page).findByText("Yours · 1 agent")).toBeDefined();
  });

  test("one switch gives a skill to every agent, and taking it back leaves the choices", async () => {
    const { fake, scout } = crewOf();
    renderApp(fake);
    const page = await openSkills();
    fireEvent.click(await within(page).findByRole("button", { name: /deploy/ }));
    const dialog = await screen.findByRole("dialog", { name: "deploy" });
    expect(within(dialog).getByText("The agents you make later too.")).toBeDefined();
    fireEvent.click(within(dialog).getByRole("switch", { name: "Every agent" }));
    const locked = await within(dialog).findByRole("switch", { name: "Scout can use it" });
    await waitFor(() => expect(locked.getAttribute("aria-checked")).toBe("true"));
    expect(locked).toHaveProperty("disabled", true);
    expect(fake.calls.find((each) => each.method === "skills.everyone")?.params).toEqual({
      name: "deploy",
      on: true,
    });
    fireEvent.click(within(dialog).getByRole("button", { name: "Close" }));
    expect(await within(page).findByText("Yours · Every agent")).toBeDefined();

    fireEvent.click(within(page).getByRole("button", { name: /deploy/ }));
    const again = await screen.findByRole("dialog", { name: "deploy" });
    fireEvent.click(within(again).getByRole("switch", { name: "Every agent" }));
    const free = within(again).getByRole("switch", { name: "Scout can use it" });
    await waitFor(() => expect(free).toHaveProperty("disabled", false));
    expect(free.getAttribute("aria-checked")).toBe("false");
    expect(fake.skills.skills[0]?.botIds).not.toContain(scout.id);
  });

  test("every skill goes to every agent after a question that says what they run", async () => {
    const { fake } = crewOf();
    renderApp(fake);
    const page = await openSkills();
    const all = await within(page).findByRole("switch", { name: "Every skill for every agent" });
    fireEvent.click(all);
    const ask = await screen.findByRole("dialog", { name: "Give all 2 skills to every agent?" });
    expect(
      within(ask).getByText("1 of them runs tools without asking you, or commands by itself."),
    ).toBeDefined();
    fireEvent.click(within(ask).getByRole("button", { name: "Give to every agent" }));
    await waitFor(() => expect(fake.skills.everyone).toBe(true));
    expect(await within(page).findByText("claude.ai · Every agent")).toBeDefined();
    expect(all.getAttribute("aria-checked")).toBe("true");

    // Each skill says so and cannot be changed on its own.
    fireEvent.click(within(page).getByRole("button", { name: /pdf/ }));
    const dialog = await screen.findByRole("dialog", { name: "pdf" });
    expect(
      within(dialog).getByText("Every skill is on for every agent, at the top of Skills."),
    ).toBeDefined();
    expect(within(dialog).getByRole("switch", { name: "Every agent" })).toHaveProperty(
      "disabled",
      true,
    );
    fireEvent.click(within(dialog).getByRole("button", { name: "Close" }));

    // Off at once, without asking.
    fireEvent.click(all);
    await waitFor(() => expect(fake.skills.everyone).toBe(false));
    expect(screen.queryByRole("dialog")).toBeNull();
    expect(await within(page).findByText("claude.ai · No agent yet")).toBeDefined();
  });

  test("a skill too big to copy says so and cannot be given", async () => {
    const { fake } = crewOf();
    fake.skills.add({ name: "web-qa", description: "Checks a site.", tooBig: true });
    renderApp(fake);
    const page = await openSkills();
    expect(await within(page).findByText("Yours · Too big to give")).toBeDefined();
    fireEvent.click(within(page).getByRole("switch", { name: "Every skill for every agent" }));
    const ask = await screen.findByRole("dialog", { name: "Give all 2 skills to every agent?" });
    expect(within(ask).getByText("1 is too big to give and stays out.")).toBeDefined();
    fireEvent.click(within(ask).getByRole("button", { name: "Cancel" }));

    fireEvent.click(within(page).getByRole("button", { name: /web-qa/ }));
    const dialog = await screen.findByRole("dialog", { name: "web-qa" });
    expect(within(dialog).getByRole("alert").textContent).toContain("Too big to give to agents");
    expect(within(dialog).queryByRole("switch")).toBeNull();
  });

  test("with many skills, a search finds one by its name or what it does", async () => {
    const { fake } = crewOf();
    for (const name of ["animate", "brandkit", "graphify", "humanizer", "minimalist-ui"]) {
      fake.skills.add({
        name,
        description: name === "graphify" ? "Builds a knowledge graph." : "",
      });
    }
    renderApp(fake);
    const page = await openSkills();
    const search = await within(page).findByRole("searchbox", { name: "Search skills" });
    fireEvent.change(search, { target: { value: "HUMAN" } });
    const cards = within(page).getAllByRole("listitem");
    expect(cards.map((card) => card.textContent?.split("No description")[0])).toEqual([
      "humanizer",
    ]);
    fireEvent.change(search, { target: { value: "knowledge" } });
    expect(within(page).getByRole("button", { name: /graphify/ })).toBeDefined();
    fireEvent.change(search, { target: { value: "nothing like it" } });
    expect(within(page).getByText('No skill matches "nothing like it".')).toBeDefined();
  });
});

describe("skills in the chat and the details (spec 38.5)", () => {
  test("an agent asks for a skill and the owner gives it", async () => {
    const { fake, scout } = crewOf();
    fake.setBotState(scout.id, "idle");
    fake.chat.ask(
      scout.id,
      "mcp__botloft__ask_skill",
      "deploy",
      JSON.stringify({ name: "deploy", why: "To ship the site." }),
    );
    renderApp(fake);
    await crewOpened("Ops");
    openBot("Scout");
    const card = await screen.findByRole("region", { name: "Scout asks for the skill deploy" });
    expect(within(card).getByText("To ship the site.")).toBeDefined();
    fireEvent.click(within(card).getByRole("button", { name: "Give the skill" }));
    expect(await screen.findByText("You gave deploy to Scout")).toBeDefined();
  });

  test("the details list the agent's skills and lead to the page", async () => {
    const { fake, scout } = crewOf();
    fake.skills.skills[0]?.botIds.push(scout.id);
    renderApp(fake);
    await crewOpened("Ops");
    openBot("Scout");
    fireEvent.click(await screen.findByRole("button", { name: "Show details" }));
    const details = await screen.findByRole("complementary", { name: "About Scout" });
    const row = await within(details).findByRole("button", { name: /^Skills/ });
    await waitFor(() => expect(row.textContent).toContain("deploy"));
    fireEvent.click(row);
    const page = await screen.findByRole("region", { name: "Settings" });
    expect(await within(page).findByRole("button", { name: /deploy/ })).toBeDefined();
  });
});
