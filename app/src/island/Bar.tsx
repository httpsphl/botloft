// The top of the island's panel (spec 31.2): the tabs as icons on the left;
// on the right, pause or resume every crew, sound on or off, open Botloft
// and hide. It stays in place while the panel scrolls.

import {
  Gauge,
  House,
  MessageCircle,
  Pause,
  Play,
  SquareArrowOutUpRight,
  Volume2,
  VolumeX,
  X,
} from "lucide-react";
import type { ReactNode } from "react";
import { useT } from "../i18n";
import type { BotloftApi } from "../lib/api";
import type { Crew } from "../lib/protocol.gen";
import { prefs, usePref } from "../shell/prefs";

export type Tab = "home" | "chat" | "usage";

function Icon({
  label,
  onClick,
  children,
}: {
  label: string;
  onClick: () => void;
  children: ReactNode;
}) {
  return (
    <button type="button" className="island-tab" title={label} aria-label={label} onClick={onClick}>
      {children}
    </button>
  );
}

export function Bar({
  api,
  crews,
  tab,
  onTab,
  onOpenApp,
  onHide,
  onChanged,
}: {
  api: BotloftApi;
  crews: Crew[];
  tab: Tab;
  onTab: (tab: Tab) => void;
  onOpenApp: () => void;
  onHide: () => void;
  onChanged: () => void;
}) {
  const t = useT();
  const sound = usePref(prefs.sound);
  const tabs: [Tab, string, ReactNode][] = [
    ["home", t.island.tabs.home, <House key="home" aria-hidden size={16} />],
    ["chat", t.island.tabs.chat, <MessageCircle key="chat" aria-hidden size={16} />],
    ["usage", t.island.tabs.usage, <Gauge key="usage" aria-hidden size={16} />],
  ];
  const active = crews.filter((crew) => crew.archivedAt === null);
  const allPaused = active.length > 0 && active.every((crew) => crew.paused);
  const pauseAll = () =>
    Promise.all(
      active
        .filter((crew) => crew.paused !== !allPaused)
        .map((crew) => api.call("crews.setPaused", { crewId: crew.id, paused: !allPaused })),
    ).then(onChanged, onChanged);
  return (
    <div className="sticky top-0 z-10 flex items-center justify-between bg-[var(--island-bg)] px-3 pt-2.5 pb-2">
      <div role="tablist" aria-label={t.island.tabs.label} className="flex gap-1">
        {tabs.map(([id, name, icon]) => (
          <button
            key={id}
            type="button"
            role="tab"
            aria-selected={tab === id}
            aria-label={name}
            title={name}
            className="island-tab"
            onClick={() => onTab(id)}
          >
            {icon}
          </button>
        ))}
      </div>
      <div className="flex gap-1">
        {active.length > 0 && (
          <Icon label={allPaused ? t.island.resumeAll : t.island.pauseAll} onClick={pauseAll}>
            {allPaused ? <Play aria-hidden size={15} /> : <Pause aria-hidden size={15} />}
          </Icon>
        )}
        <Icon
          label={sound ? t.island.soundOff : t.island.soundOn}
          onClick={() => prefs.sound.set(!sound)}
        >
          {sound ? <Volume2 aria-hidden size={15} /> : <VolumeX aria-hidden size={15} />}
        </Icon>
        <Icon label={t.island.openApp} onClick={onOpenApp}>
          <SquareArrowOutUpRight aria-hidden size={15} />
        </Icon>
        <Icon label={t.island.hide} onClick={onHide}>
          <X aria-hidden size={15} />
        </Icon>
      </div>
    </div>
  );
}
