import { cleanup, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { FakeBotloft } from "../../lib/fake";
import { AVATAR_PALETTE } from "../../lib/protocol.gen";
import { crewOpened, openBot, renderApp } from "../../test/app";
import { hsvToRgb, parseHex, rgbToHsv, toHexColor } from "./colorSpace";

afterEach(cleanup);

describe("color space", () => {
  test("hex, rgb and hsv round-trip", () => {
    for (const hex of [...AVATAR_PALETTE, "#000000", "#FFFFFF", "#123ABC"]) {
      const rgb = parseHex(hex);
      expect(rgb).not.toBeNull();
      if (rgb) {
        expect(toHexColor(hsvToRgb(rgbToHsv(rgb)))).toBe(hex);
      }
    }
  });

  test("reads hex with or without #, in any case, and nothing else", () => {
    expect(parseHex("a48bff")).toEqual([164, 139, 255]);
    expect(parseHex(" #A48BFF ")).toEqual([164, 139, 255]);
    expect(parseHex("#abc")).toBeNull();
    expect(parseHex("#GGGGGG")).toBeNull();
  });
});

describe("picking a bot's color", () => {
  async function editScout() {
    const fake = new FakeBotloft();
    const crew = fake.addCrew("Ops");
    const scout = fake.addBot(crew.id, "Scout");
    renderApp(fake);
    await crewOpened("Ops");
    openBot("Scout");
    await screen.findByRole("heading", { level: 1, name: "Scout" });
    fireEvent.click(screen.getByRole("button", { name: "More bot actions" }));
    fireEvent.click(screen.getByRole("menuitem", { name: "Edit" }));
    return { fake, scout, dialog: screen.getByRole("dialog", { name: "Edit Scout" }) };
  }

  test("offers the whole palette and any color typed as hex", async () => {
    const { fake, scout, dialog } = await editScout();
    for (const swatch of AVATAR_PALETTE) {
      expect(within(dialog).getByRole("button", { name: `Color ${swatch}` })).toBeDefined();
    }
    fireEvent.click(within(dialog).getByRole("button", { name: "Pick any color" }));
    fireEvent.change(within(dialog).getByLabelText("Hex"), { target: { value: "#123abc" } });
    expect((within(dialog).getByLabelText("R") as HTMLInputElement).value).toBe("18");
    fireEvent.click(within(dialog).getByRole("button", { name: "Save" }));
    await waitFor(() => expect(fake.bots.get(scout.id)?.color).toBe("#123ABC"));
  });

  test("red, green and blue set the color too", async () => {
    const { fake, scout, dialog } = await editScout();
    fireEvent.click(within(dialog).getByRole("button", { name: "Pick any color" }));
    for (const [channel, value] of [
      ["R", "0"],
      ["G", "128"],
      ["B", "255"],
    ]) {
      fireEvent.change(within(dialog).getByLabelText(channel as string), {
        target: { value },
      });
    }
    expect((within(dialog).getByLabelText("Hex") as HTMLInputElement).value).toBe("#0080FF");
    fireEvent.click(within(dialog).getByRole("button", { name: "Save" }));
    await waitFor(() => expect(fake.bots.get(scout.id)?.color).toBe("#0080FF"));
  });

  test("a bot with a color outside the palette opens with the picker", async () => {
    const fake = new FakeBotloft();
    const crew = fake.addCrew("Ops");
    const scout = fake.addBot(crew.id, "Scout");
    scout.color = "#123ABC";
    renderApp(fake);
    await crewOpened("Ops");
    openBot("Scout");
    await screen.findByRole("heading", { level: 1, name: "Scout" });
    fireEvent.click(screen.getByRole("button", { name: "More bot actions" }));
    fireEvent.click(screen.getByRole("menuitem", { name: "Edit" }));
    const dialog = screen.getByRole("dialog", { name: "Edit Scout" });
    expect((within(dialog).getByLabelText("Hex") as HTMLInputElement).value).toBe("#123ABC");
  });
});
