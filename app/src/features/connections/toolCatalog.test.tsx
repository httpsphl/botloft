import { cleanup, render } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { ToolTile } from "./ToolTile";
import { contrastOnWhite, KNOWN_TOOLS, knownTool } from "./toolCatalog";

afterEach(cleanup);

describe("the claude.ai tools the app knows (spec 34.7)", () => {
  test("a tool is known by its name or a name that starts with it as a word", () => {
    expect(knownTool("Gmail")?.key).toBe("gmail");
    expect(knownTool("Atlassian MCP")?.key).toBe("atlassian");
    expect(knownTool("Zoom for Claude")?.key).toBe("zoom");
    expect(knownTool("Fathom – Your Meeting Intelligence Layer")?.key).toBe("fathom");
    expect(knownTool("Intuit QuickBooks Labs")?.key).toBe("quickbooks");
    expect(knownTool("Makeswift")).toBeUndefined();
    expect(knownTool("ZoomInfo")).toBeUndefined();
    expect(knownTool("Higgsfield")).toBeUndefined();
  });

  test("each has its own key and name", () => {
    expect(new Set(KNOWN_TOOLS.map((tool) => tool.key)).size).toBe(KNOWN_TOOLS.length);
    expect(new Set(KNOWN_TOOLS.map((tool) => tool.name)).size).toBe(KNOWN_TOOLS.length);
    expect(KNOWN_TOOLS.length).toBeGreaterThanOrEqual(60);
  });

  test("the tile draws the brand's mark, its initial, or a plug", () => {
    const tile = (name: string) => render(<ToolTile name={name} />).container.firstElementChild;
    const gmail = tile("Gmail");
    expect(gmail?.querySelector("svg")?.getAttribute("fill")).toBe("#EA4335");
    expect((gmail as HTMLElement).style.background).toBe("rgb(255, 255, 255)");
    // A pale brand color is the background, with the mark dark on it.
    const mailchimp = tile("Intuit Mailchimp");
    expect(mailchimp?.querySelector("svg")?.getAttribute("fill")).toBe("#111111");
    expect(contrastOnWhite("#FFE01B")).toBeLessThan(2.2);
    expect(tile("Slack")?.textContent).toBe("S");
    expect(tile("Higgsfield")?.querySelector("svg")).not.toBeNull();
    expect(tile("Higgsfield")?.className).toContain("bg-sunken");
  });
});
