import { act, cleanup } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../lib/fake";
import { FakeHost } from "../lib/fakeHost";
import { crewOpened, renderApp } from "../test/app";
import { prefs, resetPrefs } from "./prefs";

// A question asked or a note left needs the owner (spec 23.10). Without the
// island, which shows them itself (31.6), a Windows notification tells.
beforeEach(() => {
  prefs.island.set(false);
});

afterEach(() => {
  cleanup();
  resetPrefs();
});

async function scoutBehind() {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const scout = fake.addBot(ops.id, "Scout", "Finds sources");
  fake.setBotState(scout.id, "idle");
  const host = new FakeHost();
  renderApp(fake, host);
  await crewOpened("Ops");
  host.front = false;
  return { fake, host, scout };
}

describe("questions and notes", () => {
  test("tell who asks or left a note, with its first words in plain text", async () => {
    const { fake, host, scout } = await scoutBehind();
    act(() => {
      fake.questions.ask(scout.id, "Which **client** first?", ["Acme", "Globex"]);
    });
    expect(host.notices).toEqual([
      { title: "Scout has a question for you", body: "Which client first?", sound: true },
    ]);
    act(() => {
      fake.questions.tell(scout.id, `The report is ready.\n\n${"More words. ".repeat(20)}`);
    });
    const last = host.notices.at(-1);
    expect(last?.title).toBe("Scout left you a note");
    expect(last?.body.startsWith("The report is ready. More words.")).toBe(true);
    expect(last?.body.length).toBe(140);
    expect(last?.body.endsWith("…")).toBe(true);
  });

  test("a note closed or seen does not ring, and the owner may turn them off", async () => {
    const { fake, host, scout } = await scoutBehind();
    act(() => {
      fake.questions.tell(scout.id, "Backups ran fine.");
    });
    expect(host.notices).toHaveLength(1);
    act(() => fake.questions.closeNotes(scout.id));
    expect(host.notices).toHaveLength(1);

    prefs.notifyNeeds.set(false);
    act(() => {
      fake.questions.tell(scout.id, "The site is down.");
    });
    expect(host.notices).toHaveLength(1);
  });
});
