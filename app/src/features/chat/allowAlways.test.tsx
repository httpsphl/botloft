import { act, cleanup, fireEvent, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { crewOpened, openBot, renderApp } from "../../test/app";

afterEach(cleanup);

async function openScout() {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const scout = fake.addBot(ops.id, "Scout", "Finds sources");
  fake.setBotState(scout.id, "idle");
  renderApp(fake);
  await crewOpened("Ops");
  openBot("Scout");
  await screen.findByRole("list", { name: "Messages" });
  return { fake, scout };
}

const details = () => screen.getByRole("complementary", { name: "About Scout" });

describe("Allow always", () => {
  test("allows the request for good and lists it in the agent's details, where it can go", async () => {
    const { fake, scout } = await openScout();
    act(() => {
      fake.chat.ask(scout.id, "Bash", "git status", '{"command":"git status"}');
    });
    const card = screen.getByRole("region", { name: "Scout asks to run a command" });
    const always = within(card).getByRole("button", { name: "Always allow this command" });
    expect(always.getAttribute("title")).toBe(
      "Scout will not ask for this again. You can undo it in Scout's details.",
    );
    fireEvent.click(always);
    expect(await screen.findByText("Allowed: run a command")).toBeDefined();
    expect(fake.calls.find((call) => call.method === "approvals.answer")?.params).toMatchObject({
      allow: true,
      always: true,
    });

    fireEvent.click(screen.getByRole("button", { name: /^Show details/ }));
    const rule = await within(details()).findByText("git status");
    expect(rule).toBeDefined();
    fireEvent.click(
      within(details()).getByRole("button", { name: "Ask again: Run a command: git status" }),
    );
    expect(
      await within(details()).findByText(
        'Nothing yet. When Scout asks for something, "Always allow" puts it here.',
      ),
    ).toBeDefined();
  });

  test("names the site, and is not offered for what cannot be allowed for good", async () => {
    const { fake, scout } = await openScout();
    act(() => {
      fake.chat.ask(
        scout.id,
        "WebFetch",
        "https://www.example.com/a",
        '{"url":"https://www.example.com/a"}',
      );
    });
    const page = screen.getByRole("region", { name: /^Scout asks to/ });
    expect(within(page).getByRole("button", { name: "Always allow example.com" })).toBeDefined();
    fireEvent.click(within(page).getByRole("button", { name: "Allow" }));
    await screen.findByText(/^Allowed:/);
    // A plain Allow keeps no rule.
    expect(fake.allow.rules).toEqual([]);

    act(() => {
      fake.chat.ask(scout.id, "Glob", "*.md", '{"pattern":"*.md"}');
    });
    const glob = screen.getByRole("region", { name: /^Scout asks to/ });
    expect(within(glob).queryByRole("button", { name: /^Always allow/ })).toBeNull();
  });
});
