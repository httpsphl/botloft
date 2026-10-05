// A bot asking to reach another crew, or one bot of it (spec 10.4): what
// it wants there and why. The owner allows it only now, always for that
// bot, always for the whole crew, or says no with a note. Answered, it
// shrinks to one line that says for how long.

import { Ban, Check, Network, TimerOff } from "lucide-react";
import { useState } from "react";
import { useT } from "../../i18n";
import { errorText } from "../../lib/api";
import type { ApprovalItem, Bot } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";
import { Button } from "../../ui/Button";
import { NoteArea, SectionCard, SettledLine } from "../../ui/ChatCard";

export const CREW_ACCESS_TOOL = "mcp__botloft__ask_crew_access";

type Scope = "once" | "bot" | "crew";

interface Asked {
  crew: string;
  bot: string | null;
  access: string[];
  why: string;
  scope?: Scope;
}

function askedOf(input: string): Partial<Asked> {
  try {
    return JSON.parse(input) as Partial<Asked>;
  } catch {
    return {};
  }
}

function Answered({ approval, bot }: { approval: ApprovalItem; bot: Bot }) {
  const w = useT().chat.crewAccess;
  const asked = askedOf(approval.input);
  const where = asked.crew ?? "";
  if (approval.status === "allowed") {
    const scope = asked.scope ?? "once";
    const text =
      scope === "crew"
        ? w.doneCrew(bot.name, where)
        : scope === "bot"
          ? w.doneBot(bot.name, asked.bot ?? "", where)
          : w.doneOnce(bot.name, where);
    return <SettledLine icon={Check} tone="ok" text={text} />;
  }
  if (approval.status === "denied") {
    return (
      <SettledLine
        icon={Ban}
        tone="danger"
        text={w.declined(bot.name, where)}
        note={approval.note}
      />
    );
  }
  return <SettledLine icon={TimerOff} tone="quiet" text={w.expired(where)} />;
}

export function CrewAccessCard({ approval, bot }: { approval: ApprovalItem; bot: Bot }) {
  const t = useT();
  const w = t.chat.crewAccess;
  const api = useApi();
  const [note, setNote] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  if (approval.status !== "pending") {
    return <Answered approval={approval} bot={bot} />;
  }
  const asked = askedOf(approval.input);
  const crew = asked.crew ?? "";
  const target = asked.bot ?? null;
  const answer = async (allow: boolean, scope?: Scope) => {
    setBusy(true);
    setError(null);
    const trimmed = note.trim();
    try {
      await api.call("approvals.answer", {
        approvalId: approval.approvalId,
        allow,
        ...(!allow && trimmed ? { note: trimmed } : {}),
        ...(allow && scope ? { input: JSON.stringify({ scope }) } : {}),
      });
    } catch (failure) {
      setError(`${w.failed}: ${errorText(failure)}`);
    }
    setBusy(false);
  };
  return (
    <SectionCard
      label={target ? w.titleBot(bot.name, target, crew) : w.titleCrew(bot.name, crew)}
      icon={Network}
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
            <Button
              variant="primary"
              icon={Check}
              disabled={busy}
              onClick={() => answer(true, "once")}
            >
              {w.once}
            </Button>
            {target && (
              <Button disabled={busy} onClick={() => answer(true, "bot")}>
                {w.alwaysBot(target)}
              </Button>
            )}
            <Button disabled={busy} onClick={() => answer(true, "crew")}>
              {w.alwaysCrew(crew)}
            </Button>
            <Button icon={Ban} disabled={busy} onClick={() => answer(false)}>
              {w.decline}
            </Button>
          </div>
        </>
      }
    >
      <div className="flex flex-col gap-2 px-4 py-3 text-sm">
        <ul className="flex flex-col gap-1">
          {(asked.access ?? []).map((kind) => (
            <li key={kind} className="text-ink-soft">
              {kind === "talk"
                ? w.talk(target ?? crew)
                : kind === "read"
                  ? w.read(target ?? crew)
                  : kind === "edit"
                    ? w.edit(target ?? crew)
                    : kind}
            </li>
          ))}
        </ul>
        {asked.why && (
          <p>
            <span className="text-muted">{w.why(bot.name)} </span>
            <span data-selectable>{asked.why}</span>
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
