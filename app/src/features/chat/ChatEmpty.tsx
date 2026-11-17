// A chat with nothing in it yet: the bot, its role, and how to start. The
// Maker invites the owner to tell it about their work, or to bring their
// Claude Code history or their claude.ai export (spec 36.3, 36.5, 36.6).

import { FileArchive, History } from "lucide-react";
import { useState } from "react";
import { useT } from "../../i18n";
import type { Bot } from "../../lib/protocol.gen";
import { makerBot } from "../../store/app";
import { useApp } from "../../store/context";
import { Button } from "../../ui/Button";
import { BotAvatar, moodOf } from "../bots/BotAvatar";
import { ClaudeAiImport } from "../maker/ClaudeAiImport";
import { HistoryImport } from "../maker/HistoryImport";

export function ChatEmpty({ bot, stopped }: { bot: Bot; stopped: boolean }) {
  const t = useT();
  const isMaker = useApp((state) => makerBot(state)?.id === bot.id);
  const [importing, setImporting] = useState<"code" | "ai" | null>(null);
  return (
    <div className="flex flex-col items-center gap-3 py-16 text-center">
      {/* The Maker shows larger, with its wand and its flow behind it (spec 36.3). */}
      <span className="animate-float">
        <BotAvatar
          color={bot.color}
          size={isMaker ? 96 : 56}
          maker={isMaker}
          scene={isMaker}
          mood={moodOf(bot, stopped)}
          botId={bot.id}
          starting={bot.state === "launching"}
        />
      </span>
      {isMaker ? (
        <>
          <p className="max-w-md font-semibold text-base">{t.maker.emptyTitle}</p>
          <p className="max-w-md text-ink-soft text-sm leading-relaxed">{t.maker.emptyBody}</p>
          <div className="flex flex-wrap justify-center gap-2">
            <Button icon={History} onClick={() => setImporting("code")}>
              {t.maker.history.open}
            </Button>
            <Button icon={FileArchive} onClick={() => setImporting("ai")}>
              {t.maker.claudeAi.open}
            </Button>
          </div>
          {importing === "code" && <HistoryImport onClose={() => setImporting(null)} />}
          {importing === "ai" && <ClaudeAiImport onClose={() => setImporting(null)} />}
        </>
      ) : (
        <>
          <p className="font-semibold text-base">{t.chat.view.emptyTitle(bot.name)}</p>
          {bot.role && <p className="max-w-md text-ink-soft text-sm">{bot.role}</p>}
          <p className="max-w-md text-muted text-sm">{t.chat.view.emptyBody}</p>
        </>
      )}
    </div>
  );
}
