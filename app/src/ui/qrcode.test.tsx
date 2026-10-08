import { cleanup, render, screen } from "@testing-library/react";
import qrcode from "qrcode-generator";
import { afterEach, expect, test } from "vitest";
import { QrCode } from "./QrCode";

afterEach(cleanup);

/** The dark squares a path draws, as "row,column". */
function squares(path: string): Set<string> {
  const dark = new Set<string>();
  for (const [, x, y, width] of path.matchAll(/M(\d+) (\d+)h(\d+)v1h-\d+z/g)) {
    for (let column = Number(x); column < Number(x) + Number(width); column++) {
      dark.add(`${y},${column}`);
    }
  }
  return dark;
}

test("the picture holds exactly the squares of the code for the address", () => {
  const url = `https://botloft.example/m#p=${"A".repeat(22)}&s=${"b".repeat(43)}&d=${"C".repeat(87)}`;
  render(<QrCode text={url} label="Code" />);
  const image = screen.getByRole("img", { name: "Code" });
  const drawn = squares(image.querySelector("path")?.getAttribute("d") ?? "");

  const code = qrcode(0, "M");
  code.addData(url);
  code.make();
  const expected = new Set<string>();
  for (let row = 0; row < code.getModuleCount(); row++) {
    for (let column = 0; column < code.getModuleCount(); column++) {
      if (code.isDark(row, column)) {
        expected.add(`${row},${column}`);
      }
    }
  }
  expect(drawn.size).toBeGreaterThan(500);
  expect(drawn).toEqual(expected);
  // Black on white and a quiet margin, whatever the theme.
  expect(image.querySelector("rect")?.getAttribute("fill")).toBe("#fff");
  expect(image.getAttribute("viewBox")?.startsWith("-4 -4")).toBe(true);
});
