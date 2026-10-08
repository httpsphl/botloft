import { act, cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test, vi } from "vitest";
import type { ApprovalCard, ChatLine, PhoneItem } from "../lib/protocol.gen";
import { FakePhone } from "./fake";
import { PhoneApp } from "./PhoneApp";
import type { SeenStore } from "./talk";

afterEach(cleanup);

const line = (change: Partial<ChatLine> = {}): ChatLine => ({
  botId: "bot_1",
  name: "Scout",
  color: "#aabbcc",
  crew: "Ops",
  state: "idle",
  lastReplyAt: 100,
  last: { kind: "reply", text: "All done", tool: null, at: 100 },
  ...change,
});

const you = (id: string, text: string, at = 1): PhoneItem => ({
  kind: "you",
  id,
  at,
  text,
  cut: false,
});
const reply = (id: string, text: string, at = 2): PhoneItem => ({
  kind: "reply",
  id,
  at,
  text,
  cut: false,
});

/** A phone with the list loaded, on the Chats tab. */
function withList(...lines: ChatLine[]) {
  const phone = new FakePhone();
  phone.receive({ t: "chats", bots: lines, first: true });
  render(<PhoneApp api={phone} />);
  fireEvent.click(screen.getByRole("tab", { name: /Chats/ }));
  return phone;
}

/** Opens the first bot and hands over its newest page. */
async function openChat(phone: FakePhone, items: PhoneItem[], more = false) {
  fireEvent.click(screen.getByRole("button", { name: /Scout/ }));
  await waitFor(() => expect(phone.asked.some((ask) => ask.t === "history")).toBe(true));
  const ask = phone.asked.find((one) => one.t === "history");
  const req = ask?.t === "history" ? ask.req : 0;
  act(() => phone.receive({ t: "history", req, botId: "bot_1", items, more, done: true }));
}

describe("the list of conversations", () => {
  test("shows each bot with its last words, working, and a dot for a reply not seen", () => {
    const phone = new FakePhone();
    phone.receive({
      t: "chats",
      first: true,
      bots: [
        line(),
        line({
          botId: "bot_2",
          name: "Writer",
          state: "busy",
          last: { kind: "owner", text: "Draft it", tool: null, at: 50 },
        }),
      ],
    });
    phone.set({ seen: { bot_1: 50, bot_2: 100 } });
    render(<PhoneApp api={phone} />);
    // The tab tells there is something new before the tab is opened.
    expect(screen.getByRole("img", { name: "new reply" })).toBeTruthy();
    fireEvent.click(screen.getByRole("tab", { name: /Chats/ }));
    const scout = screen.getByRole("button", { name: /Scout/ });
    expect(scout.textContent).toContain("in Ops");
    expect(scout.textContent).toContain("All done");
    expect(within(scout).getByRole("img", { name: "new reply" })).toBeTruthy();
    const writer = screen.getByRole("button", { name: /Writer/ });
    expect(writer.textContent).toContain("working…");
    expect(within(writer).queryByRole("img")).toBeNull();
  });

  test("says there is nothing, or that it is loading", () => {
    const phone = new FakePhone();
    render(<PhoneApp api={phone} />);
    fireEvent.click(screen.getByRole("tab", { name: /Chats/ }));
    expect(screen.getByText("Loading the bots…")).toBeTruthy();
    act(() => phone.receive({ t: "chats", bots: [], first: true }));
    expect(screen.getByText("No bots yet")).toBeTruthy();
  });

  test("a line that changes moves the bot up", () => {
    const quiet = line({ botId: "bot_2", name: "Writer" });
    delete quiet.last;
    const phone = withList(line(), quiet);
    act(() =>
      phone.receive({
        t: "line",
        bot: line({
          botId: "bot_2",
          name: "Writer",
          last: { kind: "reply", text: "Hi", tool: null, at: 500 },
        }),
      }),
    );
    const names = screen.getAllByRole("button", { name: /Scout|Writer/ }).map((b) => b.textContent);
    expect(names[0]).toContain("Writer");
  });

  test("the number kept for each bot is a time and never a text", async () => {
    const kept: Record<string, number>[] = [];
    const store: SeenStore = { load: () => ({}), save: (seen) => kept.push(seen) };
    const phone = new FakePhone(store);
    phone.receive({ t: "chats", bots: [line()], first: true });
    render(<PhoneApp api={phone} />);
    fireEvent.click(screen.getByRole("tab", { name: /Chats/ }));
    fireEvent.click(screen.getByRole("button", { name: /Scout/ }));
    await waitFor(() => expect(kept.at(-1)).toEqual({ bot_1: 100 }));
  });
});

