// Dev only: Scout searching in its browser in the fake preview, so the
// browser panel has something live to show. The "page" is drawn on a
// canvas, the size the panel asks for, and sent as JPEG frames, like the
// daemon's screencast.

import type { FakeBotloft } from "../lib/fake";
import type { PageSize } from "../lib/fakeBrowser";
import type { BotId } from "../lib/protocol.gen";

const RECIPES = ["Classic sourdough loaf", "Starter in 5 days", "No-knead rye", "Focaccia"];

interface Scene {
  url: string;
  title: string;
  query: string;
  results: boolean;
}

function draw(scene: Scene, size: PageSize): string {
  const canvas = document.createElement("canvas");
  canvas.width = size.width;
  canvas.height = size.height;
  const g = canvas.getContext("2d");
  if (!g) {
    return "";
  }
  g.fillStyle = "#ffffff";
  g.fillRect(0, 0, size.width, size.height);
  g.fillStyle = "#1f2937";
  g.font = "600 34px system-ui, sans-serif";
  g.fillText(scene.results ? "Results" : "Find a recipe", 120, 150);
  g.strokeStyle = "#9ca3af";
  g.lineWidth = 2;
  g.beginPath();
  g.roundRect(120, 190, 760, 56, 28);
  g.stroke();
  g.fillStyle = scene.query ? "#111827" : "#9ca3af";
  g.font = "22px system-ui, sans-serif";
  g.fillText(scene.query || "Search recipes", 150, 226);
  g.fillStyle = "#2563eb";
  g.beginPath();
  g.roundRect(900, 190, 150, 56, 28);
  g.fill();
  g.fillStyle = "#ffffff";
  g.fillText("Search", 938, 226);
  if (scene.results) {
    // A taller page shows more of the list.
    for (let row = 0; 300 + row * 110 < size.height - 60; row++) {
      const y = 300 + row * 110;
      g.fillStyle = "#1d4ed8";
      g.font = "600 24px system-ui, sans-serif";
      g.fillText(RECIPES[row % RECIPES.length] ?? "", 120, y);
      g.fillStyle = "#e5e7eb";
      g.fillRect(120, y + 18, 820, 14);
      g.fillRect(120, y + 42, 560, 14);
    }
  }
  return canvas.toDataURL("image/jpeg", 0.7).split(",")[1] ?? "";
}

/** Scout searches for a recipe, over and over. */
export function seedBrowser(fake: FakeBotloft, scout: BotId): void {
  const home: Scene = {
    url: "https://recipes.example/",
    title: "Recipes",
    query: "",
    results: false,
  };
  const steps: ((scene: Scene) => Scene)[] = [
    (scene) => {
      fake.browser.act(scout, "open", { label: "recipes.example" });
      return scene;
    },
    (scene) => {
      fake.browser.act(scout, "type", { x: 500, y: 218, label: "Search recipes" });
      return { ...scene, query: "sourdough" };
    },
    (scene) => {
      fake.browser.act(scout, "click", { x: 975, y: 218, label: "Search" });
      return { ...scene, url: "https://recipes.example/search?q=sourdough", results: true };
    },
    (scene) => {
      fake.browser.act(scout, "click", { x: 260, y: 300, label: "Classic sourdough loaf" });
      return scene;
    },
    () => {
      fake.browser.act(scout, "back");
      return home;
    },
  ];
  // How it reads in the chat: the site the owner allowed, then the calls.
  const asked = fake.chat.ask(
    scout,
    "mcp__botloft__browser",
    "recipes.example",
    JSON.stringify({ site: "recipes.example", url: home.url }),
  );
  if (asked.body.kind === "approval") {
    void fake.call("approvals.answer", { approvalId: asked.body.approvalId, allow: true });
  }
  const calls: [string, string][] = [
    ["browser_open", home.url],
    ["browser_type", "e4"],
    ["browser_click", "e5"],
  ];
  for (const [tool, summary] of calls) {
    fake.chat.tool(scout, `mcp__botloft__${tool}`, { summary, status: "done" });
  }
  fake.chat.ask(
    scout,
    "mcp__botloft__browser",
    "wikipedia.org",
    JSON.stringify({ site: "wikipedia.org", url: "https://en.wikipedia.org/wiki/Sourdough" }),
  );
  let scene = home;
  let step = 0;
  fake.browser.open(scout, home.url, home.title);
  fake.browser.paint(scout, (size) => draw(scene, size));
  setInterval(() => {
    const next = steps[step % steps.length];
    step += 1;
    if (next) {
      scene = next(scene);
      fake.browser.open(scout, scene.url, scene.title);
      fake.browser.repaint(scout);
    }
  }, 2200);
}
