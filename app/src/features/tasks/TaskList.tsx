import {
  ArrowRight,
  CircleCheck,
  CircleDashed,
  CircleSlash,
  CircleX,
  Hourglass,
  type LucideIcon,
} from "lucide-react";
import { useState } from "react";
import { useShallow } from "zustand/react/shallow";
import { fromNow, when } from "../../lib/format";
import type { Bot, CrewId, Task, TaskStatus } from "../../lib/protocol.gen";
import { tasksOf } from "../../store/app";
import { useApp } from "../../store/context";
import { BotAvatar } from "../bots/BotAvatar";

const STATUS: Record<TaskStatus, { label: string; icon: LucideIcon; tone: string }> = {
  open: { label: "Open", icon: CircleDashed, tone: "text-work" },
  done: { label: "Done", icon: CircleCheck, tone: "text-ok" },
  failed: { label: "Failed", icon: CircleX, tone: "text-danger" },
  cancelled: { label: "Cancelled", icon: CircleSlash, tone: "text-quiet" },
  expired: { label: "Expired", icon: Hourglass, tone: "text-warn" },
};

function Handle({ bot }: { bot: Bot | undefined }) {
  if (!bot) {
    return <span className="text-muted">an archived bot</span>;
  }
  return (
    <span className="inline-flex items-center gap-1.5 font-medium">
      <BotAvatar color={bot.color} size={14} />
      {bot.name}
    </span>
  );
}

type Show = "open" | "all";

/** Tasks between the crew's bots (spec 9.4), newest first. */
export function TaskList({ crewId }: { crewId: CrewId }) {
  const tasks = useApp(useShallow((state) => tasksOf(state, crewId)));
  const bots = useApp((state) => state.bots);
  const [show, setShow] = useState<Show>("open");
  const shown = show === "open" ? tasks.filter((task) => task.status === "open") : tasks;

  return (
    <div className="min-h-0 flex-1 overflow-y-auto">
      <fieldset className="flex items-center gap-1 px-5 pt-4 pb-2">
        <legend className="sr-only">Show</legend>
        {(["open", "all"] as const).map((option) => (
          <button
            key={option}
            type="button"
            aria-pressed={show === option}
            onClick={() => setShow(option)}
            className={`h-7 rounded-[3px] px-2.5 font-medium text-sm ${show === option ? "bg-ink text-canvas" : "text-ink-soft hover:bg-sunken"}`}
          >
            {option === "open" ? "Open" : "All"}
          </button>
        ))}
      </fieldset>
      {shown.length === 0 ? (
        <p className="px-5 py-3 text-muted text-sm">
          {show === "open"
            ? "No open tasks. Bots create tasks for each other with send_message."
            : "No tasks yet. Bots create tasks for each other with send_message."}
        </p>
      ) : (
        <ul aria-label="Tasks" className="px-5 pb-5">
          {shown.map((task) => (
            <TaskRow key={task.id} task={task} bots={bots} />
          ))}
        </ul>
      )}
    </div>
  );
}

function TaskRow({ task, bots }: { task: Task; bots: Record<string, Bot> }) {
  const status = STATUS[task.status];
  const Icon = status.icon;
  const overdue = task.status === "open" && task.deadlineAt < Date.now();
  return (
    <li className="border-line border-b py-3 last:border-b-0">
      <div className="flex items-center gap-2 text-sm">
        <span className={`inline-flex w-24 shrink-0 items-center gap-1 font-medium ${status.tone}`}>
          <Icon aria-hidden size={13} />
          {status.label}
        </span>
        <Handle bot={bots[task.requesterBotId]} />
        <ArrowRight aria-label="asked" size={12} className="text-muted" />
        <Handle bot={bots[task.assigneeBotId]} />
        {task.hops > 1 && (
          <span className="text-muted text-xs" title="Position in a chain of delegations">
            hop {task.hops}
          </span>
        )}
        <span className={`ml-auto shrink-0 text-xs ${overdue ? "text-danger" : "text-muted"}`}>
          {task.status === "open"
            ? `${overdue ? "overdue, was due" : "due"} ${fromNow(task.deadlineAt)}`
            : `${status.label.toLowerCase()} ${when(task.updatedAt)}`}
        </span>
      </div>
      {task.result && (
        <p
          className="mt-1 ml-26 line-clamp-4 whitespace-pre-wrap break-words text-ink-soft text-sm"
          data-selectable
        >
          {task.result}
        </p>
      )}
    </li>
  );
}