describe("a conversation", () => {
  test("opens with the newest items, asks to be told what happens and says what the bot does", async () => {
    const phone = withList(line({ state: "busy" }));
    await openChat(phone, [
      you("itm_1", "Hello"),
      { kind: "tool", id: "itm_2", at: 2, summary: "ls -la" },
      reply("itm_3", "Hi **there**"),
    ]);
    expect(phone.asked.map((ask) => ask.t)).toEqual(["watch", "history"]);
    expect(screen.getByText("Hello")).toBeTruthy();
    expect(screen.getByText("Used: ls -la")).toBeTruthy();
    expect(screen.getByText("there").tagName).toBe("STRONG");
    expect(screen.getByRole("status").textContent).toBe("Scout is working…");

    // The reply as it is written, whole each time, then the real item.
    act(() => phone.receive({ t: "live", botId: "bot_1", text: "Writing the rep" }));
    expect(screen.getByText("Writing the rep")).toBeTruthy();
    expect(screen.queryByRole("status")).toBeNull();
    act(() => phone.receive({ t: "live", botId: "bot_1", text: "" }));
    act(() => phone.receive({ t: "item", botId: "bot_1", item: reply("itm_4", "Final", 3) }));
    expect(screen.getByText("Final")).toBeTruthy();
    expect(screen.queryByText("Writing the rep")).toBeNull();

    fireEvent.click(screen.getByRole("button", { name: "Chats" }));
    expect(phone.asked.at(-1)).toEqual({ t: "watch" });
    expect(screen.getByRole("tab", { name: /Chats/ })).toBeTruthy();
  });

  test("older items come on request and go above, without a repeat", async () => {
    const phone = withList(line());
    await openChat(phone, [you("itm_5", "Newer", 5)], true);
    fireEvent.click(screen.getByRole("button", { name: "See older" }));
    const ask = phone.asked.at(-1);
    expect(ask).toMatchObject({ t: "history", botId: "bot_1", before: "itm_5" });
    const req = ask?.t === "history" ? ask.req : 0;
    act(() =>
      phone.receive({
        t: "history",
        req,
        botId: "bot_1",
        items: [you("itm_3", "Old one", 3)],
        more: false,
        done: false,
      }),
    );
    // Not done yet: nothing changes on screen.
    expect(screen.queryByText("Old one")).toBeNull();
    act(() =>
      phone.receive({
        t: "history",
        req,
        botId: "bot_1",
        items: [you("itm_4", "Old two", 4), you("itm_5", "Newer", 5)],
        more: false,
        done: true,
      }),
    );
    const texts = screen.getAllByText(/Old one|Old two|Newer/).map((node) => node.textContent);
    expect(texts).toEqual(["Old one", "Old two", "Newer"]);
    expect(screen.queryByRole("button", { name: "See older" })).toBeNull();
  });

  test("an answer to an older request is ignored", async () => {
    const phone = withList(line());
    await openChat(phone, [you("itm_1", "Now")]);
    act(() =>
      phone.receive({
        t: "history",
        req: 999,
        botId: "bot_1",
        items: [you("itm_0", "Stale")],
        more: false,
        done: true,
      }),
    );
    expect(screen.queryByText("Stale")).toBeNull();
  });

  test("a request that waits shows as its card, with the same buttons", async () => {
    const phone = withList(line());
    const card: ApprovalCard = {
      approvalId: "apr_1",
      bot: { name: "Scout", color: "#aabbcc" },
      crew: "Ops",
      createdAt: 1,
      toolName: "Bash",
      summary: "git status",
      text: "git status",
      cut: false,
      atComputer: false,
    };
    phone.add(card);
    await openChat(phone, [
      {
        kind: "approval",
        id: "itm_a",
        at: 1,
        approvalId: "apr_1",
        summary: "git status",
        status: "pending",
      },
    ]);
    fireEvent.click(screen.getByRole("button", { name: "Allow" }));
    await waitFor(() => expect(phone.calls[0]?.method).toBe("answerApproval"));
    // Once it is answered, the item is one line with how it ended.
    act(() => phone.set({ approvals: [] }));
    act(() =>
      phone.receive({
        t: "item",
        botId: "bot_1",
        item: {
          kind: "approval",
          id: "itm_a",
          at: 1,
          approvalId: "apr_1",
          summary: "git status",
          status: "allowed",
        },
      }),
    );
    expect(screen.getByText(/Asked to: git status — Allowed/)).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Allow" })).toBeNull();
  });

  test("failed turns and notices are told", async () => {
    const phone = withList(line());
    await openChat(phone, [
      { kind: "failed", id: "itm_f", at: 1, error: "rate_limit" },
      { kind: "notice", id: "itm_n", at: 2, level: "warning", text: "Limit reached" },
    ]);
    expect(screen.getByText("The turn failed: rate_limit")).toBeTruthy();
    expect(screen.getByText("Limit reached")).toBeTruthy();
  });
});

