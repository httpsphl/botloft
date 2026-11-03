import { cleanup, render } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { ToolTile } from "./ToolTile";
import { KNOWN_TOOLS, knownTool, TOOL_KINDS, toolIcon } from "./toolCatalog";

afterEach(cleanup);

describe("the claude.ai tools the app knows (spec 34.7)", () => {
  test("a tool is known by its name or a name that starts with it as a word", () => {
    expect(knownTool("Gmail")?.key).toBe("gmail");
    expect(knownTool("Atlassian MCP")?.key).toBe("atlassian");
    expect(knownTool("Zoom for Claude")?.key).toBe("zoom");
    expect(knownTool("Fathom – Your Meeting Intelligence Layer")?.key).toBe("fathom");
    expect(knownTool("Intuit QuickBooks Labs")?.key).toBe("quickbooks");
    expect(knownTool("HyperFrames by HeyGen")?.key).toBe("heygen");
    expect(knownTool("Makeswift")).toBeUndefined();
    expect(knownTool("ZoomInfo")?.key).toBe("zoominfo");
    expect(knownTool("Hexagon")).toBeUndefined();
    expect(knownTool("Some Company Tool")).toBeUndefined();
  });

  test("each has its own key, name and official icon, in a kind Discover shows", () => {
    expect(new Set(KNOWN_TOOLS.map((tool) => tool.key)).size).toBe(KNOWN_TOOLS.length);
    expect(new Set(KNOWN_TOOLS.map((tool) => tool.name)).size).toBe(KNOWN_TOOLS.length);
    expect(KNOWN_TOOLS.length).toBeGreaterThanOrEqual(80);
    expect(KNOWN_TOOLS.filter((tool) => !toolIcon(tool.key)).map((tool) => tool.key)).toEqual([]);
    expect(KNOWN_TOOLS.filter((tool) => !TOOL_KINDS.includes(tool.kind))).toEqual([]);
  });

  test("the tile shows the official icon, or a plug for a tool the app does not know", () => {
    const tile = (name: string) => render(<ToolTile name={name} />).container.firstElementChild;
    expect(tile("Gmail")?.querySelector("img")?.getAttribute("src")).toContain("gmail.png");
    const unknown = tile("Some Company Tool");
    expect(unknown?.querySelector("img")).toBeNull();
    expect(unknown?.className).toContain("bg-sunken");
  });
});
