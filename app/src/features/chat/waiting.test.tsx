import { act, cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { crewOpened, openBot, renderApp } from "../../test/app";

/** Where the waiting cards are, against a chat 500 px tall. */
let cardTop = 0;

beforeEach(() => {
  cardTop = 0;
  vi.spyOn(Element.prototype, "getBoundingClientRect").mockImplementation(function (this: Element) {
    const [top, height] = this.matches("[data-item]") ? [cardTop, 200] : [0, 500];
    return new DOMRect(0, top, 600, height);
  });
  Element.prototype.scrollIntoView = vi.fn();
});

afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
});

async function openScout() {
  const fake = new FakeBotloft();
  const ops = fake.addCrew("Ops");
  const scout = fake.addBot(ops.id, "Scout", "Finds sources");
  fake.setBotState(scout.id, "idle");
  renderApp(fake);
  await crewOpened("Ops");
  openBot("Scout");
  await screen.findByRole("list", { name: "Messages" });
  return { fake, scout };
}

const pill = () => screen.queryByRole("button", { name: /waiting for you/ });

function scrolled() {
  const scroller = screen.getByRole("list", { name: "Messages" }).closest(".overflow-y-auto");
  fireEvent.scroll(scroller as Element);
}

describe("requests waiting for the owner", () => {
  test("a request out of view shows a pill that brings it back", async () => {
    const { fake, scout } = await openScout();
    // Newer messages pushed the request up, out of the chat.
    cardTop = -400;
    act(() => {
      fake.chat.ask(scout.id, "Bash", "npm test", '{"command":"npm test"}');
    });
    await waitFor(() => expect(pill()?.textContent).toBe("1 request waiting for you"));
    fireEvent.click(pill() as HTMLElement);
    const card = screen.getByRole("region", { name: "Scout asks to run a command" });
    const marked = card.closest("[data-item]") as HTMLElement;
    expect(marked.scrollIntoView).toHaveBeenCalledWith({ block: "center" });
    expect(marked.classList.contains("search-focus")).toBe(true);

    // In view, it goes; answered, it does not come back.
    cardTop = 100;
    scrolled();
    await waitFor(() => expect(pill()).toBeNull());
    cardTop = -400;
    fireEvent.click(within(card).getByRole("button", { name: "Allow" }));
    await screen.findByText("Allowed: run a command");
    scrolled();
    await new Promise((done) => setTimeout(done, 50));
    expect(pill()).toBeNull();
  });

  test("a request in view, or one only partly hidden, shows no pill", async () => {
    const { fake, scout } = await openScout();
    // 60 px of it still show at the top of the chat.
    cardTop = -140;
    act(() => {
      fake.chat.ask(scout.id, "Write", "notes.md");
      fake.chat.ask(scout.id, "Write", "todo.md");
    });
    await new Promise((done) => setTimeout(done, 50));
    expect(pill()).toBeNull();
    cardTop = 520;
    scrolled();
    await waitFor(() => expect(pill()?.textContent).toBe("2 requests waiting for you"));
  });
});
