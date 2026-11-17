import { cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { crewOpened, openBot, renderApp } from "../../test/app";

afterEach(cleanup);

const HOUR = 60 * 60 * 1000;

/** Scout, whose memory changed three times, open on its details. */
async function scoutDetails(seed?: (fake: FakeBotloft, scout: string) => void) {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const scout = fake.addBot(ops.id, "Scout", "Finds sources");
  seed?.(fake, scout.id);
  renderApp(fake);
  await crewOpened("Ops");
  openBot("Scout");
  fireEvent.click(await screen.findByRole("button", { name: "Show details" }));
  const details = await screen.findByRole("complementary", { name: "About Scout" });
  return { fake, scout, details };
}

function threeMemories(fake: FakeBotloft, scout: string) {
  const at = fake.now - 3 * HOUR;
  fake.versions.add(scout, "memory", "start", "Client: Acme.\n", at);
  fake.versions.add(scout, "memory", "agent", "Client: Acme.\nShort reports.\n", at + HOUR);
  fake.versions.add(
    scout,
    "memory",
    "outside",
    "Client: Acme Ltd.\nShort reports.\n",
    at + 2 * HOUR,
  );
}

describe("the history of an agent's memory and instructions (spec 8.10)", () => {
  test("lists each version newest first, with who made it", async () => {
    const { details } = await scoutDetails(threeMemories);
    fireEvent.click(within(details).getByRole("button", { name: "Memory history" }));
    const dialog = await screen.findByRole("dialog", { name: "Scout's memory history" });
    const list = await within(dialog).findByRole("list", { name: "Versions" });
    const rows = within(list).getAllByRole("listitem");
    expect(rows.map((row) => row.textContent)).toEqual([
      expect.stringMatching(/^Changed outside Botloft.*Now$/),
      expect.stringContaining("Scout changed it"),
      expect.stringContaining("How it was at first"),
    ]);
    // A version opens to what it changed from the one before.
    fireEvent.click(within(rows[1] as HTMLElement).getByRole("button", { expanded: false }));
    expect(within(rows[1] as HTMLElement).getByText("Short reports.")).toBeDefined();
  });

  test("an older version comes back as a new one", async () => {
    const { fake, details } = await scoutDetails(threeMemories);
    fireEvent.click(within(details).getByRole("button", { name: "Memory history" }));
    const dialog = await screen.findByRole("dialog", { name: "Scout's memory history" });
    const rows = within(await within(dialog).findByRole("list")).getAllByRole("listitem");
    const agent = rows[1] as HTMLElement;
    fireEvent.click(within(agent).getByRole("button", { expanded: false }));
    fireEvent.click(within(agent).getByRole("button", { name: "Go back to this version" }));
    const confirm = screen.getByRole("dialog", { name: "Go back to this version?" });
    expect(confirm.textContent).toContain("Scout's memory goes back to how it was on");
    fireEvent.click(within(confirm).getByRole("button", { name: "Go back" }));
    await waitFor(() =>
      expect(fake.calls.filter((call) => call.method === "versions.restore")).toHaveLength(1),
    );
    const newest = await within(dialog).findByText(/^You went back to the version of/);
    expect(newest).toBeDefined();
    expect(within(dialog).getAllByRole("listitem")).toHaveLength(4);
  });

  test("instructions have their own history, empty until they change", async () => {
    const { details } = await scoutDetails();
    fireEvent.click(within(details).getByRole("button", { name: "Instructions history" }));
    const dialog = await screen.findByRole("dialog", { name: "Scout's instructions history" });
    expect(
      await within(dialog).findByText(
        "No changes yet. The history starts the first time the instructions change.",
      ),
    ).toBeDefined();
  });
});
