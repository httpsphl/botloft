// The phone's page with a made-up computer, to look at it in a browser:
// `pnpm dev`, then open /src/phone/preview.html (add ?lang=en for English).

const params = new URLSearchParams(location.search);
localStorage.setItem("botloft.locale", params.get("lang") ?? "pt-BR");
document.documentElement.dataset.theme = params.get("theme") === "dark" ? "dark" : "light";

const { createRoot } = await import("react-dom/client");
const { FakePhone } = await import("./fake");
const { PhoneApp } = await import("./PhoneApp");
await import("./phone.css");

const now = Date.now();
const bot = (
  botId: string,
  name: string,
  crew: string,
  color: string,
  state: "idle" | "busy" | "offline",
  text: string,
  kind: "reply" | "owner" | "tool",
  ago: number,
) => ({
  botId,
  name,
  crew,
  color,
  state,
  lastReplyAt: now - ago,
  last: { kind, text, tool: null, at: now - ago },
});

const phone = new FakePhone({
  load: () => ({ bot_1: now, bot_2: now, bot_3: now, bot_4: now, bot_5: now }),
  save: () => {},
});
phone.receive({
  t: "chats",
  first: true,
  bots: [
    bot("bot_1", "Lead", "Agência", "#FF7A59", "busy", "ls -la src", "tool", 60_000),
    bot(
      "bot_2",
      "Redatora",
      "Agência",
      "#6C8EF5",
      "idle",
      "Texto do post pronto, quer revisar?",
      "reply",
      600_000,
    ),
    bot(
      "bot_3",
      "Designer",
      "Agência",
      "#4CC38A",
      "idle",
      "Troquei as cores do banner.",
      "reply",
      3_600_000,
    ),
    bot(
      "bot_4",
      "Caixa",
      "Financeiro",
      "#E5B83D",
      "idle",
      "Conciliação de outubro concluída.",
      "reply",
      7_200_000,
    ),
    bot(
      "bot_5",
      "Analista",
      "Financeiro",
      "#C77DFF",
      "offline",
      "Gere o relatório",
      "owner",
      86_400_000,
    ),
  ],
});
// A reply the owner has not seen.
phone.set({ seen: { bot_1: now, bot_2: 0, bot_3: now, bot_4: now, bot_5: now } });
phone.set({
  approvals: [
    {
      approvalId: "apr_1",
      bot: { name: "Lead", color: "#FF7A59" },
      crew: "Agência",
      createdAt: now,
      toolName: "Bash",
      summary: "git status",
      text: "git status --short",
      cut: false,
      atComputer: false,
    },
  ],
});

const history = [
  { kind: "you", id: "i1", at: 1, text: "Prepare o post de lançamento", cut: false },
  { kind: "tool", id: "i2", at: 2, summary: "Read brief.md" },
  {
    kind: "reply",
    id: "i3",
    at: 3,
    text: "Pronto! Escrevi **três versões** do post:\n\n1. Direta\n2. Divertida\n3. Técnica\n\nQual você prefere?",
    cut: false,
  },
  { kind: "you", id: "i4", at: 4, text: "A divertida, e deixe mais curta", cut: false },
] as const;

const original = phone.openChat;
phone.openChat = async (botId: string) => {
  await original(botId);
  const req = phone.asked.filter((ask) => ask.t === "history").at(-1);
  if (req?.t === "history") {
    phone.receive({
      t: "history",
      req: req.req,
      botId,
      items: [...history],
      more: true,
      done: true,
    });
    phone.receive({
      t: "live",
      botId,
      text: "Aqui vai a versão curta:\n\nHoje o Botloft ganhou um…",
    });
  }
};

const root = document.getElementById("root");
if (root) {
  createRoot(root).render(<PhoneApp api={phone} />);
}

// ?scene=requests | chats | crew | chat shows that screen at once.
const scene = params.get("scene");
const click = (find: (el: HTMLElement) => boolean) =>
  [...document.querySelectorAll<HTMLElement>("button, [role=tab]")].find(find)?.click();
setTimeout(() => {
  if (scene === "chats" || scene === "crew" || scene === "chat") {
    click(
      (el) => el.getAttribute("role") === "tab" && /Conversas|Chats/.test(el.textContent ?? ""),
    );
  }
  if (scene === "crew") {
    setTimeout(
      () =>
        click(
          (el) =>
            el.getAttribute("aria-pressed") === "false" && /Financeiro/.test(el.textContent ?? ""),
        ),
      50,
    );
  }
  if (scene === "chat") {
    setTimeout(
      () => click((el) => /Redatora/.test(el.textContent ?? "") && el.tagName === "BUTTON"),
      50,
    );
  }
}, 100);
