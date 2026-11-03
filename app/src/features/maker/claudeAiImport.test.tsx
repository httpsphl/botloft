import { cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { FakeHost } from "../../lib/fakeHost";
import { crewOpened, openBot, renderApp } from "../../test/app";

afterEach(cleanup);

/** The Maker's empty chat, then the claude.ai dialog open on its three steps. */
async function openExport() {
  const fake = new FakeBotloft();
  fake.addCrew("Ops");
  const maker = fake.maker.ensure();
  fake.setBotState(maker.id, "idle");
  const host = new FakeHost();
  renderApp(fake, host);
  await crewOpened("Ops");
  openBot("^The Maker");
  fireEvent.click(await screen.findByRole("button", { name: /Bring my claude.ai export/ }));
  const dialog = await screen.findByRole("dialog", { name: "Bring your claude.ai export" });
  return { fake, maker, host, dialog };
}

describe("bringing the claude.ai export", () => {
  test("the owner gets the ZIP, chooses it and sends its summary to the Maker", async () => {
    const { fake, maker, host, dialog } = await openExport();
    expect(within(dialog).getByText(/open Settings > Privacy and click Export data/)).toBeDefined();
    fireEvent.click(within(dialog).getByRole("button", { name: "Open claude.ai" }));
    expect(host.opened).toContain("https://claude.ai/settings");

    host.nextFile = "C:\\Users\\me\\Downloads\\export.zip";
    fireEvent.click(within(dialog).getByRole("button", { name: "Choose the ZIP" }));
    expect(await within(dialog).findByText(/128 conversations/)).toBeDefined();
    expect(host.fileFilters.at(-1)).toEqual(["zip"]);
    expect(within(dialog).getByText("Projects: Clinic, Newsletter")).toBeDefined();
    fireEvent.click(within(dialog).getByRole("radio", { name: "Last 6 months" }));

    fireEvent.click(within(dialog).getByRole("button", { name: "Make the summary" }));
    expect(await within(dialog).findByText(/Draft the clinic's weekly schedule/)).toBeDefined();
    const create = fake.calls.find((call) => call.method === "imports.create")?.params as {
      source: string;
      path: string;
      from?: number;
    };
    expect(create.source).toBe("claude_ai");
    expect(create.path).toBe("C:\\Users\\me\\Downloads\\export.zip");
    expect(create.from).toBeGreaterThan(0);

    fireEvent.click(within(dialog).getByRole("button", { name: "Send to the Maker" }));
    await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
    const sent = fake.imports.made[0];
    const toMaker = fake.conversation.messages.find((each) => each.id === sent?.sentMessageId);
    expect(toMaker?.toBotId).toBe(maker.id);
    expect(toMaker?.attachments[0]?.name).toBe("claude-ai-history.md");
  });

  test("a ZIP that is not an export says so and nothing is made", async () => {
    const { fake, host, dialog } = await openExport();
    host.nextFile = "C:\\Users\\me\\Downloads\\photos.zip";
    fireEvent.click(within(dialog).getByRole("button", { name: "Choose the ZIP" }));
    expect(await screen.findByText(/Could not read this file/)).toBeDefined();
    expect(within(dialog).getByRole("button", { name: "Make the summary" })).toHaveProperty(
      "disabled",
      true,
    );
    expect(fake.imports.made).toHaveLength(0);
  });
});
