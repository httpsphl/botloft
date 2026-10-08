// The small pieces the phone's screens share: big buttons for a thumb, the
// bot's dot, and a bot's markdown with no links and no pictures.

import type { ButtonHTMLAttributes, ReactNode } from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { BotAvatar, moodOf } from "../features/bots/BotAvatar";
import type { BotState } from "../lib/protocol.gen";

type Look = "primary" | "danger" | "quiet";

const looks: Record<Look, string> = {
  primary: "border-ink bg-ink text-canvas",
  danger: "border-danger/60 bg-panel text-danger",
  quiet: "border-line-strong bg-panel text-ink",
};

/** 48 px tall: a target a thumb finds. */
export function PhoneButton({
  look = "quiet",
  className = "",
  type = "button",
  ...rest
}: ButtonHTMLAttributes<HTMLButtonElement> & { look?: Look }) {
  return (
    <button
      type={type}
      className={`inline-flex h-12 min-w-0 flex-1 items-center justify-center rounded-xl border px-4 font-medium text-base active:scale-[0.98] disabled:pointer-events-none disabled:opacity-45 ${looks[look]} ${className}`}
      {...rest}
    />
  );
}

/** The bot's mascot in its color, as on the computer; it moves with what the bot is doing. */
export function BotDot({
  color,
  state,
  size = 36,
}: {
  /** Kept for callers; the mascot has no letter. */
  name?: string;
  color: string;
  state?: BotState;
  size?: number;
}) {
  return (
    // The flame of a working bot rises above the drawing: the box holds it.
    <span
      aria-hidden
      style={{ width: size * 1.3, height: size * 1.3 }}
      className="inline-flex shrink-0 items-end justify-center overflow-hidden"
    >
      <BotAvatar
        color={/^#[0-9a-f]{3,8}$/i.test(color) ? color : "#FF7A59"}
        size={size}
        mood={state ? moodOf({ state, paused: false }) : undefined}
      />
    </span>
  );
}

/** What a bot wrote as markdown; links and pictures are shown as words only. */
export function PhoneMarkdown({ children }: { children: string }) {
  return (
    <div className="chat-md text-base">
      <ReactMarkdown
        remarkPlugins={[remarkGfm]}
        skipHtml
        components={{
          a: ({ children: label }: { children?: ReactNode }) => (
            <span className="underline decoration-dotted">{label}</span>
          ),
          img: ({ alt }: { alt?: string | undefined }) => <span>{alt}</span>,
        }}
      >
        {children}
      </ReactMarkdown>
    </div>
  );
}
