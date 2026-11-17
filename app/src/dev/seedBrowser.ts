// Dev only: Scout searching in its browser in the fake preview, so the
// browser panel has something live to show: three tabs, the search in the
// active one. The "page" is drawn on a canvas, the size the panel asks for,
// and sent as JPEG frames, like the daemon's screencast.

import type { FakeBotloft } from "../lib/fake";
import type { PageSize } from "../lib/fakePages";
import type { BotId } from "../lib/protocol.gen";
import { activeTab, drawPage } from "./seedPage";

const SITE = "https://recipes.example/";

const RECIPES = ["Classic sourdough loaf", "Starter in 5 days", "No-knead rye", "Focaccia"];

interface Scene {
  url: string;
  title: string;
  query: string;
  results: boolean;
}

/** Where the search box and its button sit, so a narrow page shows both. */
function searchBar(width: number): { left: number; box: number; button: number } {
  const left = Math.min(120, Math.round(width * 0.1));
  const box = Math.min(760, width - 2 * left - 170);
  return { left, box, button: left + box + 20 };
}

function draw(scene: Scene, size: PageSize): string {
  const canvas = document.createElement("canvas");
  canvas.width = size.width;
  canvas.height = size.height;
  const g = canvas.getContext("2d");
  if (!g) {
    return "";
  }
  const { left, box, button } = searchBar(size.width);
  g.fillStyle = "#ffffff";
  g.fillRect(0, 0, size.width, size.height);
  g.fillStyle = "#1f2937";
  g.font = "600 34px system-ui, sans-serif";
  g.fillText(scene.results ? "Results" : "Find a recipe", left, 150);
  g.strokeStyle = "#9ca3af";
  g.lineWidth = 2;
  g.beginPath();
  g.roundRect(left, 190, box, 56, 28);
  g.stroke();
  g.fillStyle = scene.query ? "#111827" : "#9ca3af";
  g.font = "22px system-ui, sans-serif";
  g.fillText(scene.query || "Search recipes", left + 30, 226);
  g.fillStyle = "#2563eb";
  g.beginPath();
  g.roundRect(button, 190, 150, 56, 28);
  g.fill();
  g.fillStyle = "#ffffff";
  g.fillText("Search", button + 38, 226);
  if (scene.results) {
    const line = Math.min(820, size.width - 2 * left);
    // A taller page shows more of the list.
    for (let row = 0; 300 + row * 110 < size.height - 60; row++) {
      const y = 300 + row * 110;
      g.fillStyle = "#1d4ed8";
      g.font = "600 24px system-ui, sans-serif";
      g.fillText(RECIPES[row % RECIPES.length] ?? "", left, y);
      g.fillStyle = "#e5e7eb";
      g.fillRect(left, y + 18, line, 14);
      g.fillRect(left, y + 42, Math.round(line * 0.68), 14);
    }
  }
  return canvas.toDataURL("image/jpeg", 0.7).split(",")[1] ?? "";
}

/** Scout searches for a recipe, over and over, while the browser is its. */
export function seedBrowser(fake: FakeBotloft, scout: BotId): void {
  const home: Scene = {
    url: SITE,
    title: "Recipes",
    query: "",
    results: false,
  };
  // The width of the page last drawn: the clicks land where it put things.
  let width = 1200;
  const steps: ((scene: Scene) => Scene)[] = [
    (scene) => {
      fake.browser.act(scout, "open", { label: "recipes.example" });
      return scene;
    },
    (scene) => {
      const { left, box } = searchBar(width);
      fake.browser.act(scout, "type", {
        x: left + Math.round(box / 2),
        y: 218,
        label: "Search recipes",
      });
      return { ...scene, query: "sourdough" };
    },
    (scene) => {
      const { button } = searchBar(width);
      fake.browser.act(scout, "click", { x: button + 75, y: 218, label: "Search" });
      return { ...scene, url: "https://recipes.example/search?q=sourdough", results: true };
    },
    (scene) => {
      const { left } = searchBar(width);
      fake.browser.act(scout, "click", { x: left + 140, y: 300, label: "Classic sourdough loaf" });
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
  // Two pages it opened on the way, then the one it works in.
  fake.browser.open(scout, "https://bakers.example/forum/hydration", "Hydration tips · Bakers");
  fake.browser.openTab(scout, "https://mill.example/flour/rye", "Rye flour · The Mill");
  fake.browser.openTab(scout, home.url, home.title);
  fake.browser.paint(scout, (size) => {
    width = size.width;
    const tab = activeTab(fake, scout);
    return tab?.url.startsWith(SITE) ? draw(scene, size) : drawPage(tab, size);
  });
  setInterval(() => {
    // In the owner's hands, the bot waits.
    if (fake.browser.state(scout).control === "owner") {
      return;
    }
    const next = steps[step % steps.length];
    step += 1;
    if (next) {
      scene = next(scene);
      fake.browser.open(scout, scene.url, scene.title);
      fake.browser.repaint(scout);
    }
  }, 2200);
}
