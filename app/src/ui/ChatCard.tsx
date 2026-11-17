// The cards a bot puts in front of the owner in its chat (spec 15.1):
// permission requests, plans, bot and routine suggestions, questions.
// Two shapes share one look, a gray block like the bot's balloons: a
// compact request, and a card in sections (head, body, a footer with the
// owner's note). Once answered, a card shrinks to one settled line.

import { ChevronRight, type LucideIcon } from "lucide-react";
import { type ReactNode, useId } from "react";
import { TONES, type Tone } from "./tone";

/** The round icon at the head of a card. */
export function IconBadge({ icon: Icon, tone }: { icon: LucideIcon; tone: Tone }) {
  return (
    <span className={`grid h-7 w-7 shrink-0 place-items-center rounded-full ${TONES[tone].soft}`}>
      <Icon aria-hidden size={15} />
    </span>
  );
}

interface CardHead {
  /** What a screen reader calls the card. */
  label: string;
  icon: LucideIcon;
  tone: Tone;
  /** The line beside the icon; `label` when absent. */
  title?: ReactNode;
}

/** A short request, its icon in the card's tone: a command, a site. */
export function RequestCard({
  label,
  icon,
  tone,
  title,
  children,
}: CardHead & { children: ReactNode }) {
  return (
    <section
      aria-label={label}
      className="my-1 max-w-2xl rounded-bubble bg-bubble px-4 py-3.5 animate-attention"
    >
      <p className="flex items-center gap-2.5 font-semibold text-sm">
        <IconBadge icon={icon} tone={tone} />
        {title ?? label}
      </p>
      {children}
    </section>
  );
}

/**
 * A longer card: the head, the body as the caller lays it out, and a
 * footer, usually the owner's note and the buttons.
 */
export function SectionCard({
  label,
  icon,
  tone,
  title,
  children,
  footer,
}: CardHead & { children: ReactNode; footer?: ReactNode }) {
  return (
    <section
      aria-label={label}
      className="my-1 overflow-hidden rounded-bubble bg-bubble animate-attention"
    >
      <p className="flex items-center gap-2.5 px-4 pt-3.5 font-semibold text-sm">
        <IconBadge icon={icon} tone={tone} />
        {title ?? label}
      </p>
      {children}
      {footer && <div className="px-4 pt-1 pb-3.5">{footer}</div>}
    </section>
  );
}

interface NoteProps {
  /** Said to screen readers; the field shows `placeholder`. */
  label: string;
  value: string;
  placeholder: string;
  onChange(value: string): void;
}

const NOTE =
  "rounded-xl border border-line-strong bg-panel px-3 text-sm outline-none transition-colors placeholder:text-muted focus:border-muted";

/**
 * One line for the owner's note, in a request card. It opens when the owner
 * asks for it, so it takes the typing at once.
 */
export function NoteInput({
  label,
  value,
  placeholder,
  onChange,
  onEnter,
  onEscape,
}: NoteProps & { onEnter: () => void; onEscape: () => void }) {
  const id = useId();
  return (
    <>
      <label htmlFor={id} className="sr-only">
        {label}
      </label>
      <input
        id={id}
        value={value}
        onChange={(event) => onChange(event.target.value)}
        onKeyDown={(event) => {
          if (event.key === "Enter") {
            event.preventDefault();
            onEnter();
          } else if (event.key === "Escape") {
            event.preventDefault();
            onEscape();
          }
        }}
        placeholder={placeholder}
        // biome-ignore lint/a11y/noAutofocus: it opens on the owner's click
        autoFocus
        className={`mt-3 h-9 w-full ${NOTE}`}
      />
    </>
  );
}

/** Two lines for the owner's note, in a card's footer. */
export function NoteArea({
  label,
  value,
  placeholder,
  onChange,
  onEnter,
}: NoteProps & {
  /** Enter without Shift sends; Shift+Enter starts a new line. */
  onEnter?: () => void;
}) {
  const id = useId();
  return (
    <>
      <label htmlFor={id} className="sr-only">
        {label}
      </label>
      <textarea
        id={id}
        rows={2}
        value={value}
        onChange={(event) => onChange(event.target.value)}
        onKeyDown={(event) => {
          if (onEnter && event.key === "Enter" && !event.shiftKey) {
            event.preventDefault();
            onEnter();
          }
        }}
        placeholder={placeholder}
        className={`block w-full resize-none py-2 ${NOTE}`}
      />
    </>
  );
}

/**
 * How an answered card reads: an icon in its tone, what happened, and the
 * owner's note. With `details`, the line opens to show what was asked.
 */
export function SettledLine({
  icon: Icon,
  tone,
  text,
  note,
  aside,
  details,
}: {
  icon: LucideIcon;
  tone: Tone;
  text: string;
  note?: string | null;
  /** After the note: what it was about, or when it happened. */
  aside?: ReactNode;
  details?: ReactNode;
}) {
  const line = (
    <>
      <Icon aria-hidden size={14} className={`shrink-0 ${TONES[tone].text}`} />
      <span className={`font-medium ${note || aside ? "shrink-0" : "truncate"}`}>{text}</span>
      {note && <span className="truncate text-muted text-xs">{`“${note}”`}</span>}
      {aside}
    </>
  );
  const row = "flex min-w-0 items-center gap-2 rounded-lg px-1.5 py-1 text-sm";
  if (!details) {
    return <p className={row}>{line}</p>;
  }
  return (
    <details className="group">
      <summary className={`${row} cursor-default hover:bg-sunken`}>
        {line}
        <ChevronRight
          aria-hidden
          size={13}
          className="shrink-0 text-muted transition-transform group-open:rotate-90"
        />
      </summary>
      <div
        className="mt-1.5 ml-6 flex flex-col gap-2 rounded-2xl bg-bubble px-4 py-3 text-sm"
        data-selectable
      >
        {details}
      </div>
    </details>
  );
}
