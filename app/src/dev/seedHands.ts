// Dev only: a Clerk that asks the owner to sign in for it, in the fake
// preview (spec 21.10). The sign-in page is drawn on a canvas and answers
// the owner's clicks and keys, so taking the browser can be tried end to end.

import type { FakeBotloft } from "../lib/fake";
import type { PageSize } from "../lib/fakePages";
import type { BrowserInput, CrewId } from "../lib/protocol.gen";
import { activeTab, drawPage } from "./seedPage";

const SITE = "https://flour.example/";

const FIELDS = { user: 300, password: 390 } as const;
type Field = keyof typeof FIELDS;

interface Form {
  user: string;
  password: string;
  field: Field | null;
  signedIn: boolean;
}

function draw(form: Form, size: PageSize): string {
  const canvas = document.createElement("canvas");
  canvas.width = size.width;
  canvas.height = size.height;
  const g = canvas.getContext("2d");
  if (!g) {
    return "";
  }
  g.fillStyle = "#f7f4ee";
  g.fillRect(0, 0, size.width, size.height);
  g.fillStyle = "#3b2a1a";
  g.font = "600 36px Georgia, serif";
  if (form.signedIn) {
    g.fillText(`Welcome back, ${form.user || "baker"}`, 440, 220);
    g.font = "20px system-ui, sans-serif";
    const orders = ["#1042 · 25 kg flour", "#1043 · 10 kg rye", "#1044 · 5 kg semolina"];
    for (const [row, line] of orders.entries()) {
      g.fillText(`Order ${line}`, 440, 300 + row * 48);
    }
    return canvas.toDataURL("image/jpeg", 0.7).split(",")[1] ?? "";
  }
  g.fillText("Flour Co. · Sign in", 440, 220);
  for (const name of ["user", "password"] as const) {
    const y = FIELDS[name];
    g.fillStyle = "#ffffff";
    g.strokeStyle = form.field === name ? "#c2410c" : "#b5a896";
    g.lineWidth = form.field === name ? 3 : 2;
    g.beginPath();
    g.roundRect(440, y, 400, 52, 12);
    g.fill();
    g.stroke();
    const value = name === "password" ? "•".repeat(form.password.length) : form.user;
    g.fillStyle = value ? "#1f1409" : "#9a8c7a";
    g.font = "20px system-ui, sans-serif";
    g.fillText(value || (name === "user" ? "E-mail" : "Password"), 460, y + 33);
  }
  g.fillStyle = "#c2410c";
  g.beginPath();
  g.roundRect(440, 480, 400, 52, 26);
  g.fill();
  g.fillStyle = "#ffffff";
  g.font = "600 20px system-ui, sans-serif";
  g.fillText("Sign in", 606, 513);
  return canvas.toDataURL("image/jpeg", 0.7).split(",")[1] ?? "";
}

/** The page's answer to one of the owner's events; true once they sign in. */
function handle(form: Form, input: BrowserInput): boolean {
  if (input.kind === "mouse" && input.action === "down") {
    const inside = (top: number) =>
      input.x >= 440 && input.x <= 840 && input.y >= top && input.y <= top + 52;
    form.field = inside(FIELDS.user) ? "user" : inside(FIELDS.password) ? "password" : null;
    return inside(480);
  }
  const field = form.field;
  if (input.kind === "text" && field) {
    form[field] += input.text;
  } else if (input.kind === "key" && input.key === "Enter") {
    return true;
  } else if (input.kind === "key" && input.key === "Tab") {
    form.field = form.field === "user" ? "password" : "user";
  } else if (input.kind === "key" && input.key === "Backspace" && field) {
    form[field] = form[field].slice(0, -1);
  } else if (input.kind === "key" && [...input.key].length === 1 && field && input.modifiers < 2) {
    form[field] += input.key;
  }
  return false;
}

export function seedHands(fake: FakeBotloft, crewId: CrewId): void {
  const clerk = fake.addBot(crewId, "Clerk", "Places the bakery's orders");
  fake.setBotState(clerk.id, "needs_approval", 2);
  const form: Form = { user: "", password: "", field: null, signedIn: false };
  const login = `${SITE}login`;
  const onSite = () => activeTab(fake, clerk.id)?.url.startsWith(SITE) ?? false;
  fake.browser.open(clerk.id, login, "Sign in · Flour Co.");
  fake.browser.paint(clerk.id, (size) =>
    onSite() ? draw(form, size) : drawPage(activeTab(fake, clerk.id), size),
  );
  fake.chat.tool(clerk.id, "mcp__botloft__browser_open", { summary: login, status: "done" });
  const task = "Sign in to Flour Co. with the bakery's account";
  fake.chat.tool(clerk.id, "mcp__botloft__browser_ask_owner", { summary: task, status: "running" });
  fake.browser.ask(clerk.id, task);
  fake.browser.onInput(clerk.id, (input) => {
    if (form.signedIn || !onSite()) {
      return;
    }
    if (handle(form, input)) {
      form.signedIn = true;
      fake.browser.open(clerk.id, `${SITE}orders`, "Orders · Flour Co.");
    }
    fake.browser.repaint(clerk.id);
  });
}
