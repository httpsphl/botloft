// A bot asking to use a site in its browser (spec 21.5): the site, the
// address, and Allow or Deny. Once answered it shrinks to one line.

import { Ban, Check, Globe, TimerOff } from "lucide-react";
import { useT } from "../../i18n";
import type { ApprovalItem, Bot } from "../../lib/protocol.gen";
import { Button } from "../../ui/Button";
import { NoteInput, RequestCard, SettledLine } from "../../ui/ChatCard";
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
  switch (approval.status) {
    case "allowed":
      return <SettledLine icon={Check} tone="ok" text={words.allowed(site)} />;
    case "denied":
      return (
        <SettledLine icon={Ban} tone="danger" text={words.denied(site)} note={approval.note} />
      );
    default:
      return <SettledLine icon={TimerOff} tone="quiet" text={words.expired(site)} />;
  }
}

export function SiteCard({ approval, bot }: { approval: ApprovalItem; bot: Bot }) {
  const t = useT();
  const words = t.browser.site;
  const { note, setNote, busy, answer } = useAnswer(approval, {
    allow: t.chat.approval.allowFailed,
    deny: t.chat.approval.denyFailed,
  });
  const site = approval.summary;
  if (approval.status !== "pending") {
    return <Answered approval={approval} site={site} />;
  }
  const url = addressOf(approval.input);

  return (
    <RequestCard
      label={words.asks(bot.name, site)}
      icon={Globe}
      tone="warn"
      title={
        <span>
          {words.wants(bot.name)} <span className="font-mono">{site}</span>
        </span>
      }
    >
      {url && (
        <p className="mt-2 break-all font-mono text-ink-soft text-xs" data-selectable>
          {url}
        </p>
      )}
      <p className="mt-1.5 text-muted text-xs">{words.why}</p>
      <NoteInput
        label={t.chat.approval.noteLabel(bot.name)}
        value={note}
        onChange={setNote}
        placeholder={t.chat.approval.notePlaceholder}
      />
      <div className="mt-2.5 flex gap-2">
        <Button variant="primary" icon={Check} disabled={busy} onClick={() => answer(true)}>
          {t.chat.approval.allow}
        </Button>
        <Button variant="danger" icon={Ban} disabled={busy} onClick={() => answer(false)}>
          {t.chat.approval.deny}
        </Button>
      </div>
    </RequestCard>
  );
}
