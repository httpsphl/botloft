import { act, cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { crewOpened, openBot, renderApp } from "../../test/app";

afterEach(cleanup);

const panel = () => screen.getByRole("complementary", { name: "Scout's browser" });

describe("the owner opens the bot's closed browser", () => {
  test("on its last page, in their hands, without telling the bot", async () => {
    const fake = new FakeBotloft();
    const ops = fake.addCrew("Ops");
    const scout = fake.addBot(ops.id, "Scout", "Finds sources");
    fake.setBotState(scout.id, "idle");
    fake.browser.open(scout.id, "https://example.com/", "Example");
    renderApp(fake);
    await crewOpened("Ops");
    openBot("Scout");
    await screen.findByRole("list", { name: "Messages" });
    fireEvent.click(screen.getByRole("button", { name: /Show browser|Hide browser/ }));
    await waitFor(() => expect(fake.browser.watching).toBe(scout.id));
    act(() => {
      fake.browser.frame(scout.id, "BBBB");
    });
    act(() => {
      fake.browser.close(scout.id);
    });
    expect(await within(panel()).findByText("Browser closed")).toBeDefined();
    expect(within(panel()).getByText(/Scout is not woken or told/)).toBeDefined();

    fireEvent.click(within(panel()).getByRole("button", { name: "Open it for me" }));
    await waitFor(() => expect(fake.browser.state(scout.id).control).toBe("owner"));
    expect(fake.browser.state(scout.id).url).toBe("https://example.com/");
    expect(fake.calls.some((call) => call.method === "messages.send")).toBe(false);

    fireEvent.click(
      await within(panel()).findByRole("button", { name: "Done, give it back to Scout" }),
    );
    await waitFor(() => expect(fake.browser.state(scout.id).control).toBe("bot"));
  });
});
