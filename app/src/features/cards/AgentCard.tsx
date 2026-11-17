// A card an agent showed in its chat (spec 8.11): the title, the blocks top
// to bottom and, once the owner answered this version, what they chose. A
// button or the form answers only the version on screen; the agent shows
// the card again to ask once more.

import { Check, LayoutDashboard } from "lucide-react";
import { useState } from "react";
import { useT } from "../../i18n";
import { clock, when } from "../../lib/format";
import type { CardBlock, CardItem, ChatItemId } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";
import { Button } from "../../ui/Button";
import { IconBadge } from "../../ui/ChatCard";
import { attempt } from "../../ui/toast";
import { Markdown } from "../chat/Markdown";
import { CardChart } from "./CardChart";
import { CardForm } from "./CardForm";
import { CardTable } from "./CardTable";

type Answer = { button: string } | { form: Record<string, string> };

export function AgentCard({
  id,
  card,
  bot,
  updatedAt,
}: {
  id: ChatItemId;
  card: CardItem;
  bot: string;
  updatedAt: number;
}) {
  const t = useT().agentCards;
  const api = useApi();
  const [busy, setBusy] = useState(false);
  const answer = card.answer?.version === card.version ? card.answer : null;

  const act = async (choice: Answer) => {
    setBusy(true);
    await attempt(t.failed, () =>
      api.call("chat.act", { itemId: id, version: card.version, ...choice }),
    );
    setBusy(false);
  };

  return (
    <section
      aria-label={t.label(bot, card.title)}
      className="my-1 flex w-full max-w-[44rem] flex-col overflow-hidden rounded-bubble bg-bubble"
    >
      <header className="flex items-center gap-2.5 px-4 pt-3.5">
        <IconBadge icon={LayoutDashboard} tone="accent" />
        <h3 className="min-w-0 flex-1 font-semibold text-sm">{card.title}</h3>
        {card.version > 1 && (
          <span className="shrink-0 text-muted text-xs">{t.updated(clock(updatedAt))}</span>
        )}
      </header>
      <div className="flex min-w-0 flex-col gap-3 px-4 py-3" data-selectable>
        {card.blocks.map((block, at) => (
          <Block
            // The blocks only change all together, as a new version.
            // biome-ignore lint/suspicious/noArrayIndexKey: blocks have no ids
            key={`${card.version}-${at}`}
            block={block}
            disabled={busy || answer !== null}
            act={act}
          />
        ))}
      </div>
      {answer && (
        <footer className="flex items-center gap-2 border-line border-t px-4 py-2.5 text-sm">
          <Check aria-hidden size={14} className="shrink-0 text-ok" />
          <span className="min-w-0 flex-1">
            {answer.button !== null ? t.pressed(answer.text) : t.sent}
          </span>
          <time
            className="shrink-0 text-muted text-xs"
            dateTime={new Date(answer.at).toISOString()}
            title={t.waiting(bot)}
          >
            {when(answer.at)}
          </time>
        </footer>
      )}
    </section>
  );
}

function Block({
  block,
  disabled,
  act,
}: {
  block: CardBlock;
  disabled: boolean;
  act: (choice: Answer) => void;
}) {
  switch (block.type) {
    case "text":
      return <Markdown text={block.text} />;
    case "table":
      return <CardTable columns={block.columns} rows={block.rows} />;
    case "chart":
      return <CardChart chart={block} />;
    case "buttons":
      return (
        <div className="flex flex-wrap gap-2">
          {block.buttons.map((button) => (
            <Button key={button.id} disabled={disabled} onClick={() => act({ button: button.id })}>
              {button.label}
            </Button>
          ))}
        </div>
      );
    case "form":
      return (
        <CardForm
          fields={block.fields}
          submit={block.submit}
          disabled={disabled}
          send={(form) => act({ form })}
        />
      );
  }
}
