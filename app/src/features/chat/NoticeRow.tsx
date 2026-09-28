// Something the daemon tells the owner in the chat: a usage limit, a
// sign-in problem, a session that started over (spec 8.2).

import { CircleAlert, Info, TriangleAlert } from "lucide-react";
import type { NoticeItem } from "../../lib/protocol.gen";

const LOOK = {
  info: { icon: Info, tone: "text-work", frame: "border-line" },
  warning: { icon: TriangleAlert, tone: "text-warn", frame: "border-warn/50" },
  error: { icon: CircleAlert, tone: "text-danger", frame: "border-danger/50" },
} as const;

export function NoticeRow({ notice }: { notice: NoticeItem }) {
  const { icon: Icon, tone, frame } = LOOK[notice.level];
  return (
    <li
      role={notice.level === "info" ? undefined : "alert"}
      className={`mx-auto flex max-w-2xl items-start gap-2 rounded-md border bg-panel px-3 py-2 text-sm ${frame}`}
    >
      <Icon aria-hidden size={15} className={`mt-0.5 shrink-0 ${tone}`} />
      <p className="whitespace-pre-wrap break-words text-ink-soft" data-selectable>
        {notice.text}
      </p>
    </li>
  );
}