describe("writing to a bot", () => {
  const box = () => screen.getByLabelText("Message to Scout") as HTMLTextAreaElement;
  const write = (value: string) => fireEvent.change(box(), { target: { value } });
  const sendButton = () => screen.getByRole("button", { name: "Send" }) as HTMLButtonElement;

  test("shows the message at once as sending, and swaps it for the real one", async () => {
    const phone = withList(line());
    await openChat(phone, []);
    expect(sendButton().disabled).toBe(true);
    write("  Do the report  ");
    expect(sendButton().disabled).toBe(false);
    fireEvent.click(sendButton());
    expect(await screen.findByText("Sending…")).toBeTruthy();
    expect(box().value).toBe("");
    const asked = phone.asked.at(-1);
    expect(asked).toMatchObject({ t: "send", botId: "bot_1", text: "Do the report" });
    const clientId = asked?.t === "send" ? asked.clientId : "";

    act(() => phone.receive({ t: "sent", clientId, ok: true }));
    expect(screen.getByText("Sent. The bot will see it when it can.")).toBeTruthy();
    act(() => phone.receive({ t: "item", botId: "bot_1", item: you("itm_9", "Do the report", 9) }));
    // One bubble only: the pending one is gone.
    expect(screen.getAllByText("Do the report")).toHaveLength(1);
    expect(screen.queryByText(/Sent\./)).toBeNull();
  });

  test("a message that is refused stays marked, and its text comes back to the box", async () => {
    const phone = withList(line());
    await openChat(phone, []);
    write("Too much");
    fireEvent.click(sendButton());
    const asked = phone.asked.at(-1);
    const clientId = asked?.t === "send" ? asked.clientId : "";
    act(() => phone.receive({ t: "sent", clientId, ok: false, reason: "rate_limited" }));
    expect(
      await screen.findByText("Not sent. Too many messages in a minute. Wait a bit."),
    ).toBeTruthy();
    await waitFor(() => expect(box().value).toBe("Too much"));
  });

  test("without a connection it says so, and a text too long is not sent", async () => {
    const phone = withList(line());
    await openChat(phone, []);
    phone.sendable = false;
    write("Hi");
    fireEvent.click(sendButton());
    expect(await screen.findByText("Not sent. Check the connection and try again.")).toBeTruthy();

    phone.sendable = true;
    write("é".repeat(7000));
    fireEvent.click(sendButton());
    expect(await screen.findByText("Too long to send from the phone.")).toBeTruthy();
    expect(phone.asked.filter((ask) => ask.t === "send")).toHaveLength(1);
  });
});

