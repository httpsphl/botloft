// What a panel or page shows with nothing in it yet (spec 15.3): the bot's
// mascot, resting, with what the place is for, and an icon of it.

import type { LucideIcon } from "lucide-react";
import { BotAvatar } from "../features/bots/BotAvatar";

export function EmptyState({
  color,
  icon: Icon,
  title,
  body,
  framed = false,
}: {
  /** The mascot's color: the bot's, or the app's first for a page of all bots. */
  color: string;
  icon?: LucideIcon;
  title: string;
  body: string;
  /** On a dotted board or a busy page, the text sits on a card. */
  framed?: boolean;
}) {
  return (
    <div
      className={`flex flex-col items-center gap-3 text-center ${
        framed ? "mx-auto max-w-80 rounded-2xl bg-panel/80 px-6 py-10" : "px-6 py-14"
      }`}
    >
      <span className="relative">
        <BotAvatar color={color} size={44} mood="idle" />
        {Icon && (
          <Icon
            aria-hidden
            size={18}
            className="absolute -right-2 -bottom-1 rounded-full bg-panel p-0.5 text-muted"
          />
        )}
      </span>
      <p className="font-semibold text-sm">{title}</p>
      <p className="max-w-80 text-muted text-sm">{body}</p>
    </div>
  );
}
