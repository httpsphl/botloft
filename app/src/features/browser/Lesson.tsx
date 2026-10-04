// Teaching the bot a task (spec 21.13): with the browser in the owner's
// hands, a lesson records what they do as steps by what it means; at its
// end they name the task and make it a routine, or send it for the bot to
// remember.

import { AlarmClock, Send, X } from "lucide-react";
import { type FormEvent, useState } from "react";
import { useT } from "../../i18n";
import type { Bot, BrowserState, LessonStep } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";
import { Button } from "../../ui/Button";
import { Dialog } from "../../ui/Dialog";
import { TextField } from "../../ui/Field";
import { attempt } from "../../ui/toast";
import { RoutineDialog } from "../routines/RoutineDialog";
import { lessonText, stepText } from "./lessonText";

export interface Lesson {
  /** The steps so far, while the owner teaches; `null` otherwise. */
  steps: LessonStep[] | null;
  start(): Promise<void>;
  /** Ends the lesson and opens what to do with it. */
  finish(): Promise<void>;
  cancel(): Promise<void>;
  /** The steps of the lesson just finished, until the owner is done with them. */
  done: LessonStep[] | null;
  close(): void;
}

export function useLesson(bot: Pick<Bot, "id">, state: BrowserState | null): Lesson {
  const api = useApi();
  const t = useT().browser.lesson;
  const [done, setDone] = useState<LessonStep[] | null>(null);
  const teach = async (on: boolean) => {
    let steps: LessonStep[] = [];
    const ok = await attempt(t.failed, async () => {
      steps = await api.call("browser.teach", { botId: bot.id, on });
    });
    return ok ? steps : null;
  };
  return {
    steps: state?.lesson ?? null,
    start: async () => {
      await teach(true);
    },
    finish: async () => {
      const steps = await teach(false);
      if (steps) {
        setDone(steps);
      }
    },
    cancel: async () => {
      await teach(false);
    },
    done,
    close: () => setDone(null),
  };
}

/** Under the owner's pill: the lesson being given, live. */
export function LessonArea({ lesson }: { lesson: Lesson }) {
  const t = useT().browser.lesson;
  if (lesson.steps === null) {
    return null;
  }
  return (
    <section
      aria-label={t.recording}
      className="mx-auto mt-3 flex w-full max-w-md animate-rise flex-col gap-2 rounded-xl border border-danger/40 bg-panel px-3.5 py-3"
    >
      <p className="flex items-center gap-2 font-medium text-sm">
        <span aria-hidden className="live-dot" />
        {t.recording}
      </p>
      {/* A fixed height: steps coming in never resize the page above. */}
      {lesson.steps.length === 0 ? (
        <p className="h-16 text-muted text-xs">{t.empty}</p>
      ) : (
        <ol className="flex h-16 flex-col gap-1 overflow-y-auto text-ink-soft text-xs">
          {lesson.steps.map((step, index) => (
            // biome-ignore lint/suspicious/noArrayIndexKey: steps only grow at the end
            <li key={index} className="animate-rise">
              {index + 1}. {stepText(step, t)}
            </li>
          ))}
        </ol>
      )}
      {/* The pill above finishes it; here it can be dropped. */}
      <div className="flex">
        <Button size="sm" variant="ghost" onClick={() => void lesson.cancel()}>
          {t.cancel}
        </Button>
      </div>
    </section>
  );
}

/** What to do with the lesson just finished: a routine, or a message. */
export function LessonDialog({ bot, lesson }: { bot: Bot; lesson: Lesson }) {
  const t = useT().browser.lesson;
  const api = useApi();
  const [name, setName] = useState("");
  const [steps, setSteps] = useState(lesson.done ?? []);
  const [routine, setRoutine] = useState(false);
  const ready = name.trim().length > 0 && steps.length > 0;
  const text = lessonText(name, steps, t);

  const send = async (event?: FormEvent) => {
    event?.preventDefault();
    if (!ready) return;
    const body = `${text}\n\n${t.forMemory(name.trim())}`;
    // It shows in the bot's chat, beside the panel.
    if (await attempt(t.sendFailed, () => api.call("messages.send", { botId: bot.id, body }))) {
      lesson.close();
    }
  };

  if (routine) {
    return (
      <RoutineDialog
        bot={bot}
        start={{ name: name.trim(), prompt: `${text}\n\n${t.forRoutine}` }}
        onClose={lesson.close}
      />
    );
  }
  return (
    <Dialog title={t.title(bot.name)} onClose={lesson.close}>
      <form onSubmit={send} className="flex flex-col gap-4">
        <TextField
          label={t.name}
          value={name}
          placeholder={t.namePlaceholder}
          onChange={(event) => setName(event.target.value)}
          autoFocus
        />
        <div>
          <p className="mb-1.5 font-medium text-sm">{t.steps}</p>
          <ol className="flex max-h-64 flex-col gap-1 overflow-y-auto">
            {steps.map((step, index) => (
              // biome-ignore lint/suspicious/noArrayIndexKey: a list the owner only shortens
              <li key={index} className="flex items-center gap-2 text-ink-soft text-sm">
                <span className="w-5 shrink-0 text-right text-muted text-xs">{index + 1}.</span>
                <span className="min-w-0 flex-1 truncate" title={stepText(step, t)}>
                  {stepText(step, t)}
                </span>
                <Button
                  size="sm"
                  variant="ghost"
                  icon={X}
                  label={t.remove(index + 1)}
                  onClick={() => setSteps(steps.filter((_, at) => at !== index))}
                />
              </li>
            ))}
          </ol>
        </div>
        <div className="flex flex-wrap justify-end gap-2">
          <Button icon={Send} type="submit" disabled={!ready}>
            {t.sendToBot(bot.name)}
          </Button>
          <Button
            variant="primary"
            icon={AlarmClock}
            disabled={!ready}
            onClick={() => setRoutine(true)}
          >
            {t.makeRoutine}
          </Button>
        </div>
      </form>
    </Dialog>
  );
}
