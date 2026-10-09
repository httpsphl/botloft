// Rendering the whole app against FakeBotloft and FakeHost for tests.

import { fireEvent, render, screen, within } from "@testing-library/react";
import { App } from "../App";
import type { Client } from "../lib/client";
import { FakeBotloft } from "../lib/fake";
import { FakeHost } from "../lib/fakeHost";

export function renderApp(fake = new FakeBotloft(), host = new FakeHost()) {
  render(<App host={host} connect={() => fake as Client} />);
  return { fake, host };
}

/** The crews sidebar. */
export const sidebar = () => screen.getByRole("navigation", { name: "Crews" });
/** The icons at the far left (shell/Rail.tsx). */
export const rail = () => screen.getByRole("navigation", { name: "Places" });

/** Waits for the first crew to open. */
export async function crewOpened(name: string) {
  return screen.findByRole("heading", { level: 1, name });
}

export function openBot(name: string) {
  fireEvent.click(within(sidebar()).getByRole("button", { name: new RegExp(name) }));
}

export function openTab(name: string | RegExp) {
  fireEvent.click(screen.getByRole("tab", { name }));
}

/**
 * Opens "New crew" and takes the empty crew in the template step (spec 29.1),
 * so the form is there. Returns the dialog.
 */
export async function openNewCrewForm() {
  fireEvent.click(screen.getByRole("button", { name: "New crew" }));
  const chooser = await screen.findByRole("dialog", { name: "Start from a template" });
  fireEvent.click(within(chooser).getByRole("button", { name: /Start with an empty crew/ }));
  return screen.getByRole("dialog", { name: "New crew" });
}
