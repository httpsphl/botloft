// Dev only: the preview's desktop (spec 24). Analyst asks to see a
// spreadsheet in Excel, and Scout may already see Notepad.

import type { FakeBotloft } from "../lib/fake";
import type { BotId } from "../lib/protocol.gen";

const EXCEL = "C:\\Program Files\\Microsoft Office\\root\\Office16\\EXCEL.EXE";

export function seedDesktop(fake: FakeBotloft, bots: { scout: BotId; analyst: BotId }) {
  fake.desktop.grant(bots.scout, "C:\\Windows\\notepad.exe", "Notepad");
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
