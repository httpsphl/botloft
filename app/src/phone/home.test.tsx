// The phone's Home (spec 28.15): where the page opens, with the shortcuts,
// what waits, who works now and the day so far.

import { act, cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import type { ChatLine, DayActivity, FromPhone } from "../lib/protocol.gen";
import { FakePhone } from "./fake";
import { midnight } from "./home";
import { PhoneApp } from "./PhoneApp";

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

const day: DayActivity = {
  tasksDone: 3,
  tasksFailed: 1,
  botMessages: 2,
  requestsAnswered: 4,
  filesEdited: 5,
  lines: { added: 10, removed: 2 },
  busiest: { botId: "bot_1", name: "Scout", color: "#aabbcc", crew: "Ops", workMs: 3_600_000 },
};

const shortcut = { routineId: "rtn_1", name: "Morning summary", botId: "bot_1" };

function home() {
  const phone = new FakePhone();
  phone.receive({
    t: "chats",
    first: true,
    bots: [
      line({
        state: "busy",
        last: { kind: "tool", text: "Writing the report", tool: null, at: 90 },
      }),
      line({ botId: "bot_2", name: "Quill", crew: "Ops" }),
    ],
  });
  phone.receive({ t: "home", shortcuts: [shortcut], today: day });
  render(<PhoneApp api={phone} />);
  return phone;
}

const runs = (phone: FakePhone) =>
  phone.asked.filter(
    (ask): ask is Extract<FromPhone, { t: "routine.run" }> => ask.t === "routine.run",
  );

describe("the Home", () => {
  test("opens first, with the shortcuts, what waits, who works and the day", () => {
    home();
    expect(screen.getByRole("tab", { name: "Home" }).getAttribute("aria-selected")).toBe("true");
    expect(screen.getByRole("heading", { name: "Shortcuts" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Run Morning summary, by Scout" })).toBeTruthy();
    expect(screen.getByRole("heading", { name: "Needs you" })).toBeTruthy();
    expect(screen.getByText("Nothing is waiting for you")).toBeTruthy();

    const working = screen.getByRole("region", { name: "Working now" });
    expect(within(working).getByText("Scout")).toBeTruthy();
    expect(within(working).getByText("Writing the report")).toBeTruthy();
    expect(within(working).queryByText("Quill")).toBeNull();

    const today = screen.getByRole("region", { name: "Today" });
    expect(within(today).getByText("Tasks done").nextSibling?.textContent).toContain("3");
    expect(within(today).getByText("1 failed")).toBeTruthy();
    expect(within(today).getByText("Requests answered").nextSibling?.textContent).toBe("4");
    expect(within(today).getByText("Worked the most").nextSibling?.textContent).toContain("Scout");
  });

  test("a tap runs a shortcut, and the word under it says how that went", async () => {
    const phone = home();
    const button = screen.getByRole("button", { name: "Run Morning summary, by Scout" });
    fireEvent.click(button);
    await waitFor(() => expect(runs(phone)).toHaveLength(1));
    expect(runs(phone)[0]).toMatchObject({ routineId: "rtn_1" });
    expect(screen.getByText("Starting…")).toBeTruthy();
    expect((button as HTMLButtonElement).disabled).toBe(true);

    const asksBefore = phone.asked.filter((ask) => ask.t === "home").length;
    act(() => phone.receive({ t: "ran", clientId: runs(phone)[0]?.clientId ?? "", ok: true }));
    expect(screen.getByText("Scout started: Morning summary")).toBeTruthy();
    // The day changes with it: the Home is asked for again.
    expect(phone.asked.filter((ask) => ask.t === "home").length).toBe(asksBefore + 1);

    fireEvent.click(button);
    await waitFor(() => expect(runs(phone)).toHaveLength(2));
    act(() =>
      phone.receive({
        t: "ran",
        clientId: runs(phone)[1]?.clientId ?? "",
        ok: false,
        reason: "not_found",
      }),
    );
    expect(screen.getByText("Could not run it")).toBeTruthy();
  });

  test("asks again for the day a moment after an agent starts or stops", async () => {
    const phone = new FakePhone();
    phone.receive({ t: "chats", first: true, bots: [line()] });
    render(<PhoneApp api={phone} />);
    const asks = () => phone.asked.filter((ask) => ask.t === "home");
    await waitFor(() => expect(asks()).toHaveLength(1), { timeout: 3000 });
    expect(asks()[0]).toEqual({ t: "home", since: midnight() });

    act(() => phone.receive({ t: "line", bot: line({ state: "busy" }) }));
    await waitFor(() => expect(asks()).toHaveLength(2), { timeout: 3000 });
  });

  test("without shortcuts, workers or a day, shows only what waits", () => {
    const phone = new FakePhone();
    phone.receive({ t: "home", shortcuts: [] });
    render(<PhoneApp api={phone} />);
    expect(screen.queryByRole("heading", { name: "Shortcuts" })).toBeNull();
    expect(screen.queryByRole("heading", { name: "Working now" })).toBeNull();
    expect(screen.queryByRole("heading", { name: "Today" })).toBeNull();
    expect(screen.getByRole("heading", { name: "Needs you" })).toBeTruthy();
  });
});