describe("finding the way around", () => {
  const many = () => [
    line({ botId: "bot_1", name: "Scout", crew: "Ops" }),
    line({ botId: "bot_2", name: "Writer", crew: "Marketing", lastReplyAt: 1 }),
    line({ botId: "bot_3", name: "Editor", crew: "Marketing", lastReplyAt: 1 }),
  ];

  test("groups the bots by crew, and the chips narrow the list to one", () => {
    const phone = withList(...many());
    act(() => phone.set({ seen: { bot_1: 100, bot_2: 1, bot_3: 1 } }));
    const sections = screen.getAllByRole("region");
    expect(sections.map((section) => section.getAttribute("aria-label"))).toEqual([
      "Marketing",
      "Ops",
    ]);
    expect(within(sections[0] as HTMLElement).getByText("2 bots")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: /^Ops/ }));
    expect(screen.queryByRole("button", { name: /Writer/ })).toBeNull();
    expect(screen.getByRole("button", { name: /Scout/ })).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "All" }));
    expect(screen.getByRole("button", { name: /Writer/ })).toBeTruthy();
  });

  test("a chip tells when its crew has a reply not seen", () => {
    const phone = withList(...many());
    act(() => phone.set({ seen: { bot_1: 100, bot_2: 0, bot_3: 1 } }));
    const chip = screen.getByRole("button", { name: /Marketing/ });
    expect(within(chip).getByRole("img", { name: "new reply" })).toBeTruthy();
    expect(within(screen.getByRole("button", { name: /^Ops/ })).queryByRole("img")).toBeNull();
  });

  test("the phone's back gesture closes the chat instead of leaving the app", async () => {
    const phone = withList(line());
    await openChat(phone, []);
    expect(history.state).toMatchObject({ botloft: true });
    act(() => {
      window.dispatchEvent(new PopStateEvent("popstate"));
    });
    await waitFor(() => expect(phone.getState().convo).toBeNull());
    expect(phone.asked.at(-1)).toEqual({ t: "watch" });
    expect(screen.getByRole("tab", { name: /Chats/ })).toBeTruthy();
  });

  test("the page's back button takes the step of history back too", async () => {
    const phone = withList(line());
    const back = vi.spyOn(history, "back");
    await openChat(phone, []);
    fireEvent.click(screen.getByRole("button", { name: "Chats" }));
    await waitFor(() => expect(phone.getState().convo).toBeNull());
    expect(back).toHaveBeenCalled();
    back.mockRestore();
  });
});

describe("what the owner read on the computer", () => {
  test("clears the dot, whether the phone was on or hears of it in the list", () => {
    const phone = withList(line(), line({ botId: "bot_2", name: "Writer", crew: "Ops" }));
    act(() => phone.set({ seen: {} }));
    expect(screen.getAllByRole("img", { name: "new reply" }).length).toBeGreaterThan(0);
    act(() => phone.receive({ t: "read", botId: "bot_1", upto: 100 }));
    const scout = screen.getByRole("button", { name: /Scout/ });
    expect(within(scout).queryByRole("img")).toBeNull();
    expect(within(screen.getByRole("button", { name: /Writer/ })).getByRole("img")).toBeTruthy();
    // A later list carries it too.
    act(() =>
      phone.receive({
        t: "chats",
        first: true,
        bots: [line({ botId: "bot_2", name: "Writer", crew: "Ops", readAt: 100 })],
      }),
    );
    expect(within(screen.getByRole("button", { name: /Writer/ })).queryByRole("img")).toBeNull();
  });
});

describe("following the end of a conversation", () => {
  test("goes down as it grows while the owner reads the end, and not when they scrolled up", async () => {
    let grew = () => {};
    vi.stubGlobal(
      "ResizeObserver",
      class {
        constructor(callback: ResizeObserverCallback) {
          grew = () => callback([], this as unknown as ResizeObserver);
        }
        observe() {}
        disconnect() {}
      },
    );
    const scrollTo = vi.fn();
    vi.stubGlobal("scrollTo", scrollTo);
    Object.defineProperty(document.documentElement, "scrollHeight", {
      value: 3000,
      configurable: true,
    });
    Object.defineProperty(window, "innerHeight", { value: 800, configurable: true });
    const at = (y: number) => {
      Object.defineProperty(window, "scrollY", { value: y, configurable: true });
      window.dispatchEvent(new Event("scroll"));
    };
    const phone = withList(line({ state: "busy" }));
    await openChat(phone, [you("itm_1", "Hello")]);
    expect(scrollTo).toHaveBeenCalledWith({ top: 3000 });

    // "working…" and the reply as it is written make it taller.
    scrollTo.mockClear();
    act(() => phone.receive({ t: "live", botId: "bot_1", text: "Writing" }));
    act(() => grew());
    expect(scrollTo).toHaveBeenCalledWith({ top: 3000 });

    // Scrolled up to read: left alone.
    at(0);
    scrollTo.mockClear();
    act(() => grew());
    expect(scrollTo).not.toHaveBeenCalled();

    // Back at the end: followed again.
    at(2200);
    act(() => grew());
    expect(scrollTo).toHaveBeenCalledWith({ top: 3000 });
    vi.unstubAllGlobals();
  });
});
