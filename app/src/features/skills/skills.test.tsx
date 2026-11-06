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
    expect(await within(details).findByText("deploy")).toBeDefined();
    fireEvent.click(within(details).getByRole("button", { name: "Choose skills" }));
    const page = await screen.findByRole("region", { name: "Settings" });
    expect(await within(page).findByRole("button", { name: /deploy/ })).toBeDefined();
  });
});
