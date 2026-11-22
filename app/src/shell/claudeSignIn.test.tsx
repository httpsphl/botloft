import { cleanup, screen } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../lib/fake";
import { FakeHost } from "../lib/fakeHost";
import { crewOpened, renderApp } from "../test/app";
import { endsSoon } from "./ClaudeSignInNotice";

// The strip about Claude's sign-in for the agents (spec 42.9).

afterEach(cleanup);

const DAY = 24 * 60 * 60 * 1000;

function withCrew(change: (fake: FakeBotloft) => void) {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  fake.addBot(ops.id, "Scout");
  change(fake);
  renderApp(fake, new FakeHost());
  return crewOpened("Ops");
}

describe("Claude's sign-in for the agents", () => {
  test("ending within days, this computer offers to sign in again", async () => {
    await withCrew((fake) => {
      fake.system = {
        ...fake.system,
        claudeSignInEnds: { from: Date.now() + 3 * DAY, by: Date.now() + 6 * DAY },
      };
    });
    expect(await screen.findByText("Claude's sign-in for your agents ends soon")).toBeTruthy();
    expect(screen.getByRole("button", { name: "Sign in to Claude" })).toBeTruthy();
  });

  test("on a server, the strip says what to run in its terminal", async () => {
    await withCrew((fake) => {
      fake.system = {
        ...fake.system,
        server: true,
        claudeSignInEnds: { from: Date.now() + DAY, by: Date.now() + DAY },
      };
    });
    expect(await screen.findByText(/It may end any day now/)).toBeTruthy();
    expect(screen.getByText("sudo -u botloft-agents -i claude auth login")).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Sign in to Claude" })).toBeNull();
  });

  test("signed out on a server, the agents' stop is said with the command", async () => {
    await withCrew((fake) => {
      fake.system = { ...fake.system, server: true, claudeSignedIn: false };
    });
    expect(await screen.findByText(/Your agents stopped/)).toBeTruthy();
    expect(screen.getByText("sudo -u botloft-agents -i claude auth login")).toBeTruthy();
  });

  test("an end weeks away shows nothing yet", () => {
    const now = Date.now();
    expect(endsSoon({ from: now + 30 * DAY, by: now + 33 * DAY }, now)).toBe(false);
    expect(endsSoon({ from: now + 4 * DAY, by: now + 7 * DAY }, now)).toBe(true);
    expect(endsSoon(undefined, now)).toBe(false);
  });
});
