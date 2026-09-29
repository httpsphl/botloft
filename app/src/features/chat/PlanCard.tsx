// A bot in plan mode asking to go ahead (spec 10.1): Claude Code asks for
// it as the ExitPlanMode tool, with the plan. Approving lets the bot work;
// sending it back keeps it planning, with what should change.

import { Check, ChevronRight, ListTodo, PencilLine, TimerOff } from "lucide-react";
import { useId } from "react";
import { useT } from "../../i18n";
import type { ApprovalItem, Bot } from "../../lib/protocol.gen";
import { Button } from "../../ui/Button";
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
  const noteId = useId();
  const plan = planText(approval.input);
  if (approval.status !== "pending") {
    return <Answered approval={approval} plan={plan} />;
  }

  return (
    <section
      aria-label={p.ready(bot.name)}
      className="my-1 overflow-hidden rounded-2xl border border-line bg-panel shadow-sm"
    >
      <p className="flex items-center gap-2.5 border-line border-b px-4 py-3 font-semibold text-sm">
        <span className="grid h-7 w-7 place-items-center rounded-full bg-work/12 text-work">
          <ListTodo aria-hidden size={15} />
        </span>
        {p.ready(bot.name)}
      </p>
      <div className="max-h-[28rem] overflow-y-auto px-4 py-3" data-selectable>
        <Markdown text={plan} />
      </div>
      <div className="border-line border-t bg-canvas/40 px-4 py-3">
        <label htmlFor={noteId} className="sr-only">
          {p.noteLabel(bot.name)}
        </label>
        <textarea
          id={noteId}
          rows={2}
          value={note}
          onChange={(event) => setNote(event.target.value)}
          placeholder={p.notePlaceholder}
          className="block w-full resize-none rounded-xl border border-line-strong bg-panel px-3 py-2 text-sm outline-none placeholder:text-muted focus:border-muted"
        />
        <div className="mt-2.5 flex flex-wrap gap-2">
          <Button variant="primary" icon={Check} disabled={busy} onClick={() => answer(true)}>
            {p.approve}
          </Button>
          <Button icon={PencilLine} disabled={busy} onClick={() => answer(false)}>
            {p.keepPlanning}
          </Button>
        </div>
      </div>
    </section>
  );
}

/** One line once answered; the plan stays a click away. */
function Answered({ approval, plan }: { approval: ApprovalItem; plan: string }) {
  const p = useT().chat.plan;
  const [Icon, text, tone] =
    approval.status === "allowed"
      ? [Check, p.approved, "text-ok"]
      : approval.status === "denied"
        ? [PencilLine, p.sentBack, "text-work"]
        : [TimerOff, p.expired, "text-quiet"];
  return (
    <details className="group">
      <summary className="flex min-w-0 cursor-default items-center gap-2 rounded-lg px-1.5 py-1 text-sm hover:bg-sunken">
        <Icon aria-hidden size={14} className={`shrink-0 ${tone}`} />
        <span className="shrink-0 font-medium">{text}</span>
        {approval.note && (
          <span className="truncate text-muted text-xs">{`“${approval.note}”`}</span>
        )}
        <ChevronRight
          aria-hidden
          size={13}
          className="shrink-0 text-muted transition-transform group-open:rotate-90"
        />
      </summary>
      <div className="mt-1.5 ml-6 rounded-xl border border-line bg-panel px-4 py-3" data-selectable>
        <Markdown text={plan} />
      </div>
    </details>
  );
}
