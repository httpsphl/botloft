// A line of "Needs you" on the home (spec 15.1): the agent's face, then who
// asks, from which crew and when, with the way to its chat, over the card.

import { MessageSquare } from "lucide-react";
import type { ReactNode } from "react";
import { useT } from "../../i18n";
import { when } from "../../lib/format";
import type { CardBot } from "../../lib/protocol.gen";
import { useApp } from "../../store/context";
import { Button } from "../../ui/Button";
import { BotAvatar } from "../bots/BotAvatar";

export function Entry({
  bot,
  crew,
  at,
  children,
}: {
  bot: CardBot;
  crew: string;
  at: number;
  children: ReactNode;
}) {
  const t = useT();
  const selectBot = useApp((state) => state.selectBot);
  return (
    <li className="flex gap-3">
      <BotAvatar color={bot.color} size={28} still />
      <div className="flex min-w-0 flex-1 flex-col gap-1">
        <div className="flex items-center gap-2 text-sm">
          <span className="font-semibold">{bot.name}</span>
          <span className="truncate text-muted text-xs">{t.questions.box.inCrew(crew)}</span>
          <time className="text-muted text-xs" dateTime={new Date(at).toISOString()}>
            {when(at)}
          </time>
          <Button
            variant="ghost"
            size="sm"
            icon={MessageSquare}
            label={t.home.request.openChat(bot.name)}
            className="ml-auto"
            onClick={() => selectBot(bot.id)}
          />
        </div>
        {children}
      </div>
    </li>
  );
}

/** What is answered in the chat: why, and the way there. */
export function Elsewhere({ text, onOpen }: { text: string; onOpen(): void }) {
  const t = useT().home.request;
  return (
    <div className="mt-3 flex flex-wrap items-center gap-3">
      <p className="min-w-0 flex-1 text-muted text-xs">{text}</p>
      <Button variant="primary" icon={MessageSquare} onClick={onOpen}>
        {t.openInChat}
      </Button>
    </div>
  );
}
