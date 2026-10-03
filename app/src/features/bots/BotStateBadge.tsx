import {
  Archive,
  Hand,
  Hourglass,
  KeyRound,
  LoaderCircle,
  Pause,
  Power,
  RotateCw,
} from "lucide-react";
import { type Messages, t, useT } from "../../i18n";
import type { Bot, BotState } from "../../lib/protocol.gen";
import { TONES, type Tone as ToneName } from "../../ui/tone";
import { ReadyIcon, type StateIcon, WorkingIcon } from "./StateIcons";

type Tone = Exclude<ToneName, "accent">;

export interface StateView {
  label: string;
  tone: Tone;
  icon: StateIcon;
  spin?: boolean;
  /** One sentence on what the state means for the owner. */
  hint: string;
}

/** How a state looks; its words come from the current language. */
type Look = Omit<StateView, "label" | "hint">;

const looks: Record<BotState, Look> = {
  offline: { tone: "quiet", icon: Power },
  launching: { tone: "work", icon: LoaderCircle, spin: true },
  idle: { tone: "ok", icon: ReadyIcon },
  busy: { tone: "work", icon: WorkingIcon },
  needs_approval: { tone: "warn", icon: Hand },
  rate_limited: { tone: "warn", icon: Hourglass },
  auth_error: { tone: "danger", icon: KeyRound },
  backoff: { tone: "warn", icon: RotateCw },
  archived: { tone: "quiet", icon: Archive },
};

const pausedLook: Look = { tone: "quiet", icon: Pause };

/**
 * How a bot's state reads in the UI. Paused and stopped reads as paused.
 * Components pass the messages from `useT()` so they follow a language
 * change; other callers get the current language.
 */
export function stateView(
  bot: Pick<Bot, "state" | "paused">,
  crewPaused = false,
  messages: Messages = t(),
): StateView {
  const words = messages.bots.states;
  if ((bot.paused || crewPaused) && bot.state === "offline") {
    return { ...pausedLook, ...words.paused };
  }
  return { ...looks[bot.state], ...words[bot.state] };
}

export function BotStateBadge({
  bot,
  crewPaused,
  compact = false,
}: {
  bot: Pick<Bot, "state" | "paused">;
  crewPaused?: boolean;
  compact?: boolean;
}) {
  const messages = useT();
  const view = stateView(bot, crewPaused, messages);
  const Icon = view.icon;
  return (
    <span
      title={view.hint}
      className={`inline-flex shrink-0 items-center gap-1 font-medium transition-colors duration-300 ${compact ? "text-xs" : "text-sm"} ${TONES[view.tone].text}`}
    >
      <Icon
        aria-hidden
        size={compact ? 12 : 14}
        className={
          view.spin ? "animate-spin" : bot.state === "needs_approval" ? "animate-wave" : ""
        }
      />
      {view.label}
    </span>
  );
}
