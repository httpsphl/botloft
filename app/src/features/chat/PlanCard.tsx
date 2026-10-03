// A bot in plan mode asking to go ahead (spec 10.1): Claude Code asks for
// it as the ExitPlanMode tool, with the plan. Approving lets the bot work;
// sending it back keeps it planning, with what should change.

import { Check, ListTodo, PencilLine, TimerOff } from "lucide-react";
import { useT } from "../../i18n";
import type { ApprovalItem, Bot } from "../../lib/protocol.gen";
import { Button } from "../../ui/Button";
import { NoteArea, SectionCard, SettledLine } from "../../ui/ChatCard";
import { Markdown } from "./Markdown";
import { useAnswer } from "./useAnswer";

export const PLAN_TOOL = "ExitPlanMode";

/** The plan in the request's input; the input itself if it is not JSON. */
export function planText(input: string): string {
  try {
    const parsed: unknown = JSON.parse(input);
    if (typeof parsed === "object" && parsed !== null && "plan" in parsed) {
      const { plan } = parsed;
      if (typeof plan === "string") {
        return plan;
      }
    }
  } catch {
    // Cut short or not JSON: show it as it came.
  }
  return input;
}

export function PlanCard({ approval, bot }: { approval: ApprovalItem; bot: Bot }) {
  const p = useT().chat.plan;
  const { note, setNote, busy, answer } = useAnswer(approval, {
    allow: p.approveFailed,
    deny: p.keepFailed,
  });
  const plan = planText(approval.input);
  if (approval.status !== "pending") {
    return <Answered approval={approval} plan={plan} />;
  }

  return (
    <SectionCard
      label={p.ready(bot.name)}
      icon={ListTodo}
      tone="work"
      footer={
        <>
          <NoteArea
            label={p.noteLabel(bot.name)}
            value={note}
            onChange={setNote}
            placeholder={p.notePlaceholder}
          />
          <div className="mt-2.5 flex flex-wrap gap-2">
            <Button variant="primary" icon={Check} disabled={busy} onClick={() => answer(true)}>
              {p.approve}
            </Button>
            <Button icon={PencilLine} disabled={busy} onClick={() => answer(false)}>
              {p.keepPlanning}
            </Button>
          </div>
        </>
      }
    >
      <div className="max-h-[28rem] overflow-y-auto px-4 py-3" data-selectable>
        <Markdown text={plan} />
      </div>
    </SectionCard>
  );
}

/** One line once answered; the plan stays a click away. */
function Answered({ approval, plan }: { approval: ApprovalItem; plan: string }) {
  const p = useT().chat.plan;
  const [icon, text, tone] =
    approval.status === "allowed"
      ? ([Check, p.approved, "ok"] as const)
      : approval.status === "denied"
        ? ([PencilLine, p.sentBack, "work"] as const)
        : ([TimerOff, p.expired, "quiet"] as const);
  return (
    <SettledLine
      icon={icon}
      tone={tone}
      text={text}
      note={approval.note}
      details={<Markdown text={plan} />}
    />
  );
}
