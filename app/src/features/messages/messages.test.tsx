import { act, cleanup, fireEvent, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, test, vi } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { crewOpened, openTab, renderApp } from "../../test/app";

afterEach(cleanup);

/** A crew "Ops" with @lead and @writer. */
function crew() {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const lead = fake.addBot(ops.id, "Lead");
  const writer = fake.addBot(ops.id, "Writer");
  return { fake, ops, lead, writer };
}

const messages = () => screen.getByRole("list", { name: "Messages" });

describe("messages", () => {
  test("the crew timeline shows what bots send and what the owner writes", async () => {
    const { fake, lead, writer } = crew();
    const { delivery } = fake.conversation.say({
      from: lead.id,
      to: writer.id,
      body: "Draft the notes",
    });
    fake.conversation.deliver(delivery.id, "sent");
    renderApp(fake);
    await crewOpened("Ops");
    openTab("Timeline");
    const first = await within(messages()).findByText("Draft the notes");
    const row = first.closest("li") as HTMLElement;
    expect(within(row).getByText("Delivered")).toBeDefined();

    fireEvent.change(screen.getByLabelText("Recipient"), { target: { value: writer.id } });
    fireEvent.change(screen.getByLabelText("Message to"), { target: { value: "Ship it today" } });
    fireEvent.click(screen.getByRole("button", { name: "Send" }));
    const sent = await within(messages()).findByText("Ship it today");
    const sentRow = sent.closest("li") as HTMLElement;
    expect(within(sentRow).getByText("You")).toBeDefined();
    expect(within(sentRow).getByText("Waiting for the bot")).toBeDefined();
    expect(fake.calls.at(-1)).toEqual({
      method: "messages.send",
      params: { botId: writer.id, body: "Ship it today" },
    });
    const owner = [...fake.conversation.deliveries.values()].at(-1);
    act(() => {
      fake.conversation.deliver(owner?.id ?? "", "sent");
    });
    expect(within(sentRow).getByText("Delivered")).toBeDefined();
  });

  test("older messages load on demand", async () => {
    const { fake, lead, writer } = crew();
    for (let n = 1; n <= 60; n += 1) {
      fake.conversation.say({ from: lead.id, to: writer.id, body: `note ${n}` });
    }
    renderApp(fake);
    await crewOpened("Ops");
    openTab("Timeline");
    await within(messages()).findByText("note 60");
    expect(within(messages()).queryByText("note 10")).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "Load older messages" }));
    expect(await within(messages()).findByText("note 1")).toBeDefined();
    expect(within(messages()).getAllByRole("listitem")).toHaveLength(60);
    expect(screen.queryByRole("button", { name: "Load older messages" })).toBeNull();
    // Sixty messages drawn twice: about 2 s alone, past the default 5 s
    // when the whole suite runs at once.
  }, 15_000);

  test("a message that was not delivered can be retried from the title bar", async () => {
    const { fake, lead, writer } = crew();
    const { delivery } = fake.conversation.say({ from: lead.id, to: writer.id, body: "lost note" });
    fake.conversation.deliver(delivery.id, "dead", "the bot's inbox did not answer");
    const { host } = renderApp(fake);
    fireEvent.click(await screen.findByRole("button", { name: "1 not delivered" }));
    const dialog = screen.getByRole("dialog", { name: "Messages not delivered" });
    expect(await within(dialog).findByText("lost note")).toBeDefined();
    expect(within(dialog).getByText("the bot's inbox did not answer")).toBeDefined();
    expect(host.attention).toBe(true);
    fireEvent.click(within(dialog).getByRole("button", { name: "Retry all" }));
    await vi.waitFor(() =>
      expect(fake.conversation.deliveries.get(delivery.id)?.state).toBe("pending"),
    );
    expect(screen.queryByRole("button", { name: /not delivered/ })).toBeNull();
    expect(host.attention).toBe(false);
  });

  test("a dead delivery shows in the timeline with a retry", async () => {
    const { fake, lead, writer } = crew();
    const { delivery } = fake.conversation.say({ from: lead.id, to: writer.id, body: "stuck" });
    fake.conversation.deliver(delivery.id, "dead", "gave up after 8 tries");
    renderApp(fake);
    await crewOpened("Ops");
    openTab("Timeline");
    const row = (await within(messages()).findByText("stuck")).closest("li") as HTMLElement;
    expect(within(row).getByText("Not delivered")).toBeDefined();
    fireEvent.click(within(row).getByRole("button", { name: "Retry" }));
    expect(await within(row).findByText("Waiting for the bot")).toBeDefined();
  });

  test("a bot waiting for approval marks the taskbar icon", async () => {
    const { fake, lead } = crew();
    const { host } = renderApp(fake);
    await crewOpened("Ops");
    expect(host.attention).toBe(false);
    act(() => fake.setBotState(lead.id, "needs_approval"));
    await vi.waitFor(() => expect(host.attention).toBe(true));
    act(() => fake.setBotState(lead.id, "busy"));
    await vi.waitFor(() => expect(host.attention).toBe(false));
  });
});

describe("tasks", () => {
  test("open tasks by default, and every task with its result on demand", async () => {
    const { fake, lead, writer } = crew();
    fake.conversation.task(lead.id, writer.id, { status: "done", result: "Notes are in shared/" });
    fake.conversation.task(writer.id, lead.id, { hops: 2 });
    renderApp(fake);
    await crewOpened("Ops");
    openTab("Tasks");
    const list = screen.getByRole("list", { name: "Tasks" });
    expect(within(list).getAllByRole("listitem")).toHaveLength(1);
    expect(within(list).getByText("hop 2")).toBeDefined();
    expect(within(list).getByText(/^due in/)).toBeDefined();
    fireEvent.click(screen.getByRole("button", { name: "All" }));
    expect(
      within(screen.getByRole("list", { name: "Tasks" })).getAllByRole("listitem"),
    ).toHaveLength(2);
    expect(screen.getByText("Notes are in shared/")).toBeDefined();
    // A task that changes shows at once.
    const open = [...fake.conversation.tasks.values()].find((task) => task.status === "open");
    act(() => {
      fake.conversation.task(open?.requesterBotId ?? "", open?.assigneeBotId ?? "", {
        id: open?.id ?? "",
        status: "done",
        result: "Done: 3 pages",
      });
    });
    expect(screen.getByText("Done: 3 pages")).toBeDefined();
  });
});
