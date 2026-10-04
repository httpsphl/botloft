import { act, cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { crewOpened, openBot, renderApp, sidebar } from "../../test/app";

afterEach(cleanup);

/** A crew "Ops" with @scout and @writer. */
function crew() {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const scout = fake.addBot(ops.id, "Scout", "Finds sources");
  const writer = fake.addBot(ops.id, "Writer", "Writes drafts");
  fake.setBotState(scout.id, "idle");
  fake.setBotState(writer.id, "idle");
  return { fake, scout, writer };
}

const entry = () => within(sidebar()).getByRole("button", { name: /^Questions/ });

describe("questions to the owner", () => {
  test("a question in the chat is answered with a ready answer", async () => {
    const { fake, scout } = crew();
    renderApp(fake);
    await crewOpened("Ops");
    openBot("Scout");
    const chat = await screen.findByRole("list", { name: "Messages" });

    act(() => {
      fake.chat.tool(scout.id, "mcp__botloft__ask_owner", { status: "done" });
      fake.questions.ask(scout.id, "Which **client** first?", ["Acme", "Globex"]);
    });
    const card = within(chat).getByRole("region", { name: "Scout asks" });
    expect(within(card).getByText("client").tagName).toBe("STRONG");
    // The card stands for the call; no tool line repeats it.
    expect(within(chat).queryByText("Ask you a question")).toBeNull();
    expect(within(sidebar()).getByText("Question: Which client first?")).toBeDefined();

    fireEvent.click(within(card).getByRole("button", { name: "Globex" }));
    await within(chat).findByText("You answered");
    expect(fake.calls.at(-1)).toEqual({
      method: "questions.answer",
      params: { questionId: fake.questions.asked[0]?.question.id, answer: "Globex" },
    });
    const sent = fake.conversation.messages.at(-1);
    expect(sent?.body).toBe("Globex");
    expect(sent?.questionId).toBe(fake.questions.asked[0]?.question.id);
    expect(within(chat).queryByRole("region", { name: "Scout asks" })).toBeNull();
  });

  test("the box gathers open questions, counts them and marks the taskbar", async () => {
    const { fake, scout, writer } = crew();
    const { host } = renderApp(fake);
    await crewOpened("Ops");
    expect(entry().textContent).toBe("Questions");
    expect(host.attention).toBe(false);

    act(() => {
      fake.questions.ask(scout.id, "Which client first?");
      fake.now += 1000;
      fake.questions.ask(writer.id, "Formal or casual?", ["Formal", "Casual"]);
    });
    expect(entry().getAttribute("title")).toBe("2 questions wait for you");
    await waitFor(() => expect(host.attention).toBe(true));

    fireEvent.click(entry());
    const box = await screen.findByRole("region", { name: "Questions" });
    expect(
      screen.getByRole("heading", { level: 1, name: "Questions from your bots" }),
    ).toBeDefined();
    const cards = within(box).getAllByRole("region", { name: /asks$/ });
    expect(cards.map((card) => card.getAttribute("aria-label"))).toEqual([
      "Writer asks",
      "Scout asks",
    ]);

    const field = within(box).getByLabelText("Your answer to Scout");
    fireEvent.change(field, { target: { value: "Acme, then Globex" } });
    fireEvent.keyDown(field, { key: "Enter" });
    await waitFor(() =>
      expect(within(box).queryByRole("region", { name: "Scout asks" })).toBeNull(),
    );
    expect(fake.conversation.messages.at(-1)?.body).toBe("Acme, then Globex");
    expect(entry().getAttribute("title")).toBe("1 question waits for you");

    fireEvent.click(within(box).getByRole("button", { name: "Dismiss" }));
    await within(box).findByText("No question is waiting for you");
    expect(fake.questions.asked[1]?.question.status).toBe("dismissed");
    expect(fake.conversation.messages).toHaveLength(1);
    await waitFor(() => expect(host.attention).toBe(false));
  });

  test("a question in the box leads to its bot's chat", async () => {
    const { fake, scout } = crew();
    renderApp(fake);
    await crewOpened("Ops");
    act(() => {
      fake.questions.ask(scout.id, "Ship on Friday?");
    });
    fireEvent.click(entry());
    const box = await screen.findByRole("region", { name: "Questions" });
    fireEvent.click(within(box).getByRole("button", { name: "Open the chat with Scout" }));
    const chat = await screen.findByRole("list", { name: "Messages" });
    expect(within(chat).getByRole("region", { name: "Scout asks" })).toBeDefined();
    expect(screen.queryByRole("region", { name: "Questions" })).toBeNull();
  });
});
