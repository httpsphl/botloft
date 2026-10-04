import { act, cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { crewOpened, rail, renderApp } from "../../test/app";
import { snippetParts } from "./SearchResult";

afterEach(cleanup);

/** Two crews; Scout talked about the weather a while ago. */
function world() {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const home = fake.addCrew("Home");
  const scout = fake.addBot(ops.id, "Scout");
  const cook = fake.addBot(home.id, "Cook");
  fake.setBotState(scout.id, "idle");
  fake.setBotState(cook.id, "idle");
  void fake.call("messages.send", { botId: scout.id, body: "Previsão do tempo para sábado?" });
  const reply = fake.chat.reply(scout.id, "Sábado vai chover à tarde.");
  for (let n = 0; n < 60; n += 1) {
    fake.chat.reply(scout.id, `Later note ${n}`);
  }
  fake.chat.reply(cook.id, "Rain means soup on Saturday.");
  return { fake, scout, cook, reply };
}

const field = () => screen.getByLabelText("Search the chats") as HTMLInputElement;

async function search(text: string) {
  fireEvent.change(field(), { target: { value: text } });
  return screen.findByRole("list", { name: "Results" });
}

describe("searching the chats", () => {
  test("Ctrl+K opens the search, and words are found without accents", async () => {
    const { fake } = world();
    renderApp(fake);
    await crewOpened("Ops");
    fireEvent.keyDown(window, { key: "k", ctrlKey: true });
    expect(document.activeElement).toBe(await screen.findByLabelText("Search the chats"));

    const results = await search("sabado");
    const rows = within(results).getAllByRole("listitem");
    expect(rows).toHaveLength(2);
    expect(rows[0]?.textContent).toContain("Scout: Sábado vai chover à tarde.");
    expect(rows[1]?.textContent).toContain("You: Previsão do tempo para sábado?");
    expect(within(rows[0] as HTMLElement).getByText("Sábado").tagName).toBe("MARK");

    fireEvent.change(field(), { target: { value: "x" } });
    expect(await screen.findByText("Type at least 2 letters")).toBeDefined();
    fireEvent.change(field(), { target: { value: "snow" } });
    expect(await screen.findByText("Nothing found for “snow”")).toBeDefined();
  });

  test("the search looks in one crew when asked", async () => {
    const { fake } = world();
    renderApp(fake);
    await crewOpened("Ops");
    fireEvent.click(within(rail()).getByRole("button", { name: "Search" }));
    const all = await search("saturday");
    expect(within(all).getAllByRole("listitem")).toHaveLength(1);
    fireEvent.click(screen.getByRole("button", { name: /All crews/ }));
    fireEvent.click(screen.getByRole("option", { name: "Ops" }));
    expect(await screen.findByText("Nothing found for “saturday”")).toBeDefined();
    expect(fake.calls.at(-1)?.params).toMatchObject({
      query: "saturday",
      crewId: expect.any(String),
    });
  });

  test("a result opens its chat at that point, marked", async () => {
    const { fake, scout, reply } = world();
    renderApp(fake);
    await crewOpened("Ops");
    fireEvent.keyDown(window, { key: "k", ctrlKey: true });
    const results = await search("chover");
    fireEvent.click(within(results).getByRole("button", { name: /Scout/ }));

    const chat = await screen.findByRole("list", { name: "Messages" });
    expect(
      fake.calls.some(
        (call) =>
          call.method === "chat.history" &&
          (call.params as { until?: string }).until === reply.id &&
          (call.params as { botId: string }).botId === scout.id,
      ),
    ).toBe(true);
    const found = within(chat).getByText("Sábado vai chover à tarde.").closest("li");
    await waitFor(() => expect(found?.className).toContain("search-focus"));
    // The rest of the chat is still there, older items a click away.
    expect(within(chat).getByText("Later note 59")).toBeDefined();
    expect(screen.getByRole("button", { name: "Load earlier messages" })).toBeDefined();
    act(() => {
      fake.chat.reply(scout.id, "One more");
    });
    expect(await within(chat).findByText("One more")).toBeDefined();
  });
});

test("snippets drop markdown marks", () => {
  expect(snippetParts("**Done**. See `notes.md` ## Next step")).toEqual([
    { text: "Done. See notes.md Next ", found: false },
    { text: "step", found: true },
  ]);
});

test("snippets split into text and found words", () => {
  expect(snippetParts("a \u0002word\u0003 and \u0002more\u0003")).toEqual([
    { text: "a ", found: false },
    { text: "word", found: true },
    { text: " and ", found: false },
    { text: "more", found: true },
  ]);
});
