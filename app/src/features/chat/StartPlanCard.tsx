// The plan a bot shows before long work (spec 10.5): it starts by itself
// after a few seconds unless the owner starts it now, picks the smaller
// way, changes it or cancels it.

import { Ban, Check, ListChecks, PencilLine, Play, Shrink, TimerReset } from "lucide-react";
import { useEffect, useState } from "react";
import { useT } from "../../i18n";
import type { ApprovalItem, Bot } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";
import { Button } from "../../ui/Button";
import { NoteArea, SectionCard, SettledLine } from "../../ui/ChatCard";
import { attempt } from "../../ui/toast";

export const START_PLAN_TOOL = "mcp__botloft__show_plan";

export interface StartPlan {
  summary: string;
  steps: string[];
  alternative: string | null;
  /** When it starts by itself, in Unix ms. */
  startsAt: number;
  /** Set once the owner picked the smaller way. */
  choice: "alternative" | null;
}

/** The plan in the request's input; `null` if it was cut or is not one. */
export function startPlanOf(input: string): StartPlan | null {
  try {
    const parsed = JSON.parse(input) as Record<string, unknown>;
    const steps = parsed.steps;
    if (typeof parsed.summary !== "string" || !Array.isArray(steps)) {
      return null;
    }
    return {
      summary: parsed.summary,
      steps: steps.filter((step): step is string => typeof step === "string"),
      alternative: typeof parsed.alternative === "string" ? parsed.alternative : null,
      startsAt: typeof parsed.starts_at === "number" ? parsed.starts_at : 0,
      choice: parsed.choice === "alternative" ? "alternative" : null,
    };
  } catch {
    return null;
  }
}

/** Whole seconds until `at`, counted down every second. */
function useSecondsUntil(at: number): number {
  const left = () => Math.max(0, Math.ceil((at - Date.now()) / 1000));
  const [seconds, setSeconds] = useState(left);
  useEffect(() => {
    if (seconds === 0) {
      return;
    }
    const timer = window.setTimeout(() => setSeconds(left()), 1000);
    return () => window.clearTimeout(timer);
  });
  return seconds;
}

function Plan({ plan }: { plan: StartPlan }) {
  const p = useT().chat.startPlan;
  return (
    <div className="flex flex-col gap-2 px-4 py-3 text-sm" data-selectable>
      <p className="font-medium">{plan.summary}</p>
      <ol className="list-decimal space-y-1 pl-5 text-ink-soft">
        {plan.steps.map((step, index) => (
          // Steps can repeat; their place is what tells them apart.
          // biome-ignore lint/suspicious/noArrayIndexKey: fixed list
          <li key={index}>{step}</li>
        ))}
      </ol>
      {plan.alternative && (
        <p className="rounded-lg bg-sunken px-3 py-2 text-ink-soft">
          <span className="font-medium text-ink">{p.alternative}: </span>
          {plan.alternative}
        </p>
      )}
    </div>
  );
}

export function StartPlanCard({ approval, bot }: { approval: ApprovalItem; bot: Bot }) {
  const p = useT().chat.startPlan;
  const api = useApi();
  const plan = startPlanOf(approval.input);
  const seconds = useSecondsUntil(plan?.startsAt ?? 0);
  const [changing, setChanging] = useState(false);
  const [note, setNote] = useState("");
  const [busy, setBusy] = useState(false);
  if (!plan) {
    return null;
  }
  if (approval.status !== "pending") {
    return <Answered approval={approval} plan={plan} />;
  }

  const answer = async (allow: boolean, extra: { note?: string; input?: string } = {}) => {
    setBusy(true);
    await attempt(p.failed, () =>
      api.call("approvals.answer", { approvalId: approval.approvalId, allow, ...extra }),
    );
    setBusy(false);
  };
  const change = () => {
    const trimmed = note.trim();
    if (trimmed) {
      void answer(false, { note: trimmed });
    }
  };

  return (
    <SectionCard
      label={p.ready(bot.name)}
      icon={ListChecks}
      tone="work"
      footer={
        <>
          {changing && (
            <div className="mb-2.5">
              <NoteArea
                label={p.noteLabel(bot.name)}
                value={note}
                onChange={setNote}
                onEnter={change}
                placeholder={p.notePlaceholder}
              />
            </div>
          )}
          <div className="flex flex-wrap items-center gap-2">
            {changing ? (
              <Button
                variant="primary"
                icon={PencilLine}
                disabled={busy || !note.trim()}
                onClick={change}
              >
                {p.sendChange}
              </Button>
            ) : (
              <>
                <Button variant="primary" icon={Play} disabled={busy} onClick={() => answer(true)}>
                  {p.startNow}
                </Button>
                {plan.alternative && (
                  <Button
                    icon={Shrink}
                    disabled={busy}
                    onClick={() => answer(true, { input: '{"choice":"alternative"}' })}
                  >
                    {p.onlyThis}
                  </Button>
                )}
                <Button icon={PencilLine} disabled={busy} onClick={() => setChanging(true)}>
                  {p.change}
                </Button>
              </>
            )}
            <Button icon={Ban} disabled={busy} onClick={() => answer(false)}>
              {p.cancel}
            </Button>
            {!changing && (
              <span className="ml-auto text-muted text-xs tabular-nums" aria-live="off">
                {seconds > 0 ? p.startsIn(seconds) : p.starting}
              </span>
            )}
          </div>
        </>
      }
    >
      <Plan plan={plan} />
    </SectionCard>
  );
}

/** One line once settled; the plan stays a click away. */
function Answered({ approval, plan }: { approval: ApprovalItem; plan: StartPlan }) {
  const p = useT().chat.startPlan;
  const [icon, text, tone] =
    approval.status === "allowed"
      ? plan.choice === "alternative"
        ? ([Shrink, p.didAlternative, "ok"] as const)
        : ([Check, p.started, "ok"] as const)
      : approval.status === "denied"
        ? approval.note
          ? ([PencilLine, p.changed, "work"] as const)
          : ([Ban, p.cancelled, "quiet"] as const)
        : ([TimerReset, p.startedAlone, "ok"] as const);
  return (
    <SettledLine
      icon={icon}
      tone={tone}
      text={text}
      note={approval.note}
      details={<Plan plan={plan} />}
    />
  );
}
