// The phone's Home (spec 28.15), where the page opens: the owner's shortcuts,
// what waits for them, who works now and what the crews did today, as on
// the computer's home (15.1).

import { Hand, Smartphone } from "lucide-react";
import type { ReactNode } from "react";
import { useT } from "../i18n";
import { duration } from "../lib/format";
import type { ChatLine, DayActivity, PhoneShortcut } from "../lib/protocol.gen";
import { ApprovalView } from "./ApprovalView";
import type { PhoneApi, PhoneState } from "./client";
import { FailedRuns } from "./FailedRuns";
import { BotDot } from "./parts";
import { QuestionView } from "./QuestionView";

const busy = (line: ChatLine) => line.state === "busy" || line.state === "launching";

export function HomeTab({
  api,
  state,
  waiting,
  open,
}: {
  api: PhoneApi;
  state: PhoneState;
  /** How many things wait for the owner. */
  waiting: number;
  /** Opens an agent's conversation. */
  open(botId: string): void;
}) {
  const t = useT();
  const working = state.chats.filter(busy);
  return (
    <div className="flex flex-col gap-6">
      {state.shortcuts.length > 0 && (
        <Part id="phone-shortcuts" title={t.routines.shortcuts.title}>
          <Shortcuts api={api} state={state} />
        </Part>
      )}
      <section aria-labelledby="phone-needs-you" className="flex flex-col gap-4">
        <h2
          id="phone-needs-you"
          className="flex items-center gap-1.5 font-semibold text-sm text-warn"
        >
          <Hand aria-hidden size={14} />
          {t.home.needsYou}
        </h2>
        {state.approvals.map((card) => (
          <ApprovalView
            key={card.approvalId}
            card={card}
            sending={state.sending.includes(card.approvalId)}
            answer={(allow, note) => api.answerApproval(card.approvalId, allow, note)}
          />
        ))}
        {state.questions.map((card) => (
          <QuestionView
            key={card.questionId}
            card={card}
            sending={state.sending.includes(card.questionId)}
            answer={(text) => api.answerQuestion(card.questionId, text)}
            dismiss={() => api.dismissQuestion(card.questionId)}
          />
        ))}
        <FailedRuns chats={state.chats} seen={state.seen} open={open} />
        {waiting === 0 && <Nothing state={state} />}
      </section>
      {working.length > 0 && (
        <Part id="phone-working" title={t.home.working}>
          <ul className="flex flex-col overflow-hidden rounded-2xl border border-line bg-panel">
            {working.map((line) => (
              <li key={line.botId} className="border-line border-b last:border-b-0">
                <button
                  type="button"
                  onClick={() => open(line.botId)}
                  className="flex min-h-14 w-full items-center gap-3 px-3 py-2 text-left"
                >
                  <BotDot name={line.name} color={line.color} state={line.state} size={30} />
                  <span className="flex min-w-0 flex-col">
                    <span className="truncate font-medium">{line.name}</span>
                    <span className="truncate text-ink-soft text-sm">
                      {line.last?.text ?? t.phone.chats.working}
                    </span>
                  </span>
                </button>
              </li>
            ))}
          </ul>
        </Part>
      )}
      {state.today && (
        <Part id="phone-today" title={t.home.today.title}>
          <Today day={state.today} />
        </Part>
      )}
    </div>
  );
}

function Part({ id, title, children }: { id: string; title: string; children: ReactNode }) {
  return (
    <section aria-labelledby={id} className="flex flex-col gap-3">
      <h2 id={id} className="font-semibold text-muted text-sm">
        {title}
      </h2>
      {children}
    </section>
  );
}

function Nothing({ state }: { state: PhoneState }) {
  const t = useT().phone.inbox;
  return (
    <div className="flex flex-col items-center gap-2 py-8 text-center">
      <Smartphone aria-hidden size={28} className="text-muted" />
      {state.loaded || state.computer === "offline" ? (
        <>
          <p className="font-medium text-base">{t.empty}</p>
          <p className="text-muted text-sm">{t.emptyBody}</p>
        </>
      ) : (
        <p className="text-muted text-sm">{t.loading}</p>
      )}
    </div>
  );
}

/** One tap runs the routine; the line under them says how the last one went. */
function Shortcuts({ api, state }: { api: PhoneApi; state: PhoneState }) {
  const t = useT();
  const words = t.routines.shortcuts;
  const botOf = (shortcut: PhoneShortcut) =>
    state.chats.find((line) => line.botId === shortcut.botId);
  const [routineId, run] = Object.entries(state.runs).at(-1) ?? [];
  const tapped = state.shortcuts.find((shortcut) => shortcut.routineId === routineId);
  const word =
    run === "running"
      ? t.phone.home.running
      : run === "started" && tapped
        ? words.started(botOf(tapped)?.name ?? "", tapped.name)
        : run === "failed"
          ? words.failed
          : "";
  return (
    <>
      <ul className="flex flex-wrap gap-2">
        {state.shortcuts.map((shortcut) => {
          const bot = botOf(shortcut);
          return (
            <li key={shortcut.routineId} className="max-w-full">
              <button
                type="button"
                aria-label={words.run(shortcut.name, bot?.name ?? "")}
                disabled={state.runs[shortcut.routineId] === "running"}
                onClick={() => void api.runShortcut(shortcut.routineId)}
                className="inline-flex h-12 max-w-full items-center gap-2 rounded-xl border border-line-strong bg-panel px-3 font-medium text-base active:scale-[0.99] disabled:opacity-60"
              >
                {bot && <BotDot name={bot.name} color={bot.color} state={bot.state} size={24} />}
                <span className="truncate">{shortcut.name}</span>
              </button>
            </li>
          );
        })}
      </ul>
      <p role="status" className={`text-sm ${run === "failed" ? "text-danger" : "text-muted"}`}>
        {word}
      </p>
    </>
  );
}

/** The day's numbers, as on the computer's home. */
function Today({ day }: { day: DayActivity }) {
  const t = useT().home.today;
  const quiet =
    day.tasksDone + day.tasksFailed + day.botMessages + day.requestsAnswered + day.filesEdited ===
      0 && !day.busiest;
  if (quiet) {
    return <p className="text-muted text-sm">{t.nothing}</p>;
  }
  return (
    <dl className="grid grid-cols-2 gap-2">
      <Tile label={t.tasks}>
        {day.tasksDone}
        {day.tasksFailed > 0 && (
          <span className="ml-1.5 font-normal text-danger text-xs">
            {t.failed(day.tasksFailed)}
          </span>
        )}
      </Tile>
      <Tile label={t.requests}>{day.requestsAnswered}</Tile>
      <Tile label={t.files}>{day.filesEdited}</Tile>
      <Tile label={t.messages}>{day.botMessages}</Tile>
      {day.busiest && (
        <Tile label={t.busiest} wide>
          <span className="truncate">{day.busiest.name}</span>
          <span className="ml-1.5 font-normal text-muted text-xs">
            {duration(day.busiest.workMs)}
          </span>
        </Tile>
      )}
    </dl>
  );
}

function Tile({ label, wide, children }: { label: string; wide?: boolean; children: ReactNode }) {
  return (
    <div
      className={`min-w-0 rounded-xl border border-line bg-panel px-3.5 py-2.5 ${wide ? "col-span-2" : ""}`}
    >
      <dt className="text-muted text-xs">{label}</dt>
      <dd className="mt-0.5 flex items-baseline font-semibold text-lg tabular-nums">{children}</dd>
    </div>
  );
}
