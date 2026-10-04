// A bot asking to see an app on the owner's desktop (spec 24.2): the app,
// why the bot wants it, what seeing it means, and Allow or Deny. Once
// answered it shrinks to one line.

import { Ban, Check, Monitor, TimerOff } from "lucide-react";
import { useT } from "../../i18n";
import type { ApprovalItem, Bot } from "../../lib/protocol.gen";
import { Button } from "../../ui/Button";
import { NoteInput, RequestCard, SettledLine } from "../../ui/ChatCard";
import { useAnswer } from "../chat/useAnswer";

/** The request the daemon opens from the desktop tools. */
export const DESKTOP_TOOL = "mcp__botloft__desktop";

interface Asked {
  path: string | null;
  why: string | null;
}

function askedOf(input: string): Asked {
  try {
    const parsed = JSON.parse(input) as { path?: unknown; why?: unknown };
    const text = (value: unknown) => (typeof value === "string" && value.trim() ? value : null);
    return { path: text(parsed.path), why: text(parsed.why) };
  } catch {
    return { path: null, why: null };
  }
}

function Answered({ approval, bot, app }: { approval: ApprovalItem; bot: Bot; app: string }) {
  const words = useT().desktop.card;
  switch (approval.status) {
    case "allowed":
      return <SettledLine icon={Check} tone="ok" text={words.allowed(bot.name, app)} />;
    case "denied":
      return (
        <SettledLine
          icon={Ban}
          tone="danger"
          text={words.denied(bot.name, app)}
          note={approval.note}
        />
      );
    default:
      return <SettledLine icon={TimerOff} tone="quiet" text={words.expired(app)} />;
  }
}

export function DesktopCard({ approval, bot }: { approval: ApprovalItem; bot: Bot }) {
  const t = useT();
  const words = t.desktop.card;
  const { note, setNote, busy, answer } = useAnswer(approval, {
    allow: t.chat.approval.allowFailed,
    deny: t.chat.approval.denyFailed,
  });
  const app = approval.summary;
  if (approval.status !== "pending") {
    return <Answered approval={approval} bot={bot} app={app} />;
  }
  const { path, why } = askedOf(approval.input);

  return (
    <RequestCard
      label={words.asks(bot.name, app)}
      icon={Monitor}
      tone="warn"
      title={
        <span>
          {words.wants(bot.name)} <span className="font-semibold">{app}</span>
        </span>
      }
    >
      {why && (
        <p className="mt-2 text-sm" data-selectable>
          <span className="text-muted">{words.because(bot.name)}</span> {why}
        </p>
      )}
      {path && (
        <p className="mt-1.5 break-all font-mono text-ink-soft text-xs" data-selectable>
          {path}
        </p>
      )}
      <p className="mt-1.5 text-muted text-xs">{words.means(bot.name)}</p>
      <NoteInput
        label={t.chat.approval.noteLabel(bot.name)}
        value={note}
        onChange={setNote}
        placeholder={t.chat.approval.notePlaceholder}
      />
      <div className="mt-2.5 flex gap-2">
        <Button variant="primary" icon={Check} disabled={busy} onClick={() => answer(true)}>
          {t.chat.approval.allow}
        </Button>
        <Button variant="danger" icon={Ban} disabled={busy} onClick={() => answer(false)}>
          {t.chat.approval.deny}
        </Button>
      </div>
    </RequestCard>
  );
}
