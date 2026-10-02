import { BotStateBadge, setLocaleChoice } from "@botloft/ui";

// The app follows the browser's language; the cards are in English.
setLocaleChoice("en");

type State =
  | "offline"
  | "launching"
  | "idle"
  | "busy"
  | "needs_approval"
  | "rate_limited"
  | "auth_error"
  | "backoff"
  | "archived";

const states: State[] = [
  "idle",
  "busy",
  "needs_approval",
  "launching",
  "rate_limited",
  "backoff",
  "auth_error",
  "offline",
];

export function States() {
  return (
    <div className="flex flex-col gap-2 p-4">
      {states.map((state) => (
        <BotStateBadge key={state} bot={{ state, paused: false }} />
      ))}
      <BotStateBadge bot={{ state: "offline", paused: true }} />
    </div>
  );
}

export function Compact() {
  return (
    <div className="flex flex-wrap gap-4 p-4">
      <BotStateBadge compact bot={{ state: "idle", paused: false }} />
      <BotStateBadge compact bot={{ state: "busy", paused: false }} />
      <BotStateBadge compact bot={{ state: "needs_approval", paused: false }} />
      <BotStateBadge compact bot={{ state: "offline", paused: true }} />
    </div>
  );
}

export function DarkTheme() {
  return (
    <div data-theme="dark" className="flex flex-col gap-2 bg-canvas p-4">
      <BotStateBadge bot={{ state: "idle", paused: false }} />
      <BotStateBadge bot={{ state: "busy", paused: false }} />
      <BotStateBadge bot={{ state: "needs_approval", paused: false }} />
      <BotStateBadge bot={{ state: "auth_error", paused: false }} />
    </div>
  );
}
