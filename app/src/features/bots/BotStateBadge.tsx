import {
  Activity,
  Archive,
  CircleDot,
  Hand,
  Hourglass,
  KeyRound,
  LoaderCircle,
  type LucideIcon,
  Pause,
  Power,
  RotateCw,
} from "lucide-react";
import type { Bot, BotState } from "../../lib/protocol.gen";

type Tone = "ok" | "work" | "warn" | "danger" | "quiet";

export interface StateView {
  label: string;
  tone: Tone;
  icon: LucideIcon;
  spin?: boolean;
  /** One sentence on what the state means for the owner. */
  hint: string;
}

const views: Record<BotState, StateView> = {
  offline: { label: "Offline", tone: "quiet", icon: Power, hint: "Not running." },
  launching: {
    label: "Starting",
    tone: "work",
    icon: LoaderCircle,
    spin: true,
    hint: "Claude Code is starting.",
  },
  idle: { label: "Idle", tone: "ok", icon: CircleDot, hint: "Ready for work." },
  busy: { label: "Working", tone: "work", icon: Activity, hint: "Working on something." },
  needs_approval: {
    label: "Needs approval",
    tone: "warn",
    icon: Hand,
    hint: "Waiting for you to allow or deny a tool in its chat.",
  },
  rate_limited: {
    label: "Usage limit",
    tone: "warn",
    icon: Hourglass,
    hint: "Your Claude plan hit its usage limit; messages wait until it resets.",
  },
  auth_error: {
    label: "Sign-in needed",
    tone: "danger",
    icon: KeyRound,
    hint: "Claude Code is not signed in. Open Claude Code and sign in, then restart the bot.",
  },
  backoff: {
    label: "Restarting",
    tone: "warn",
    icon: RotateCw,
    hint: "It stopped unexpectedly; Botloft starts it again shortly.",
  },
  archived: { label: "Archived", tone: "quiet", icon: Archive, hint: "Archived." },
};

const paused: StateView = {
  label: "Paused",
  tone: "quiet",
  icon: Pause,
  hint: "Paused; resume to start it.",
};

/** How a bot's state reads in the UI. Paused and stopped reads as paused. */
export function stateView(bot: Pick<Bot, "state" | "paused">, crewPaused = false): StateView {
  if ((bot.paused || crewPaused) && bot.state === "offline") {
    return paused;
  }
  return views[bot.state];
}

const toneText: Record<Tone, string> = {
  ok: "text-ok",
  work: "text-work",
  warn: "text-warn",
  danger: "text-danger",
  quiet: "text-quiet",
};

export function BotStateBadge({
  bot,
  crewPaused,
  compact = false,
}: {
  bot: Pick<Bot, "state" | "paused">;
  crewPaused?: boolean;
  compact?: boolean;
}) {
  const view = stateView(bot, crewPaused);
  const Icon = view.icon;
  return (
    <span
      title={view.hint}
      className={`inline-flex shrink-0 items-center gap-1 font-medium ${compact ? "text-xs" : "text-sm"} ${toneText[view.tone]}`}
    >
      <Icon aria-hidden size={compact ? 12 : 14} className={view.spin ? "animate-spin" : ""} />
      {view.label}
    </span>
  );
}
