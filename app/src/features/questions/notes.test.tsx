import { act, cleanup, fireEvent, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { crewOpened, openBot, rail, renderApp, sidebar } from "../../test/app";

afterEach(cleanup);

/** A crew "Ops" with @scout. */
function crew() {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const scout = fake.addBot(ops.id, "Scout", "Finds sources");
  fake.setBotState(scout.id, "idle");
  return { fake, scout };
}

const inbox = () => within(rail()).getByRole("button", { name: "Inbox" });

describe("notes to the owner", () => {
  test("a note in the chat closes with Seen, and its tool line stays hidden", async () => {
    const { fake, scout } = crew();
    renderApp(fake);
    await crewOpened("Ops");
    openBot("Scout");
    const chat = await screen.findByRole("list", { name: "Messages" });

    act(() => {
      fake.chat.tool(scout.id, "mcp__botloft__tell_owner", { status: "done" });
      fake.questions.tell(scout.id, "The **report** is in work/report.md.");
    });
    const card = within(chat).getByRole("region", { name: "Scout left you a note" });
    expect(within(card).getByText("report").tagName).toBe("STRONG");
    expect(within(card).queryByRole("button", { name: "Dismiss" })).toBeNull();
    expect(within(chat).queryByText("Leave you a note")).toBeNull();
    expect(within(sidebar()).getByText("Note: The report is in work/report.md.")).toBeDefined();
    expect(inbox().getAttribute("title")).toBe("1 question or note waits for you");

    fireEvent.click(within(card).getByRole("button", { name: "Seen" }));
    await within(chat).findByText("Seen");
    expect(fake.calls.at(-1)).toEqual({
      method: "questions.dismiss",
      params: { questionId: fake.questions.asked[0]?.question.id },
    });
    expect(fake.conversation.messages).toHaveLength(0);
    expect(inbox().getAttribute("title")).toBe("Questions and notes from your agents");
  });

  test("a reply from the inbox goes to the agent, and writing to it closes the rest", async () => {
    const { fake, scout } = crew();
    renderApp(fake);
    await crewOpened("Ops");
    act(() => {
      fake.questions.tell(scout.id, "The site is down since 3 am.");
      fake.now += 1000;
      fake.questions.tell(scout.id, "Backups ran fine.");
    });
    fireEvent.click(inbox());
    const box = await screen.findByRole("region", { name: "Inbox" });
    const cards = within(box).getAllByRole("region", { name: "Scout left you a note" });
    expect(cards).toHaveLength(2);
    // The name is on the line above: the card's head says only "Note".
    expect(within(cards[0] as HTMLElement).getByText("Note")).toBeDefined();

    const [newest] = within(box).getAllByLabelText("Your reply to Scout");
    fireEvent.change(newest as HTMLElement, { target: { value: "Great, thanks." } });
    fireEvent.click(within(cards[0] as HTMLElement).getByRole("button", { name: "Reply" }));
    // Writing to Scout closes its other note too, as seen.
    await within(box).findByText("Nothing is waiting for you");
    const sent = fake.conversation.messages.at(-1);
    expect(sent?.body).toBe("Great, thanks.");
    expect(sent?.questionId).toBe(fake.questions.asked[1]?.question.id);
    expect(fake.questions.asked[1]?.question.status).toBe("answered");
    expect(fake.questions.asked[0]?.question.status).toBe("dismissed");
  });
});
