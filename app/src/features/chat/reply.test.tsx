import { cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { crewOpened, openBot, renderApp } from "../../test/app";

afterEach(cleanup);

async function setup() {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const scout = fake.addBot(ops.id, "Scout", "Finds sources");
  const writer = fake.addBot(ops.id, "Writer", "Writes");
  fake.setBotState(scout.id, "idle");
  const said = fake.chat.reply(scout.id, "Acme signs monthly.\n\nDana approves.");
  fake.chat.turn(scout.id);
  fake.conversation.say({ from: writer.id, to: scout.id, body: "The draft is ready." });
  renderApp(fake);
  await crewOpened("Ops");
  openBot("Scout");
  await screen.findByRole("list", { name: "Messages" });
  return { fake, scout, said };
}

const field = () => screen.getByRole("textbox", { name: "Message to Scout" });
const sent = (fake: FakeBotloft) =>
  fake.calls.filter((call) => call.method === "messages.send").map((call) => call.params);

describe("replying to a message", () => {
  test("quotes the bot's reply over the composer, sends it, and shows it on the bubble", async () => {
    const { fake, scout, said } = await setup();
    fireEvent.click(screen.getByRole("button", { name: "Reply to Scout" }));
    expect(screen.getByText("Replying to Scout")).toBeDefined();
    expect(document.activeElement).toBe(field());

    fireEvent.change(field(), { target: { value: "Make it yearly" } });
    fireEvent.keyDown(field(), { key: "Enter" });
    await waitFor(() =>
      expect(sent(fake)).toEqual([{ botId: scout.id, body: "Make it yearly", replyTo: said.id }]),
    );
    // The bar leaves with the message, and the bubble keeps the quote.
    await waitFor(() => expect(screen.queryByText("Replying to Scout")).toBeNull());
    const quote = await screen.findByRole("button", { name: "Acme signs monthly. Dana approves." });
    expect(quote.title).toBe("Show what this replies to");
  });

  test("replies to another bot's message too, and Esc or X cancels", async () => {
    const { fake } = await setup();
    fireEvent.click(screen.getByRole("button", { name: "Reply to Writer" }));
    const bar = screen.getByText("Replying to Writer").parentElement as HTMLElement;
    expect(within(bar).getByText("The draft is ready.")).toBeDefined();
    fireEvent.keyDown(field(), { key: "Escape" });
    expect(screen.queryByText("Replying to Writer")).toBeNull();

    fireEvent.click(screen.getByRole("button", { name: "Reply to Writer" }));
    fireEvent.click(screen.getByRole("button", { name: "Cancel the reply" }));
    fireEvent.change(field(), { target: { value: "Plain" } });
    fireEvent.keyDown(field(), { key: "Enter" });
    await waitFor(() => expect(sent(fake).at(-1)).not.toHaveProperty("replyTo"));
  });
});
