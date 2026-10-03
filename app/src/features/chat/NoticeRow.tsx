// Something the daemon tells the owner in the chat: a usage limit, a
// sign-in problem, a session that started over (spec 8.2).

import { CircleAlert, Info, TriangleAlert } from "lucide-react";
import { memo } from "react";
import { type Messages, useT } from "../../i18n";
import type { NoticeItem } from "../../lib/protocol.gen";
import { useArrival } from "../../ui/motion";
import { TONES } from "../../ui/tone";

const LOOK = {
  info: { icon: Info, tone: TONES.work.text, frame: TONES.quiet.frame },
  warning: { icon: TriangleAlert, tone: TONES.warn.text, frame: TONES.warn.frame },
  error: { icon: CircleAlert, tone: TONES.danger.text, frame: TONES.danger.frame },
} as const;

/** `at` is when the notice came, so a new one animates in. */
export const NoticeRow = memo(function NoticeRow({
  notice,
  at = 0,
}: {
  notice: NoticeItem;
  at?: number;
}) {
  const t = useT();
  const { icon: Icon, tone, frame } = LOOK[notice.level];
  const arrival = useArrival(at);
  return (
    <li
      role={notice.level === "info" ? undefined : "alert"}
      className={`mx-auto flex max-w-2xl items-start gap-2.5 rounded-xl border px-3.5 py-2.5 text-sm ${frame} ${arrival}`}
    >
      <Icon aria-hidden size={15} className={`mt-0.5 shrink-0 ${tone}`} />
      <p className="whitespace-pre-wrap break-words text-ink-soft" data-selectable>
        {noticeText(notice, t.chat.notice)}
      </p>
    </li>
  );
});

/** The app's own words for notices it knows; the daemon's text otherwise. */
function noticeText(notice: NoticeItem, words: Messages["chat"]["notice"]): string {
  switch (notice.code) {
    case "signed_out":
      return words.signedOut;
    case "usage_limit":
      return words.usageLimit;
    case "turn_failed":
      return words.turnFailed(notice.text);
    case "model_unavailable":
      return words.modelUnavailable;
    case "compacted":
      return words.compacted;
    case "auto_compacted":
      return words.autoCompacted;
    case "compact_failed":
      return words.compactFailed(notice.text);
    default:
      return notice.text;
  }
}
