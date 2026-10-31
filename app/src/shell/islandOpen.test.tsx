import { act, cleanup, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../lib/fake";
import { FakeHost } from "../lib/fakeHost";
import { crewOpened, renderApp } from "../test/app";

afterEach(cleanup);

describe("Open in Botloft, from the island", () => {
  test("brings the window forward on the agent's chat, or just forward", async () => {
    const fake = new FakeBotloft();
    const ops = fake.addCrew("Ops");
    fake.addBot(ops.id, "Scout", "Finds sources");
    const writer = fake.addBot(ops.id, "Writer", "Writes");
    const host = new FakeHost();
    renderApp(fake, host);
    await crewOpened("Ops");

    host.hidden = true;
    act(() => host.openBot(writer.id));
    await waitFor(() =>
      expect(screen.getByRole("heading", { level: 1, name: "Writer" })).toBeDefined(),
    );
    expect(host.hidden).toBe(false);

    // An agent it does not know, or none: the window only comes forward.
    host.hidden = true;
    act(() => host.openBot("bot_gone"));
    act(() => host.openBot(null));
    expect(host.hidden).toBe(false);
    expect(screen.getByRole("heading", { level: 1, name: "Writer" })).toBeDefined();
  });
});
