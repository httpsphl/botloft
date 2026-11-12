// The open conversation's title, and "New conversation" beside it (spec
// 40.7). Only Claude Code agents have more than their main one (40.1).

import { MessageSquarePlus } from "lucide-react";
import { useState } from "react";
import { useT } from "../../i18n";
import { mainConversationOf } from "../../lib/conversations";
import type { Bot } from "../../lib/protocol.gen";
import { useApp } from "../../store/context";
import { conversationOf, openConversationOf } from "../../store/conversations";
import { Button, FitLabel } from "../../ui/Button";
import { attempt } from "../../ui/toast";

export function ConversationControls({ bot }: { bot: Bot }) {
  const t = useT().conversations;
  const openId = useApp((state) => openConversationOf(state, bot.id));
  const conversation = useApp((state) => conversationOf(state, bot.id));
  const newConversation = useApp((state) => state.newConversation);
  const [busy, setBusy] = useState(false);
  if (bot.agent !== "claude") {
    return null;
  }
  const title =
    openId === mainConversationOf(bot.id) ? t.main : (conversation?.title ?? t.untitled);
  const start = async () => {
    setBusy(true);
    await attempt(t.newFailed, async () => {
      await newConversation(bot.id);
    });
    setBusy(false);
  };
  return (
    <div className="flex min-w-0 shrink items-center gap-1">
      <span
        className="min-w-0 truncate rounded-full bg-sunken px-2 py-0.5 text-muted text-xs"
        title={`${t.current}: ${title}`}
      >
        {title}
      </span>
      <Button
        size="sm"
        variant="ghost"
        icon={MessageSquarePlus}
        title={t.newHint(bot.name)}
        disabled={busy}
        onClick={() => void start()}
      >
        <FitLabel>{t.new}</FitLabel>
      </Button>
    </div>
  );
}
