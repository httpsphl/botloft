import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { StrictMode, useState } from "react";
import { afterEach, beforeEach, describe, expect, test } from "vitest";
import { Dialog } from "./Dialog";

// jsdom runs no animations; a stand-in says this one does.
const proto = HTMLElement.prototype as { getAnimations?: () => Animation[] };
beforeEach(() => {
  proto.getAnimations = () => [];
});
afterEach(() => {
  cleanup();
  delete proto.getAnimations;
  for (const copy of document.querySelectorAll(".dialog-exit")) {
    copy.remove();
  }
});

function Owner() {
  const [open, setOpen] = useState(true);
  return open ? (
    <Dialog title="Edit Scout" onClose={() => setOpen(false)}>
      <input aria-label="Name" defaultValue="Scout" />
    </Dialog>
  ) : null;
}

const settle = () => act(() => Promise.resolve());

describe("dialog motion", () => {
  test("closing leaves a still copy that fades out, then goes", async () => {
    render(<Owner />);
    fireEvent.change(screen.getByRole("textbox", { name: "Name" }), {
      target: { value: "Scout 2" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Close" }));
    await settle();
    expect(screen.queryByRole("dialog")).toBeNull();
    const copy = document.querySelector(".dialog-exit") as HTMLElement;
    expect(copy.getAttribute("aria-hidden")).toBe("true");
    expect(copy.inert).toBe(true);
    // What the owner typed stays while it fades.
    expect(copy.querySelector("input")?.value).toBe("Scout 2");
    const end = new Event("animationend");
    fireEvent(copy, end);
    expect(document.querySelector(".dialog-exit")).toBeNull();
  });

  test("being mounted again in strict mode leaves no copy", async () => {
    render(
      <StrictMode>
        <Owner />
      </StrictMode>,
    );
    await settle();
    expect(screen.getByRole("dialog")).toBeDefined();
    expect(document.querySelector(".dialog-exit")).toBeNull();
  });
});
