// Dev only: any page of the fake preview's browsers that no seed draws
// itself, such as a tab in the background or an address the owner typed.
// A heading with the tab's title, its site, and lines of grey "text".

import type { FakeBotloft } from "../lib/fake";
import type { PageSize } from "../lib/fakePages";
import type { BotId, BrowserTab } from "../lib/protocol.gen";

/** The bot's active tab, if its browser has any. */
export function activeTab(fake: FakeBotloft, botId: BotId): BrowserTab | null {
  return fake.browser.state(botId).tabs.find((tab) => tab.active) ?? null;
}

export function drawPage(tab: BrowserTab | null, size: PageSize): string {
  const canvas = document.createElement("canvas");
  canvas.width = size.width;
  canvas.height = size.height;
  const g = canvas.getContext("2d");
  if (!g) {
    return "";
  }
  g.fillStyle = "#ffffff";
  g.fillRect(0, 0, size.width, size.height);
  if (tab && tab.url !== "about:blank") {
    g.fillStyle = "#111827";
    g.font = "600 40px Georgia, serif";
    g.fillText(tab.title || tab.url, 120, 160);
    g.fillStyle = "#6b7280";
    g.font = "20px system-ui, sans-serif";
    g.fillText(tab.url, 120, 204);
    g.fillStyle = "#e5e7eb";
    for (let y = 270; y < size.height - 80; y += 34) {
      g.fillRect(120, y, y % 5 === 0 ? 640 : 960, 14);
    }
  }
  return canvas.toDataURL("image/jpeg", 0.7).split(",")[1] ?? "";
}
