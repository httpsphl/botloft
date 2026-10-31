// The buttons under a request card (spec 10.1). Until the owner denies,
// the card is only the request and its answers. Deny first asks why: a
// field opens for what the bot should hear, Enter denies with or without
// it, and Esc or Back goes back to the answers.

import { Ban } from "lucide-react";
import { type ReactNode, useState } from "react";
import { useT } from "../../i18n";
import { Button } from "../../ui/Button";
import { NoteInput } from "../../ui/ChatCard";

export function RequestAnswers({
  bot,
  deny,
  busy,
  note,
  onNote,
  onDeny,
  children,
}: {
  /** Who reads the note. */
  bot: string;
  /** The deny button's words: "Deny", "I won't do it". */
  deny: string;
  busy: boolean;
  note: string;
  onNote(value: string): void;
  onDeny(): void;
  /** The answers before Deny. */
  children: ReactNode;
}) {
  const t = useT().chat.approval;
  const [asking, setAsking] = useState(false);
  if (!asking) {
    return (
      <div className="mt-3 flex flex-wrap gap-2">
        {children}
        <Button variant="danger" icon={Ban} disabled={busy} onClick={() => setAsking(true)}>
          {deny}
        </Button>
      </div>
    );
  }
  const back = () => {
    onNote("");
    setAsking(false);
  };
  return (
    <div className="animate-rise">
      <NoteInput
        label={t.noteLabel(bot)}
        value={note}
        onChange={onNote}
        placeholder={t.notePlaceholder}
        onEnter={() => !busy && onDeny()}
        onEscape={back}
      />
      <div className="mt-2.5 flex flex-wrap gap-2">
        <Button variant="danger" icon={Ban} disabled={busy} onClick={onDeny}>
          {deny}
        </Button>
        <Button variant="ghost" disabled={busy} onClick={back}>
          {t.back}
        </Button>
      </div>
    </div>
  );
}
