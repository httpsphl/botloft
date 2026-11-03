// What the agent left running in the background (spec 33.4): commands,
// monitors and subagents, each with when it started and a Stop button.

import { Bot as AgentIcon, CircleDot, Cloud, type LucideIcon, SquareTerminal } from "lucide-react";
import { useId } from "react";
import { useT } from "../../i18n";
import { when } from "../../lib/format";
import type { BackgroundKind, BackgroundTask, Bot } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";
import { Button } from "../../ui/Button";
import { attempt } from "../../ui/toast";

const ICONS: Record<BackgroundKind, LucideIcon> = {
  command: SquareTerminal,
  agent: AgentIcon,
  remote: Cloud,
  other: CircleDot,
};

export function BackgroundSection({ bot }: { bot: Bot }) {
  const w = useT().overview.background;
  const id = useId();
  return (
    <section aria-labelledby={id} className="flex flex-col gap-3">
      <h3 id={id} className="font-semibold text-sm">
        {w.title}
      </h3>
      {bot.background.length === 0 ? (
        <p className="text-muted text-sm">{w.none}</p>
      ) : (
        <>
          <ul aria-label={w.title} className="flex flex-col gap-0.5 rounded-xl bg-bubble p-1.5">
            {bot.background.map((task) => (
              <Task key={task.taskId} bot={bot} task={task} />
            ))}
          </ul>
          <p className="text-muted text-xs">{w.hint(bot.name)}</p>
        </>
      )}
    </section>
  );
}

function Task({ bot, task }: { bot: Bot; task: BackgroundTask }) {
  const api = useApi();
  const w = useT().overview.background;
  const Icon = ICONS[task.kind];
  const what = task.description || w.kind[task.kind];
  return (
    <li className="flex items-start gap-2.5 rounded-lg px-2 py-1.5">
      <Icon aria-hidden size={15} className="mt-0.5 shrink-0 text-muted" />
      <span className="flex min-w-0 flex-1 flex-col">
        <span className="break-words text-sm leading-snug" data-selectable>
          {what}
        </span>
        <span className="text-muted text-xs">
          {w.kind[task.kind]} · {w.since(when(task.seenAt))}
        </span>
      </span>
      <Button
        size="sm"
        variant="ghost"
        aria-label={w.stopOne(what)}
        onClick={() =>
          attempt(w.stopFailed, () =>
            api.call("bots.stopTask", { botId: bot.id, taskId: task.taskId }),
          )
        }
      >
        {w.stop}
      </Button>
    </li>
  );
}
