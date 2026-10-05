// A bot asking to rename or change itself, or the chief another bot of its
// crew (spec 10.3): what changes, before and after, and why. The owner
// allows it or says no with a note. Answered, it shrinks to one line.

import { Ban, Check, TimerOff, UserPen } from "lucide-react";
import { useState } from "react";
import { useT } from "../../i18n";
import { errorText } from "../../lib/api";
import type { ApprovalItem, Bot } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";
import { Button } from "../../ui/Button";
import { NoteArea, SectionCard, SettledLine } from "../../ui/ChatCard";

export const CHANGE_BOT_TOOL = "mcp__botloft__change_bot";

interface Profile {
  name: string;
  role: string;
  instructions: string;
}

interface Asked {
  bot_id: string;
  name: string;
  before: Profile | null;
  after: Profile | null;
  reason: string | null;
}

function askedOf(input: string): Partial<Asked> {
  try {
    return JSON.parse(input) as Partial<Asked>;
  } catch {
    return {};
  }
}

function Answered({ approval, name }: { approval: ApprovalItem; name: string }) {
  const w = useT().chat.botChange;
  const [icon, text, tone] =
    approval.status === "allowed"
      ? ([Check, w.done(name), "ok"] as const)
      : approval.status === "denied"
        ? ([Ban, w.declined(name), "danger"] as const)
        : ([TimerOff, w.expired(name), "quiet"] as const);
  return <SettledLine icon={icon} tone={tone} text={text} note={approval.note} />;
}

export function BotChangeCard({ approval, bot }: { approval: ApprovalItem; bot: Bot }) {
  const t = useT();
  const w = t.chat.botChange;
  const d = t.bots.dialog;
  const api = useApi();
  const asked = askedOf(approval.input);
  const name = asked.name ?? "";
  const [note, setNote] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const { before, after } = asked;
  if (approval.status !== "pending" || !before || !after) {
    return <Answered approval={approval} name={name} />;
  }
  const answer = async (allow: boolean) => {
    setBusy(true);
    setError(null);
    const trimmed = note.trim();
    try {
      await api.call("approvals.answer", {
        approvalId: approval.approvalId,
        allow,
        ...(!allow && trimmed ? { note: trimmed } : {}),
      });
    } catch (failure) {
      setError(`${w.failed}: ${errorText(failure)}`);
    }
    setBusy(false);
  };
  const rows: [string, string, string][] = [];
  if (before.name !== after.name) rows.push([d.name, before.name, after.name]);
  if (before.role !== after.role) rows.push([d.role, before.role, after.role]);
  const self = asked.bot_id === bot.id;
  return (
    <SectionCard
      label={self ? w.titleSelf(bot.name) : w.titleOther(bot.name, name)}
      icon={UserPen}
      tone="accent"
      footer={
        <>
          <NoteArea
            label={w.noteLabel(bot.name)}
            value={note}
            onChange={setNote}
            placeholder={w.notePlaceholder(bot.name)}
          />
          <div className="mt-2.5 flex flex-wrap gap-2">
            <Button variant="primary" icon={Check} disabled={busy} onClick={() => answer(true)}>
              {w.apply}
            </Button>
            <Button icon={Ban} disabled={busy} onClick={() => answer(false)}>
              {w.decline}
            </Button>
          </div>
        </>
      }
    >
      <div className="flex flex-col gap-3 px-4 py-3 text-sm">
        {rows.length > 0 && (
          <dl className="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1.5">
            {rows.map(([what, was, becomes]) => (
              <div key={what} className="contents">
                <dt className="text-muted">{what}</dt>
                <dd className="min-w-0 break-words">
                  <span className="text-muted line-through">{was}</span>
                  {" → "}
                  <span className="font-medium">{becomes}</span>
                </dd>
              </div>
            ))}
          </dl>
        )}
        {before.instructions !== after.instructions && (
          <details className="text-xs">
            <summary className="cursor-pointer text-muted">{w.newInstructions}</summary>
            <p className="mt-1 whitespace-pre-wrap text-ink-soft" data-selectable>
              {after.instructions}
            </p>
          </details>
        )}
        {asked.reason && (
          <p>
            <span className="text-muted">{w.reason(bot.name)} </span>
            <span data-selectable>{asked.reason}</span>
          </p>
        )}
        <p className="text-muted text-xs">{w.explain}</p>
        {error && (
          <p role="alert" className="text-danger text-sm">
            {error}
          </p>
        )}
      </div>
    </SectionCard>
  );
}
