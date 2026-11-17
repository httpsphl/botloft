import { cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { ClientErrorCode, RpcError } from "../../lib/rpc";
import { crewOpened, openBot, renderApp } from "../../test/app";

afterEach(cleanup);

/** The Maker's empty chat, then the history dialog open on its list. */
async function openHistory(fake = new FakeBotloft()) {
  fake.addCrew("Ops");
  const maker = fake.maker.ensure();
  fake.setBotState(maker.id, "idle");
  renderApp(fake);
  await crewOpened("Ops");
  openBot("^The Maker");
  fireEvent.click(await screen.findByRole("button", { name: /Bring my Claude Code history/ }));
  const dialog = await screen.findByRole("dialog", { name: "Bring your Claude Code history" });
  await within(dialog).findByText("C:\\work\\blog");
  return { fake, maker, dialog };
}

describe("bringing the Claude Code history", () => {
  test("the owner chooses projects, reads the summary and sends it to the Maker", async () => {
    const { fake, maker, dialog } = await openHistory();
    // Every project starts chosen; the owner leaves one out.
    const boxes = within(dialog).getAllByRole("checkbox");
    expect(boxes.every((box) => (box as HTMLInputElement).checked)).toBe(true);
    fireEvent.click(within(dialog).getByRole("checkbox", { name: /C:\\work\\site/ }));
    expect(within(dialog).getByText(/42 sessions · 318 requests/)).toBeDefined();

    fireEvent.click(within(dialog).getByRole("button", { name: "Make the summary" }));
    expect(await within(dialog).findByText(/Summarize today's e-mails/)).toBeDefined();
    expect(within(dialog).getByText(/2 keys or passwords, 1 e-mail/)).toBeDefined();
    expect(within(dialog).getByText(/The swap is not perfect/)).toBeDefined();
    // The model and effort the Maker reads it with, right before sending.
    expect(within(dialog).getByText("The Maker reads it with")).toBeDefined();
    expect(within(dialog).getByRole("button", { name: /^Model/ })).toBeDefined();
    expect(fake.imports.made[0]?.scope).toEqual(["C:\\work\\blog", "C:\\work\\clinic"]);

    fireEvent.click(within(dialog).getByRole("button", { name: "Send to the Maker" }));
    await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
    const sent = fake.imports.made[0];
    expect(sent?.sentAt).not.toBeNull();
    const message = fake.calls.find((call) => call.method === "imports.send");
    expect(message).toBeDefined();
    const toMaker = fake.conversation.messages.find((each) => each.id === sent?.sentMessageId);
    expect(toMaker?.toBotId).toBe(maker.id);
    expect(toMaker?.attachments[0]?.name).toBe("claude-code-history.md");
  });

  test("a summary the owner does not want is forgotten", async () => {
    const { fake, dialog } = await openHistory();
    fireEvent.click(within(dialog).getByRole("button", { name: "Make the summary" }));
    await within(dialog).findByText(/Summarize today's e-mails/);
    expect(fake.imports.made).toHaveLength(1);
    fireEvent.click(within(dialog).getByRole("button", { name: "Forget it" }));
    await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
    expect(fake.imports.made).toHaveLength(0);
    expect(fake.calls.some((call) => call.method === "imports.send")).toBe(false);
  });

  test("a reading that fails says so and tries again, never 'no history'", async () => {
    const fake = new FakeBotloft();
    fake.failNext("imports.scan", new RpcError(ClientErrorCode.timeout, "timed out"));
    fake.addCrew("Ops");
    const maker = fake.maker.ensure();
    fake.setBotState(maker.id, "idle");
    renderApp(fake);
    await crewOpened("Ops");
    openBot("^The Maker");
    fireEvent.click(await screen.findByRole("button", { name: /Bring my Claude Code history/ }));
    const dialog = await screen.findByRole("dialog", { name: "Bring your Claude Code history" });
    expect(await within(dialog).findByText(/Could not read the history\. It may/)).toBeDefined();
    expect(within(dialog).queryByText(/no history on this computer/)).toBeNull();
    fireEvent.click(within(dialog).getByRole("button", { name: "Try again" }));
    expect(await within(dialog).findByText("C:\\work\\blog")).toBeDefined();
  });

  test("while the summary is made, the Maker works on screen with the seconds", async () => {
    const { dialog } = await openHistory();
    fireEvent.click(within(dialog).getByRole("button", { name: "Make the summary" }));
    const working = within(dialog).getByRole("status");
    expect(within(working).getByText("Making the summary…")).toBeDefined();
    expect(within(working).getByText("0 s")).toBeDefined();
    expect(working.querySelector("[data-maker]")).not.toBeNull();
    await within(dialog).findByText(/Summarize today's e-mails/);
  });
});
