import { act, cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { renderApp } from "../../test/app";

afterEach(cleanup);

/** A crew "Ops" with Scout and Writer, free. */
function crew() {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const scout = fake.addBot(ops.id, "Scout", "Finds sources");
  const writer = fake.addBot(ops.id, "Writer", "Writes drafts");
  fake.setBotState(scout.id, "idle");
  fake.setBotState(writer.id, "idle");
  return { fake, scout, writer };
}

const home = () => screen.getByRole("region", { name: "Home" });
const answered = (fake: FakeBotloft) =>
  fake.calls.filter((call) => call.method === "approvals.answer").map((call) => call.params);

describe("the home", () => {
  test("the app opens on it: nothing waits, and the crews are below", async () => {
    const { fake } = crew();
    renderApp(fake);
    expect(await screen.findByRole("heading", { level: 1, name: "Home" })).toBeDefined();
    expect(within(home()).getByText("Nothing waits for you")).toBeDefined();
    expect(screen.getByText("All quiet", { selector: "header p" })).toBeDefined();
    fireEvent.click(within(home()).getByRole("button", { name: /^Ops/ }));
    expect(await screen.findByRole("heading", { level: 1, name: "Ops" })).toBeDefined();
  });

  test("a request from any crew is answered right there, oldest first", async () => {
    const { fake, scout, writer } = crew();
    fake.chat.ask(scout.id, "Bash", "npm test", '{"command":"npm test"}');
    fake.now += 1000;
    fake.chat.ask(writer.id, "Bash", "rm -rf build", '{"command":"rm -rf build"}');
    renderApp(fake);
    const requests = await within(
      await screen.findByRole("region", { name: "Home" }),
    ).findAllByRole("region", { name: /asks to run a command$/ });
    expect(requests.map((card) => card.getAttribute("aria-label"))).toEqual([
      "Scout asks to run a command",
      "Writer asks to run a command",
    ]);
    // The name is on the line above: the card says only what is asked.
    expect(within(requests[0] as HTMLElement).getByText("Run a command")).toBeDefined();
    expect(within(requests[0] as HTMLElement).getByText("npm test")).toBeDefined();
    expect(screen.getByText("2 things wait for you")).toBeDefined();

    fireEvent.click(within(requests[0] as HTMLElement).getByRole("button", { name: "Allow" }));
    await waitFor(() => expect(answered(fake)[0]).toMatchObject({ allow: true }));
    await waitFor(() =>
      expect(
        within(home()).queryByRole("region", { name: "Scout asks to run a command" }),
      ).toBeNull(),
    );

    // Deny asks why first, as in the chat.
    const next = within(home()).getByRole("region", { name: "Writer asks to run a command" });
    fireEvent.click(within(next).getByRole("button", { name: "Deny" }));
    fireEvent.change(within(next).getByRole("textbox"), { target: { value: "keep the build" } });
    fireEvent.click(within(next).getByRole("button", { name: "Deny" }));
    await waitFor(() =>
      expect(answered(fake)[1]).toMatchObject({ allow: false, note: "keep the build" }),
    );
  });

  test("a request too long to show is only denied, and one for good opens the chat", async () => {
    const { fake, scout, writer } = crew();
    const long = JSON.stringify({ command: `echo ${"x".repeat(9000)}` });
    fake.chat.ask(scout.id, "Bash", "echo xxx", long);
    fake.chat.ask(writer.id, "mcp__botloft__suggest_bot", "Researcher", "{}");
    renderApp(fake);
    const page = await screen.findByRole("region", { name: "Home" });
    const cut = await within(page).findByRole("region", { name: "Scout asks to run a command" });
    expect(within(cut).queryByRole("button", { name: "Allow" })).toBeNull();
    expect(within(cut).getByRole("button", { name: "Deny" })).toBeDefined();
    expect(within(cut).getByText(/To allow it, open it in the chat/)).toBeDefined();

    const durable = within(page).getByRole("region", { name: /^Writer asks/ });
    expect(within(durable).queryByRole("button", { name: "Deny" })).toBeNull();
    fireEvent.click(within(durable).getByRole("button", { name: "Open in the chat" }));
    expect(await screen.findByRole("heading", { level: 1, name: "Writer" })).toBeDefined();
  });

  test("a question is answered right there, and goes", async () => {
    const { fake, scout } = crew();
    fake.questions.ask(scout.id, "Papers only, or blogs too?", ["Papers only", "Both"]);
    renderApp(fake);
    const page = await screen.findByRole("region", { name: "Home" });
    const question = await within(page).findByRole("region", { name: "Scout asks" });
    expect(within(question).getByText("Question")).toBeDefined();
    fireEvent.click(within(question).getByRole("button", { name: "Both" }));
    await waitFor(() =>
      expect(within(page).queryByRole("region", { name: "Scout asks" })).toBeNull(),
    );
    expect(within(page).getByText("Nothing waits for you")).toBeDefined();
  });

  test("messages not delivered and a signed-out agent wait there too", async () => {
    const { fake, scout, writer } = crew();
    const { delivery } = fake.conversation.say({ from: scout.id, to: writer.id, body: "lost" });
    fake.conversation.deliver(delivery.id, "dead", "the agent's inbox did not answer");
    renderApp(fake);
    const page = await screen.findByRole("region", { name: "Home" });
    fireEvent.click(await within(page).findByRole("button", { name: "See and retry" }));
    expect(screen.getByRole("dialog", { name: "Messages not delivered" })).toBeDefined();
    fireEvent.keyDown(window, { key: "Escape" });

    act(() => fake.setBotState(writer.id, "auth_error"));
    const line = await within(page).findByText("Writer stopped: its engine is not signed in");
    fireEvent.click(
      within(line.closest("li") as HTMLElement).getByRole("button", { name: "Open" }),
    );
    expect(await screen.findByRole("heading", { level: 1, name: "Writer" })).toBeDefined();
  });

  test("who works and who replied show under their own titles, and open the chat", async () => {
    const { fake, scout, writer } = crew();
    fake.setBotState(scout.id, "busy");
    fake.chat.tool(scout.id, "Bash", { summary: "npm test" });
    renderApp(fake);
    const page = await screen.findByRole("region", { name: "Home" });
    const working = await within(page).findByRole("region", { name: "Working now" });
    expect(within(working).getByText("Run a command · npm test")).toBeDefined();
    expect(within(page).queryByRole("region", { name: "New replies" })).toBeNull();

    // A reply that comes while the home is open is new.
    act(() => {
      fake.chat.reply(writer.id, "The draft is ready.");
    });
    const replies = await within(page).findByRole("region", { name: "New replies" });
    fireEvent.click(within(replies).getByRole("button", { name: /^Writer/ }));
    expect(await screen.findByRole("heading", { level: 1, name: "Writer" })).toBeDefined();
  });

  test("today counts what the crews did, without asking an agent", async () => {
    const { fake, scout, writer } = crew();
    const edit = fake.chat.tool(writer.id, "Edit", {
      file: "C:Work\notes.md",
      lines: { added: 5, removed: 1 },
    });
    fake.chat.finish(edit, "The file has been updated.");
    fake.chat.turn(scout.id);
    renderApp(fake);
    const today = await within(await screen.findByRole("region", { name: "Home" })).findByRole(
      "region",
      { name: "Today" },
    );
    const tile = (label: string) => within(today).getByText(label).closest("div") as HTMLElement;
    expect(await within(today).findByText("Files edited")).toBeDefined();
    expect(tile("Files edited").textContent).toBe("Files edited1+5−1");
    expect(tile("Worked the most").textContent).toBe("Worked the mostScout4.2 s");
  });

  test("a quiet day says so", async () => {
    const { fake } = crew();
    renderApp(fake);
    const today = await within(await screen.findByRole("region", { name: "Home" })).findByRole(
      "region",
      { name: "Today" },
    );
    expect(await within(today).findByText("Nothing done yet today.")).toBeDefined();
  });
});
