import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { useState } from "react";
import { afterEach, describe, expect, test, vi } from "vitest";
import { Dialog } from "./Dialog";
import { Select } from "./Select";

afterEach(cleanup);

const LANGUAGES = [
  { value: "system", label: "System language: English" },
  { value: "en", label: "English" },
  { value: "pt-BR", label: "Português (Brasil)" },
  { value: "es", label: "Español" },
] as const;

type Language = (typeof LANGUAGES)[number]["value"];

function Picker({ onClose }: { onClose(): void }) {
  const [value, setValue] = useState<Language>("system");
  return (
    <Dialog title="Settings" onClose={onClose}>
      <Select<Language> label="Language" value={value} options={LANGUAGES} onChange={setValue} />
    </Dialog>
  );
}

const trigger = () => screen.getByRole("button", { name: /^Language:/ });

describe("a dropdown", () => {
  test("shows the choice and picks another with a click", () => {
    render(<Picker onClose={() => {}} />);
    expect(trigger().getAttribute("aria-label")).toBe("Language: System language: English");
    fireEvent.click(trigger());
    expect(
      screen
        .getByRole("option", { name: "System language: English" })
        .getAttribute("aria-selected"),
    ).toBe("true");
    fireEvent.click(screen.getByRole("option", { name: "Español" }));
    expect(screen.queryByRole("listbox")).toBeNull();
    expect(trigger().getAttribute("aria-label")).toBe("Language: Español");
  });

  test("works from the keyboard, and Escape closes only the list", () => {
    const onClose = vi.fn();
    render(<Picker onClose={onClose} />);
    fireEvent.keyDown(trigger(), { key: "ArrowDown" });
    const list = screen.getByRole("listbox", { name: "Language" });
    expect(document.activeElement?.textContent).toBe("System language: English");
    fireEvent.keyDown(list, { key: "ArrowDown" });
    fireEvent.keyDown(list, { key: "ArrowDown" });
    expect(document.activeElement?.textContent).toBe("Português (Brasil)");
    fireEvent.keyDown(document.activeElement as Element, { key: "Enter" });
    expect(trigger().getAttribute("aria-label")).toBe("Language: Português (Brasil)");
    expect(document.activeElement).toBe(trigger());

    fireEvent.click(trigger());
    fireEvent.keyDown(document.activeElement as Element, { key: "Escape" });
    expect(screen.queryByRole("listbox")).toBeNull();
    expect(onClose).not.toHaveBeenCalled();
    expect(screen.getByRole("dialog", { name: "Settings" })).toBeDefined();
  });
});
