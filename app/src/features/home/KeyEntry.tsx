// An agent's request for a key, on the home (spec 32.5): who asks, for
// which key and why. The key is pasted only in the chat's card, which keeps
// it out of the conversation, so here there is only the way to it.

import { KeyRound } from "lucide-react";
import { useT } from "../../i18n";
import type { KeyCard } from "../../lib/protocol.gen";
import { useApp } from "../../store/context";
import { RequestCard } from "../../ui/ChatCard";
import { Elsewhere, Entry } from "./Entry";

export function KeyEntry({ card }: { card: KeyCard }) {
  const t = useT();
  const selectBot = useApp((state) => state.selectBot);
  return (
    <Entry bot={card.bot} crew={card.crew} at={card.createdAt}>
      <RequestCard
        label={t.keys.card.asks(card.bot.name)}
        icon={KeyRound}
        tone="warn"
        title={t.home.key.title}
      >
        <div className="mt-2 flex flex-col gap-1 text-sm" data-selectable>
          <p className="font-mono font-semibold">{card.name}</p>
          <p className="whitespace-pre-wrap break-words text-ink-soft">
            <span className="sr-only">{t.keys.card.why}: </span>
            {card.why}
          </p>
        </div>
        <Elsewhere text={t.home.key.inChat} onOpen={() => selectBot(card.bot.id)} />
      </RequestCard>
    </Entry>
  );
}
