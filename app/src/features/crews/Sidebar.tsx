import { Pause, Plus } from "lucide-react";
import { useState } from "react";
import { useShallow } from "zustand/react/shallow";
import type { Crew } from "../../lib/protocol.gen";
import { botsOf, crewList } from "../../store/app";
import { useApp } from "../../store/context";
import { Button } from "../../ui/Button";
import { BotAvatar } from "../bots/BotAvatar";
import { BotStateBadge } from "../bots/BotStateBadge";
import { CrewDialog } from "./CrewDialog";

export function Sidebar() {
  const crews = useApp(useShallow(crewList));
  const [creating, setCreating] = useState(false);
  return (
    <nav aria-label="Crews" className="flex w-64 shrink-0 flex-col border-line border-r bg-panel">
      <div className="flex h-10 shrink-0 items-center justify-between pr-1.5 pl-4">
        <h2 className="font-semibold text-muted text-xs uppercase tracking-[0.12em]">Crews</h2>
        <Button
          variant="ghost"
          size="sm"
          icon={Plus}
          label="New crew"
          onClick={() => setCreating(true)}
        />
      </div>
      <ul className="min-h-0 flex-1 overflow-y-auto pb-3">
        {crews.map((crew) => (
          <CrewEntry key={crew.id} crew={crew} />
        ))}
      </ul>
      {creating && <CrewDialog onClose={() => setCreating(false)} />}
    </nav>
  );
}

function CrewEntry({ crew }: { crew: Crew }) {
  const bots = useApp(useShallow((state) => botsOf(state, crew.id)));
  const selectedCrew = useApp((state) => state.selectedCrewId === crew.id && !state.selectedBotId);
  const selectedBot = useApp((state) => state.selectedBotId);
  const selectCrew = useApp((state) => state.selectCrew);
  const selectBot = useApp((state) => state.selectBot);
  const row = "relative flex w-full items-center gap-2 pr-3 text-left hover:bg-sunken";
  const marker = "before:absolute before:inset-y-0 before:left-0 before:w-[3px] before:bg-accent";

  return (
    <li className="mt-1">
      <button
        type="button"
        aria-current={selectedCrew ? "page" : undefined}
        onClick={() => selectCrew(crew.id)}
        className={`${row} h-8 pl-4 font-semibold text-sm ${selectedCrew ? `bg-sunken ${marker}` : ""}`}
      >
        <span className="min-w-0 flex-1 truncate">{crew.name}</span>
        {crew.paused && (
          <span className="flex items-center gap-1 font-medium text-quiet text-xs">
            <Pause aria-hidden size={12} />
            Paused
          </span>
        )}
      </button>
      <ul>
        {bots.map((bot) => {
          const selected = selectedBot === bot.id;
          return (
            <li key={bot.id}>
              <button
                type="button"
                aria-current={selected ? "page" : undefined}
                onClick={() => selectBot(bot.id)}
                className={`${row} h-8 pl-6 text-sm ${selected ? `bg-sunken ${marker}` : ""}`}
              >
                <BotAvatar color={bot.color} size={18} />
                <span className="min-w-0 flex-1 truncate">{bot.name}</span>
                <BotStateBadge bot={bot} crewPaused={crew.paused} compact />
              </button>
            </li>
          );
        })}
      </ul>
    </li>
  );
}
