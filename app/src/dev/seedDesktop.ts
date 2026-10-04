// Dev only: the preview's desktop (spec 24). Analyst asks to see a
// spreadsheet in Excel, and Scout may already see Notepad, where it just
// clicked Save.

import type { FakeBotloft } from "../lib/fake";
import type { BotId } from "../lib/protocol.gen";

const EXCEL = "C:\\Program Files\\Microsoft Office\\root\\Office16\\EXCEL.EXE";
const NOTES = { id: 42, title: "Shopping list - Notepad", app: "Notepad" };

/** A Notepad window, drawn: a title bar, a menu and a few lines. */
function drawNotepad(): string {
  const canvas = document.createElement("canvas");
  canvas.width = 760;
  canvas.height = 480;
  const g = canvas.getContext("2d");
  if (!g) {
    return "";
  }
  g.fillStyle = "#ffffff";
  g.fillRect(0, 0, 760, 480);
  g.fillStyle = "#f3f3f3";
  g.fillRect(0, 0, 760, 72);
  g.fillStyle = "#1f1f1f";
  g.font = "14px system-ui, sans-serif";
  g.fillText(NOTES.title, 16, 24);
  g.fillText("File     Edit     View", 16, 58);
  g.font = "16px Consolas, monospace";
  ["Flour, 2 kg", "Eggs, 12", "Butter, 500 g", "Sugar, 1 kg"].forEach((line, index) => {
    g.fillText(line, 20, 112 + index * 26);
  });
  return canvas.toDataURL("image/jpeg", 0.8).split(",")[1] ?? "";
}

export function seedDesktop(fake: FakeBotloft, bots: { scout: BotId; analyst: BotId }) {
  fake.desktop.grant(bots.scout, "C:\\Windows\\notepad.exe", "Notepad");
  fake.desktop.use(bots.scout, NOTES, { kind: "click", target: "Save", option: null });
  fake.desktop.paint(bots.scout, drawNotepad(), 760, 480);
  fake.chat.tool(bots.scout, "mcp__botloft__desktop_click", {
    summary: "To save the list",
    status: "done",
  });
  fake.chat.ask(
    bots.analyst,
    "mcp__botloft__desktop",
    "Microsoft Excel",
    JSON.stringify({
      app: "Microsoft Excel",
      path: EXCEL,
      level: "see",
      why: "To read the flour prices in your spreadsheet",
    }),
  );
}
