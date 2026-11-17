import { cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { crewOpened, renderApp } from "../../test/app";

afterEach(cleanup);

const GITHUB = "ghp_abcdefghijklmnopqrstuvwxyz0123";
// In two parts, so the push protection of a code host does not take it for a
// real token.
const SLACK = ["xox", "b-1234567890-abcdefghijklmnop"].join("");

async function openSecurity() {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const scout = fake.addBot(ops.id, "Scout");
  const writer = fake.addBot(ops.id, "Writer");
  fake.keys.pasted.push(
    {
      id: "hit_github",
      value: GITHUB,
      preview: "ghp_…0123",
      botIds: [writer.id, scout.id],
      places: 3,
      lastAt: fake.now,
    },
    {
      id: "hit_slack",
      value: SLACK,
      preview: "xoxb-…mnop",
      botIds: [scout.id],
      places: 1,
      lastAt: fake.now,
    },
  );
  renderApp(fake);
  await crewOpened("Ops");
  fireEvent.click(await screen.findByRole("button", { name: /Ana Lima/ }));
  fireEvent.click(screen.getByRole("menuitem", { name: "Settings" }));
  const page = screen.getByRole("region", { name: "Settings" });
  fireEvent.click(within(page).getByRole("tab", { name: "Security" }));
  return { fake, page, scout, writer };
}

describe("keys pasted in the conversations", () => {
  test("shows only how each starts and ends, and keeps one for an agent", async () => {
    const { fake, page, writer } = await openSecurity();
    fireEvent.click(within(page).getByRole("button", { name: "Look for keys" }));
    const list = await within(page).findByRole("list", { name: "2 keys found" });
    expect(within(list).getByText("ghp_…0123")).toBeDefined();
    expect(within(list).getByText(/^Writer, Scout · in 3 places/)).toBeDefined();
    expect(page.textContent).not.toContain(GITHUB);
    expect(within(page).getByText("Change the key at the service it came from")).toBeDefined();

    const row = within(list).getByText("ghp_…0123").closest("li") as HTMLElement;
    fireEvent.click(within(row).getByRole("button", { name: "Keep as a key" }));
    expect((within(row).getByLabelText("Agent") as HTMLSelectElement).value).toBe(writer.id);
    fireEvent.change(within(row).getByLabelText("Name"), { target: { value: "github token" } });
    fireEvent.click(within(row).getByRole("button", { name: "Save" }));
    await waitFor(() =>
      expect(fake.keys.saved.get(writer.id)?.get("GITHUB_TOKEN")?.value).toBe(GITHUB),
    );
    expect(await within(page).findByRole("list", { name: "1 key found" })).toBeDefined();
  });

  test("deletes one from Botloft after asking", async () => {
    const { fake, page } = await openSecurity();
    fireEvent.click(within(page).getByRole("button", { name: "Look for keys" }));
    const list = await within(page).findByRole("list", { name: "2 keys found" });
    const row = within(list).getByText("xoxb-…mnop").closest("li") as HTMLElement;
    fireEvent.click(within(row).getByRole("button", { name: "Delete from Botloft" }));
    const confirm = screen.getByRole("dialog", { name: "Delete xoxb-…mnop from Botloft?" });
    fireEvent.click(within(confirm).getByRole("button", { name: "Delete from Botloft" }));
    expect(await within(page).findByRole("list", { name: "1 key found" })).toBeDefined();
    expect(fake.keys.saved.size).toBe(0);
    const scrubbed = fake.calls.find((call) => call.method === "keys.scrub")?.params;
    expect(scrubbed).toEqual({ hits: [{ id: "hit_slack", keep: null }] });
  });

  test("says when there is nothing to change", async () => {
    const { fake, page } = await openSecurity();
    fake.keys.pasted.length = 0;
    fireEvent.click(within(page).getByRole("button", { name: "Look for keys" }));
    expect(await within(page).findByText("No keys found in the conversations.")).toBeDefined();
    expect(within(page).getByRole("button", { name: "Look again" })).toBeDefined();
  });
});
