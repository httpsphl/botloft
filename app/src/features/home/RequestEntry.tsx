// A request waiting for the owner, on the home (spec 15.1): who asks, from
// which crew and when on its line, then the request and its answers, with
// the phone's rules (28.5). A request too long to show whole is only denied
// here; one that changes things for good is answered in the chat.

import { Check, Hand } from "lucide-react";
import { useT } from "../../i18n";
import type { ApprovalCard } from "../../lib/protocol.gen";
import { useApp } from "../../store/context";
import { Button } from "../../ui/Button";
import { RequestCard } from "../../ui/ChatCard";
import { Details } from "../../ui/Details";
import { isCommand } from "../chat/command";
import { RequestAnswers } from "../chat/RequestAnswers";
import { pretty } from "../chat/ToolLines";
import { toolAction, toolTitle } from "../chat/toolNames";
import { useAnswer } from "../chat/useAnswer";
import { Elsewhere, Entry } from "./Entry";

const BOX =
  "max-h-56 overflow-auto break-all rounded-lg border border-line bg-sunken px-2.5 py-1.5 font-mono text-xs";

export function RequestEntry({ card, onDone }: { card: ApprovalCard; onDone(): void }) {
  const t = useT();
  const words = t.home.request;
  const selectBot = useApp((state) => state.selectBot);
  const { note, setNote, busy, answer } = useAnswer(card, {
    allow: t.chat.approval.allowFailed,
    deny: t.chat.approval.denyFailed,
  });
  const reply = async (allow: boolean) => {
    await answer(allow);
    onDone();
  };
  const bot = card.bot.name;
  const open = () => selectBot(card.bot.id);
  // The command, or what the request holds, reads first when the bot said nothing.
  const plain = isCommand(card.toolName);
  return (
    <Entry bot={card.bot} crew={card.crew} at={card.createdAt}>
      <RequestCard
        label={t.chat.approval.asks(bot, toolAction(card.toolName, t.tools))}
        icon={Hand}
        tone="warn"
        title={toolTitle(card.toolName, t.tools)}
      >
        {card.explanation ? (
          <p className="mt-2 break-words text-sm" data-selectable>
            {card.explanation}
          </p>
        ) : (
          // A command shows whole below; anything else by what it is about.
          !plain &&
          card.summary && (
            <p className="mt-2 break-words font-mono text-ink-soft text-xs" data-selectable>
              {card.summary}
            </p>
          )
        )}
        <Details label={words.seeAll} open={plain && !card.explanation}>
          <div className={BOX} data-selectable>
            {plain ? card.text : pretty(card.text)}
          </div>
        </Details>
        {card.atComputer ? (
          <Elsewhere text={words.atComputer} onOpen={open} />
        ) : (
          <>
            {card.cut && <p className="mt-2 text-muted text-xs">{words.cut}</p>}
            <RequestAnswers
              bot={bot}
              deny={t.chat.approval.deny}
              busy={busy}
              note={note}
              onNote={setNote}
              onDeny={() => reply(false)}
            >
              {!card.cut && (
                <Button variant="primary" icon={Check} disabled={busy} onClick={() => reply(true)}>
                  {t.chat.approval.allow}
                </Button>
              )}
            </RequestAnswers>
          </>
        )}
      </RequestCard>
    </Entry>
  );
}
