import { cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { en } from "../../i18n/en";
import { FakeBotloft } from "../../lib/fake";
import { crewOpened, openTab, renderApp } from "../../test/app";

afterEach(cleanup);

const calls = (fake: FakeBotloft, method: string) =>
  fake.calls.filter((call) => call.method === method);

/** A crew "Site" with its chief, open on the crew's routines. */
async function openCrewRoutines(withChief = true) {
  const fake = new FakeBotloft();
  if (withChief) {
    await fake.call("crews.create", {
      name: "Site",
      lead: { name: "Chief", role: "Leads the crew", instructions: "Build it" },
    });
  } else {
    fake.addCrew("Site");
  }
  renderApp(fake);
  await crewOpened("Site");
  openTab(/^Routines/);
  return { fake };
}

describe("the daily summary", () => {
  test("is offered on the crew's routines and turned on for the chief, every day", async () => {
    const { fake } = await openCrewRoutines();
    const card = await screen.findByRole("region", { name: "Daily summary" });
    fireEvent.change(within(card).getByLabelText("Time"), { target: { value: "19:30" } });
    fireEvent.click(within(card).getByRole("button", { name: "Turn on" }));

    await waitFor(() => expect(calls(fake, "routines.create")).toHaveLength(1));
    const [create] = calls(fake, "routines.create");
    expect(create?.params).toMatchObject({
      name: "Daily summary",
      schedule: { kind: "weekly", days: [1, 2, 3, 4, 5, 6, 7], time: "19:30" },
      overlap: "skip",
      missed: "run_once",
    });
    // It asks the chief to use the tool that tells it what the bots did.
    const params = create?.params as { prompt: string; botId: string } | undefined;
    expect(params?.prompt).toContain("crew_activity");
    const chief = (await fake.call("bots.list", {})).find((bot) => bot.name === "Chief");
    expect(params?.botId).toBe(chief?.id);

    expect(await within(card).findByText(/^On · /)).toBeDefined();
    expect(within(card).getByRole("button", { name: "Turn off" })).toBeDefined();
  });

  test("can be turned off and on again, and shows in the list like any routine", async () => {
    const { fake } = await openCrewRoutines();
    const card = await screen.findByRole("region", { name: "Daily summary" });
    fireEvent.click(within(card).getByRole("button", { name: "Turn on" }));
    await within(card).findByText(/^On · /);
    expect(
      within(await screen.findByRole("list", { name: "Routines" })).getByText("Daily summary"),
    ).toBeDefined();

    fireEvent.click(within(card).getByRole("button", { name: "Turn off" }));
    expect(await within(card).findByText("Off")).toBeDefined();
    fireEvent.click(within(card).getByRole("button", { name: "Turn on" }));
    await within(card).findByText(/^On · /);
    expect(calls(fake, "routines.setEnabled").map((call) => call.params)).toEqual([
      expect.objectContaining({ enabled: false }),
      expect.objectContaining({ enabled: true }),
    ]);
    // One routine, not a second one each time.
    expect(calls(fake, "routines.create")).toHaveLength(1);
  });

  test("is not offered to a crew without a chief", async () => {
    await openCrewRoutines(false);
    await screen.findByText(en.routines.crewEmpty);
    expect(screen.queryByRole("region", { name: "Daily summary" })).toBeNull();
  });
});
