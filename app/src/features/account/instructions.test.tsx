import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { App } from "../../App";
import type { Client } from "../../lib/client";
import { FakeBotloft } from "../../lib/fake";
import { FakeHost } from "../../lib/fakeHost";
import { FIELD_LIMITS } from "../../lib/protocol.gen";
import { resetPrefs } from "../../shell/prefs";

afterEach(() => {
  cleanup();
  resetPrefs();
});

async function openInstructions(fake: FakeBotloft) {
  render(<App host={new FakeHost()} connect={() => fake as Client} />);
  fireEvent.click(await screen.findByRole("button", { name: /Ana Lima/ }));
  fireEvent.click(screen.getByRole("menuitem", { name: "Settings" }));
  const page = screen.getByRole("region", { name: "Settings" });
  fireEvent.click(within(page).getByRole("tab", { name: "Instructions for all" }));
  return page;
}

describe("instructions for every agent", () => {
  test("show what the agents have and save a change with the button", async () => {
    const fake = new FakeBotloft();
    fake.ownerInstructions = "Answer me in English.";
    const page = await openInstructions(fake);
    const field = within(page).getByRole("textbox", { name: "What every agent follows" });
    await waitFor(() => expect((field as HTMLTextAreaElement).value).toBe("Answer me in English."));
    const save = within(page).getByRole("button", { name: "Save" });
    expect(save.hasAttribute("disabled")).toBe(true);

    fireEvent.change(field, { target: { value: "Answer me in English.\nBe brief.\n\n" } });
    expect(save.hasAttribute("disabled")).toBe(false);
    fireEvent.click(save);
    await waitFor(() => expect(fake.ownerInstructions).toBe("Answer me in English.\nBe brief."));
    expect((field as HTMLTextAreaElement).value).toBe("Answer me in English.\nBe brief.");
    expect(await screen.findByText("Saved. Available agents start again to read it.")).toBeTruthy();
    expect(save.hasAttribute("disabled")).toBe(true);
  });

  test("text over the limit cannot be saved", async () => {
    const fake = new FakeBotloft();
    const page = await openInstructions(fake);
    const field = within(page).getByRole("textbox", { name: "What every agent follows" });
    await waitFor(() => expect(field.hasAttribute("disabled")).toBe(false));
    fireEvent.change(field, { target: { value: "x".repeat(FIELD_LIMITS.instructions + 1) } });
    expect(within(page).getByRole("button", { name: "Save" }).hasAttribute("disabled")).toBe(true);
    expect(fake.ownerInstructions).toBe("");
  });

  test("Settings search finds the page", async () => {
    const fake = new FakeBotloft();
    const page = await openInstructions(fake);
    fireEvent.change(within(page).getByRole("searchbox"), { target: { value: "every agent" } });
    fireEvent.click(within(page).getByRole("button", { name: /What every agent follows/ }));
    expect(
      within(page).getByRole("tab", { name: "Instructions for all" }).getAttribute("aria-selected"),
    ).toBe("true");
  });
});
