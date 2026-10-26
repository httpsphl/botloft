// An agent asking the owner for a key (spec 32.5): the name, what it is for
// and the form that keeps it out of the chat. Once answered it shrinks to
// one line that names the key, never its value.

import { Check, KeyRound, X } from "lucide-react";
import { useState } from "react";
import { useT } from "../../i18n";
import type { Bot, ChatItemId, KeyRequestItem } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";
import { Button } from "../../ui/Button";
import { SectionCard, SettledLine } from "../../ui/ChatCard";
import { attempt } from "../../ui/toast";
import { KeyForm } from "../keys/KeyForm";

export function KeyRequestCard({
  id,
  request,
  bot,
}: {
  id: ChatItemId;
  request: KeyRequestItem;
  bot: Bot;
}) {
  const t = useT().keys.card;
  if (request.status === "saved") {
    return <SettledLine icon={Check} tone="ok" text={t.saved(request.name)} />;
  }
  if (request.status === "dismissed") {
    return <SettledLine icon={X} tone="quiet" text={t.dismissed(request.name)} />;
  }
  return <OpenRequest id={id} request={request} bot={bot} />;
}

function OpenRequest({ id, request, bot }: { id: ChatItemId; request: KeyRequestItem; bot: Bot }) {
  const t = useT().keys.card;
  const api = useApi();
  const [busy, setBusy] = useState(false);
  const notNow = async () => {
    setBusy(true);
    await attempt(t.dismissFailed, () => api.call("keys.dismiss", { requestId: id }));
    setBusy(false);
  };
  return (
    <SectionCard
      label={t.asks(bot.name)}
      icon={KeyRound}
      tone="warn"
      footer={
        <KeyForm
          bot={bot}
          name={request.name}
          requestId={id}
          // The card closes when the daemon rewrites the request.
          onSaved={() => {}}
          actions={
            <Button
              type="button"
              variant="ghost"
              icon={X}
              disabled={busy}
              title={t.notNowHint(bot.name)}
              onClick={notNow}
            >
              {t.notNow}
            </Button>
          }
        />
      }
    >
      <div className="flex flex-col gap-1.5 px-4 py-3 text-sm" data-selectable>
        <p className="font-mono font-semibold">{request.name}</p>
        <p className="whitespace-pre-wrap text-ink-soft">
          <span className="sr-only">{t.why}: </span>
          {request.why}
        </p>
      </div>
    </SectionCard>
  );
}
