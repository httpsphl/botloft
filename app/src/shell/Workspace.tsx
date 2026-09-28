// The connected app: title bar, crews on the left, the selection on the
// right.

import { ChevronRight, LoaderCircle, TriangleAlert } from "lucide-react";
import type { ReactNode } from "react";
import { BotAvatar } from "../features/bots/BotAvatar";
import { BotView } from "../features/bots/BotView";
import { CrewView } from "../features/crews/CrewView";
import { Sidebar } from "../features/crews/Sidebar";
import { Welcome } from "../features/onboarding/Welcome";
import { useApp } from "../store/context";
import { Callout } from "../ui/Callout";
import { TitleBar } from "./TitleBar";

export function Workspace() {
  const loaded = useApp((state) => state.loaded);
  const loadError = useApp((state) => state.loadError);
  const hasCrews = useApp((state) => Object.keys(state.crews).length > 0);
  const runtimeError = useApp((state) => state.system?.runtimeError ?? null);

  let main: ReactNode;
  if (loadError) {
    main = (
      <div className="p-6">
        <Callout tone="danger" title="Could not load your crews">
          {loadError}
        </Callout>
      </div>
    );
  } else if (!loaded) {
    main = null;
  } else if (!hasCrews) {
    main = <Welcome />;
  } else {
    main = (
      <div className="flex min-h-0 flex-1">
        <Sidebar />
        <main className="flex min-w-0 flex-1 flex-col">
          {runtimeError && (
            <div className="border-line border-b p-3">
              <Callout tone="danger" title="Bots cannot start">
                {runtimeError} The daemon checks again every 30 seconds.
              </Callout>
            </div>
          )}
          <Selection />
        </main>
      </div>
    );
  }

  return (
    <div className="flex h-full flex-col">
      <TitleBar status={<ConnectionStatus />}>
        <Breadcrumb />
      </TitleBar>
      {main}
    </div>
  );
}

function Selection() {
  const crew = useApp((state) => (state.selectedCrewId ? state.crews[state.selectedCrewId] : null));
  const bot = useApp((state) => (state.selectedBotId ? state.bots[state.selectedBotId] : null));
  if (crew && bot) {
    return <BotView key={bot.id} bot={bot} crew={crew} />;
  }
  if (crew) {
    return <CrewView key={crew.id} crew={crew} />;
  }
  return (
    <p className="p-6 text-muted">Pick a crew on the left, or create one with the + button.</p>
  );
}

function Breadcrumb() {
  const crew = useApp((state) => (state.selectedCrewId ? state.crews[state.selectedCrewId] : null));
  const bot = useApp((state) => (state.selectedBotId ? state.bots[state.selectedBotId] : null));
  const selectCrew = useApp((state) => state.selectCrew);
  if (!crew) {
    return null;
  }
  return (
    <>
      <ChevronRight aria-hidden size={14} className="text-muted" />
      <button
        type="button"
        onClick={() => selectCrew(crew.id)}
        className="truncate text-ink-soft hover:text-ink"
      >
        {crew.name}
      </button>
      {bot && (
        <>
          <ChevronRight aria-hidden size={14} className="text-muted" />
          <BotAvatar color={bot.color} size={14} />
          <span className="truncate font-medium">{bot.name}</span>
        </>
      )}
    </>
  );
}

/** Shown only when the connection is not healthy. */
function ConnectionStatus() {
  const connection = useApp((state) => state.connection);
  if (connection.kind === "open") {
    return null;
  }
  const lost = connection.kind === "failed" || connection.kind === "closed";
  const Icon = lost ? TriangleAlert : LoaderCircle;
  return (
    <span
      role="status"
      className={`mr-1 flex items-center gap-1.5 px-2 font-medium text-xs ${lost ? "text-danger" : "text-warn"}`}
    >
      <Icon aria-hidden size={13} className={lost ? "" : "animate-spin"} />
      {lost ? "Disconnected" : "Reconnecting…"}
    </span>
  );
}
