import { cleanup, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { when } from "../../lib/format";
import { crewOpened, renderApp } from "../../test/app";

afterEach(cleanup);

const card = (name: string) =>
  within(screen.getByRole("region", { name: "Ops" }))
    .getByText(name, { selector: "span" })
    .closest("button") as HTMLElement;

describe("the agents' cards on the crew's page", () => {
  test("a free agent says when it last did something, where a busy one shows its state", async () => {
    const fake = new FakeBotloft();
    const ops = fake.addCrew("Ops");
    const scout = fake.addBot(ops.id, "Scout", "Finds sources");
    const writer = fake.addBot(ops.id, "Writer", "Writes drafts");
    const reviewer = fake.addBot(ops.id, "Reviewer", "Checks facts");
    fake.setBotState(scout.id, "idle");
    fake.setBotState(writer.id, "busy");
    fake.setBotState(reviewer.id, "idle");
    const reply = fake.chat.reply(scout.id, "Found three sources.");
    renderApp(fake);
    await crewOpened("Ops");

    expect(within(card("Scout")).getByText(`Last active: ${when(reply.createdAt)}`)).toBeDefined();
    expect(within(card("Reviewer")).getByText("Nothing done yet")).toBeDefined();
    expect(within(card("Writer")).getByText("Working")).toBeDefined();
    expect(within(card("Writer")).queryByText(/Last active|Nothing done/)).toBeNull();
  });
});
