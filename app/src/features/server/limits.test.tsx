import { cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { FakeHost } from "../../lib/fakeHost";
import { server } from "../../shell/server";
import { crewOpened, renderApp } from "../../test/app";
import { LIMITS_COMMAND } from "./LimitsSettings";

const GB = 1024 ** 3;

afterEach(() => {
  cleanup();
  server.clear();
});

async function openGeneral(fake: FakeBotloft) {
  const ops = fake.addCrew("Ops");
  fake.addBot(ops.id, "Scout");
  renderApp(fake, new FakeHost());
  await crewOpened("Ops");
  fireEvent.click(await screen.findByRole("button", { name: /Ana Lima/ }));
  fireEvent.click(screen.getByRole("menuitem", { name: "Settings" }));
  const dialog = screen.getByRole("region", { name: "Settings" });
  fireEvent.click(within(dialog).getByRole("tab", { name: "General" }));
  return dialog;
}

function onServer(): FakeBotloft {
  const fake = new FakeBotloft();
  fake.system = { ...fake.system, server: true };
  return fake;
}

describe("this server's limits, in Settings", () => {
  test("a computer that is not a server shows none", async () => {
    const dialog = await openGeneral(new FakeBotloft());
    expect(within(dialog).queryByText("This server's limits")).toBeNull();
  });

  test("the memory and the processors show with the command that changes them", async () => {
    const fake = onServer();
    fake.limits = {
      memoryTotal: 16 * GB,
      memoryLimit: 4 * GB,
      memoryUsed: 1 * GB,
      cpus: 4,
      cpuLimitPercent: 150,
      agentsAwake: 2,
      agentsMax: 0,
      agentsFit: 9,
      changeable: true,
    };
    const dialog = await openGeneral(fake);
    expect(await within(dialog).findByText("This server's limits")).toBeTruthy();
    expect(
      await within(dialog).findByText("1.0 GB in use of 4.0 GB (the server has 16.0 GB)"),
    ).toBeTruthy();
    expect(within(dialog).getByText("Up to 1.5 of 4")).toBeTruthy();
    expect(within(dialog).getByText(/2 awake now/)).toBeTruthy();
    expect(within(dialog).getByText(LIMITS_COMMAND)).toBeTruthy();
  });

  test("in a container, the limits are the container's", async () => {
    const fake = onServer();
    fake.limits = { memoryTotal: 8 * GB, cpus: 2, agentsAwake: 0, agentsMax: 0, changeable: false };
    const dialog = await openGeneral(fake);
    expect(await within(dialog).findByText("No limit (the server has 8.0 GB)")).toBeTruthy();
    expect(within(dialog).getByText("No limit (the server has 2)")).toBeTruthy();
    expect(within(dialog).getByText(/runs in a container/)).toBeTruthy();
    expect(within(dialog).queryByText(LIMITS_COMMAND)).toBeNull();
  });

  test("the agents awake at once are chosen here and saved", async () => {
    const fake = onServer();
    const dialog = await openGeneral(fake);
    await within(dialog).findByText("This server's limits");
    fireEvent.click(within(dialog).getByRole("button", { name: /No limit/ }));
    fireEvent.click(screen.getByRole("option", { name: "6" }));
    await waitFor(() => expect(fake.settings.maxRunning).toBe(6));
    expect(
      fake.calls.filter((call) => call.method === "settings.update").map((call) => call.params),
    ).toEqual([{ maxRunning: 6 }]);
  });
});
