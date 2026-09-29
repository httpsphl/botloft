// A bot asking to use a site in its browser (spec 21.5): the site, the
// address, and Allow or Deny. Once answered it shrinks to one line.

import { Ban, Check, Globe, TimerOff } from "lucide-react";
import { useId } from "react";
import { useT } from "../../i18n";
import type { ApprovalItem, Bot } from "../../lib/protocol.gen";
import { Button } from "../../ui/Button";
import { useAnswer } from "../chat/useAnswer";

/** The request the daemon opens from the browser tools. */
export const SITE_TOOL = "mcp__botloft__browser";

function addressOf(input: string): string | null {
  try {
    const parsed: unknown = JSON.parse(input);
    const url = (parsed as { url?: unknown }).url;
    return typeof url === "string" ? url : null;
  } catch {
    return null;
  }
}

function Answered({ approval, site }: { approval: ApprovalItem; site: string }) {
  const words = useT().browser.site;
  const line = "flex min-w-0 items-center gap-2 rounded-lg px-1.5 py-1 text-sm";
  switch (approval.status) {
    case "allowed":
      return (
        <p className={line}>
          <Check aria-hidden size={14} className="shrink-0 text-ok" />
          <span className="truncate font-medium">{words.allowed(site)}</span>
        </p>
      );
    case "denied":
      return (
        <p className={line}>
          <Ban aria-hidden size={14} className="shrink-0 text-danger" />
          <span className="shrink-0 font-medium">{words.denied(site)}</span>
          {approval.note && <span className="truncate text-muted text-xs">“{approval.note}”</span>}
        </p>
      );
    default:
      return (
        <p className={line}>
          <TimerOff aria-hidden size={14} className="shrink-0 text-quiet" />
          <span className="font-medium">{words.expired(site)}</span>
        </p>
      );
  }
}

export function SiteCard({ approval, bot }: { approval: ApprovalItem; bot: Bot }) {
  const t = useT();
  const words = t.browser.site;
  const { note, setNote, busy, answer } = useAnswer(approval, {
    allow: t.chat.approval.allowFailed,
    deny: t.chat.approval.denyFailed,
  });
  const noteId = useId();
  const site = approval.summary;
  if (approval.status !== "pending") {
    return <Answered approval={approval} site={site} />;
  }
  const url = addressOf(approval.input);

  return (
    <section
      aria-label={words.asks(bot.name, site)}
      className="my-1 max-w-2xl rounded-2xl border border-warn/40 bg-panel px-4 py-3.5 shadow-sm animate-attention"
    >
      <p className="flex items-center gap-2.5 font-semibold text-sm">
        <span className="grid h-7 w-7 shrink-0 place-items-center rounded-full bg-warn/12 text-warn">
          <Globe aria-hidden size={15} />
        </span>
        <span>
          {words.wants(bot.name)} <span className="font-mono">{site}</span>
        </span>
      </p>
      {url && (
        <p className="mt-2 break-all font-mono text-ink-soft text-xs" data-selectable>
          {url}
        </p>
      )}
      <p className="mt-1.5 text-muted text-xs">{words.why}</p>
      <label htmlFor={noteId} className="sr-only">
        {t.chat.approval.noteLabel(bot.name)}
      </label>
      <input
        id={noteId}
        value={note}
        onChange={(event) => setNote(event.target.value)}
        placeholder={t.chat.approval.notePlaceholder}
        className="mt-3 h-9 w-full rounded-xl border border-line-strong bg-canvas px-3 text-sm outline-none placeholder:text-muted focus:border-muted"
      />
      <div className="mt-2.5 flex gap-2">
        <Button variant="primary" icon={Check} disabled={busy} onClick={() => answer(true)}>
          {t.chat.approval.allow}
        </Button>
        <Button variant="danger" icon={Ban} disabled={busy} onClick={() => answer(false)}>
          {t.chat.approval.deny}
        </Button>
      </div>
    </section>
  );
}
