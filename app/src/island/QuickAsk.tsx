// On the island's Home (spec 31.4): ask an agent for something in one line,
// without opening the Chat tab. The agent comes from a small menu; what it
// answers shows in the Chat tab and in the app.

import { ArrowUp, ChevronDown } from "lucide-react";
import { type FormEvent, useState } from "react";
import { BotAvatar } from "../features/bots/BotAvatar";
import { useT } from "../i18n";
import type { BotloftApi } from "../lib/api";
import type { Bot, BotId } from "../lib/protocol.gen";

export function QuickAsk({
  api,
  bots,
  botId,
  onPick,
  onFollow,
}: {
  api: BotloftApi;
  /** Who waits or works first, as in the Chat tab. */
  bots: Bot[];
  botId: BotId | null;
  onPick: (id: BotId) => void;
  /** Opens the Chat tab on the agent. */
  onFollow: (id: BotId) => void;
}) {
  const t = useT().island.quick;
  const pick = useT().island.chat.pick;
  const [text, setText] = useState("");
  const [menu, setMenu] = useState(false);
  const [busy, setBusy] = useState(false);
  const [sent, setSent] = useState<Bot | null>(null);
  const [failed, setFailed] = useState(false);
  const bot = bots.find((have) => have.id === botId) ?? bots[0];
  if (!bot) {
    return null;
  }
  const send = (event: FormEvent) => {
    event.preventDefault();
    const body = text.trim();
    if (busy || body === "") {
      return;
    }
    setBusy(true);
    setFailed(false);
    api
      .call("messages.send", { botId: bot.id, body })
      .then(
        () => {
          setText("");
          setSent(bot);
        },
        () => setFailed(true),
      )
      .finally(() => setBusy(false));
  };
  return (
    <form onSubmit={send} className="island-card flex flex-col gap-2 p-3">
      <div className="flex items-center gap-2">
        <button
          type="button"
          aria-haspopup="listbox"
          aria-expanded={menu}
          aria-label={pick}
          onClick={() => setMenu(!menu)}
          className="flex shrink-0 items-center gap-1.5 rounded-full px-1.5 py-1 hover:bg-[var(--island-tab)]"
        >
          <BotAvatar color={bot.color} size={24} mood="idle" still />
          <ChevronDown aria-hidden size={13} className="island-muted" />
        </button>
        <input
          value={text}
          onChange={(event) => {
            setText(event.target.value);
            setSent(null);
          }}
          aria-label={t.label(bot.name)}
          placeholder={t.placeholder(bot.name)}
          className="h-9 min-w-0 flex-1 rounded-full bg-[var(--island-tab)] px-4 text-[13px] outline-none placeholder:text-[var(--island-muted)]"
        />
        <button
          type="submit"
          aria-label={t.send}
          disabled={busy || text.trim() === ""}
          className="grid size-9 shrink-0 place-items-center rounded-full bg-[var(--island-ink)] text-black disabled:opacity-40"
        >
          <ArrowUp aria-hidden size={15} />
        </button>
      </div>
      {sent && (
        <p role="status" className="flex items-center gap-2 px-1 text-[12px]">
          <span className="island-muted">{t.sent(sent.name)}</span>
          <button
            type="button"
            onClick={() => onFollow(sent.id)}
            className="underline underline-offset-2"
          >
            {t.follow}
          </button>
        </p>
      )}
      {failed && (
        <p role="alert" className="px-1 text-[12px] text-[var(--island-danger)]">
          {t.failed}
        </p>
      )}
      {menu && (
        <div
          role="listbox"
          aria-label={pick}
          className="flex max-h-40 flex-wrap gap-1.5 overflow-auto px-1 pb-1"
        >
          {bots.map((other) => (
            <button
              key={other.id}
              type="button"
              role="option"
              aria-selected={other.id === bot.id}
              onClick={() => {
                onPick(other.id);
                setMenu(false);
                setSent(null);
              }}
              className={`flex items-center gap-1.5 rounded-full py-1 pr-3 pl-1 text-[13px] ${other.id === bot.id ? "bg-[var(--island-ink)] text-black" : "bg-[var(--island-tab)] hover:brightness-125"}`}
            >
              <BotAvatar color={other.color} size={20} mood="idle" still />
              <span className="max-w-32 truncate">{other.name}</span>
            </button>
          ))}
        </div>
      )}
    </form>
  );
}
