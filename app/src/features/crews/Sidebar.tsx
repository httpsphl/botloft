import {
  ArrowDown,
  ArrowUp,
  ChevronDown,
  ChevronsDownUp,
  ChevronsUpDown,
  Pause,
  Plus,
} from "lucide-react";
import { useCallback, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { useShallow } from "zustand/react/shallow";
import { useT } from "../../i18n";
import type { Crew, CrewId } from "../../lib/protocol.gen";
import { crewList } from "../../store/app";
import { useApp } from "../../store/context";
import { unreadIn } from "../../store/seen";
import { CountBadge } from "../../ui/Badge";
import { Button } from "../../ui/Button";
import { ContextMenu, menuPoint, type Point } from "../../ui/ContextMenu";
import { CrewDialog } from "./CrewDialog";
import { setAllCollapsed, setCollapsed, useCollapsed, useCollapsedSet } from "./collapsed";
import { useCrewActions } from "./crewActions";
import { moveBy, orderedCrews, setCrewOrder, useCrewOrder } from "./crewOrder";
import { Conversation } from "./SidebarConversation";
import { row } from "./SidebarPages";
import { StateFilters } from "./StateFilters";
import { filteredBots, useStateFilter } from "./stateFilter";
import { useCrewDrag } from "./useCrewDrag";

/** Crews as sections and their bots as conversations (spec 15.1). */
export function Sidebar() {
  const t = useT();
  const words = t.crews.sidebar;
  const order = useCrewOrder();
  const crews = orderedCrews(useApp(useShallow(crewList)), order);
  const ids = crews.map((crew) => crew.id);
  const list = useRef<HTMLUListElement>(null);
  const drag = useCrewDrag(ids, list);
  const overview = useApp((state) => state.selectedCrewId === null && !state.page);
  const selectCrew = useApp((state) => state.selectCrew);
  const collapsed = useCollapsedSet();
  const allFolded = crews.length > 0 && crews.every((crew) => collapsed.has(crew.id));
  const [creating, setCreating] = useState(false);
  return (
    <div className="flex w-72 shrink-0 flex-col border-line border-r bg-panel">
      <nav aria-label={words.label} className="flex min-h-0 flex-1 flex-col">
        <div className="flex h-10 shrink-0 items-center gap-0.5 pr-1.5 pl-2">
          <h2 className="min-w-0 flex-1">
            <button
              type="button"
              aria-current={overview ? "page" : undefined}
              title={words.showAll}
              onClick={() => selectCrew(null)}
              className={`rounded-lg px-2 py-1 font-semibold text-xs uppercase tracking-[0.12em] transition-colors hover:bg-sunken hover:text-ink ${overview ? "bg-sunken text-ink" : "text-muted"}`}
            >
              {words.label}
            </button>
          </h2>
          <Button
            variant="ghost"
            size="sm"
            icon={allFolded ? ChevronsUpDown : ChevronsDownUp}
            label={allFolded ? words.expandAll : words.collapseAll}
            onClick={() => setAllCollapsed(allFolded ? [] : crews.map((crew) => crew.id))}
          />
          <Button
            variant="ghost"
            size="sm"
            icon={Plus}
            label={t.crews.newCrew}
            onClick={() => setCreating(true)}
          />
        </div>
        {crews.length > 0 && <StateFilters />}
        <ul ref={list} className="min-h-0 flex-1 overflow-y-auto px-2 pb-3">
          {crews.map((crew) => (
            <CrewEntry key={crew.id} crew={crew} ids={ids} drag={drag} />
          ))}
        </ul>
      </nav>
      {creating && <CrewDialog onClose={() => setCreating(false)} />}
    </div>
  );
}

function CrewEntry({
  crew,
  ids,
  drag,
}: {
  crew: Crew;
  ids: CrewId[];
  drag: ReturnType<typeof useCrewDrag>;
}) {
  const t = useT();
  const filter = useStateFilter();
  const bots = useApp(useShallow((state) => filteredBots(state, crew.id, filter)));
  const selected = useApp((state) => state.selectedCrewId === crew.id && !state.selectedBotId);
  const unread = useApp((state) => unreadIn(state, crew.id));
  const selectCrew = useApp((state) => state.selectCrew);
  const openBotId = useApp((state) => state.selectedBotId);
  const collapsed = useCollapsed(crew.id);
  // A right-click opens the crew's menu, the same as on its page.
  const actions = useCrewActions(crew);
  const [menuAt, setMenuAt] = useState<Point | null>(null);
  const closeMenu = useCallback(() => setMenuAt(null), []);
  // Folded, the open conversation still shows, and a dot tells a bot waits.
  const shown = collapsed ? bots.filter((bot) => bot.id === openBotId) : bots;
  const waiting =
    collapsed && bots.some((bot) => bot.state === "needs_approval" || bot.state === "auth_error");
  const words = t.crews.sidebar;
  // A line shows where the dragged crew would land.
  const spot = drag.over?.id === crew.id && drag.dragging !== crew.id ? drag.over : null;
  const dropEdge = spot
    ? `border-accent border-solid ${spot.after ? "border-b-2" : "border-t-2"}`
    : "";
  const index = ids.indexOf(crew.id);
  const moves = [
    index > 0 && {
      label: words.moveUp,
      icon: ArrowUp,
      onSelect: () => setCrewOrder(moveBy(ids, crew.id, -1)),
    },
    index < ids.length - 1 && {
      label: words.moveDown,
      icon: ArrowDown,
      onSelect: () => setCrewOrder(moveBy(ids, crew.id, 1)),
    },
  ].filter((item) => item !== false);

  // Filtering, a crew with nothing to show steps aside.
  if (filter !== "all" && bots.length === 0) {
    return null;
  }
  return (
    <li
      data-crew-id={crew.id}
      className={`mt-2 ${drag.dragging === crew.id ? "opacity-50" : ""} ${dropEdge}`}
    >
      <div className="relative flex items-center">
        <button
          type="button"
          aria-expanded={!collapsed}
          aria-label={collapsed ? words.expand(crew.name) : words.collapse(crew.name)}
          title={collapsed ? words.expand(crew.name) : words.collapse(crew.name)}
          onClick={() => setCollapsed(crew.id, !collapsed)}
          className="absolute left-1 z-10 grid size-6 place-items-center rounded-lg text-muted hover:bg-line hover:text-ink"
        >
          <ChevronDown
            aria-hidden
            size={14}
            className={`transition-transform duration-150 ${collapsed ? "-rotate-90" : ""}`}
          />
        </button>
        <button
          type="button"
          aria-current={selected ? "page" : undefined}
          onPointerDown={(event) => drag.start(crew.id, event)}
          onClick={() => {
            if (!drag.wasDrag()) {
              selectCrew(crew.id);
            }
          }}
          onContextMenu={(event) => {
            event.preventDefault();
            setMenuAt(menuPoint(event));
          }}
          className={`${row} h-8 gap-2 pr-2.5 pl-8 font-semibold text-sm ${selected || menuAt ? "bg-sunken" : ""}`}
        >
          {crew.color && (
            <span
              aria-hidden
              data-testid="crew-color"
              className="size-2.5 shrink-0 rounded-full"
              style={{ background: crew.color }}
            />
          )}
          <span className="min-w-0 flex-1 truncate">{crew.name}</span>
          {waiting && (
            <span className="flex shrink-0">
              <span aria-hidden className="live-dot" style={{ background: "var(--warn)" }} />
              <span className="sr-only">{words.waiting}</span>
            </span>
          )}
          {unread > 0 && (
            <CountBadge tone="accent" count={unread} label={t.crews.sidebar.unreadCount(unread)} />
          )}
          {crew.paused && (
            <span className="flex items-center gap-1 font-medium text-quiet text-xs">
              <Pause aria-hidden size={12} />
              {t.crews.paused}
            </span>
          )}
        </button>
      </div>
      {menuAt && (
        <ContextMenu
          label={words.menuOf(crew.name)}
          items={[actions.newBot, actions.pause, ...moves, ...actions.items]}
          at={menuAt}
          onClose={closeMenu}
        />
      )}
      {/* Outside the list: a dialog is not part of the navigation. */}
      {actions.dialogs && createPortal(actions.dialogs, document.body)}
      <ul>
        {shown.map((bot) => (
          <Conversation key={bot.id} bot={bot} crew={crew} />
        ))}
      </ul>
    </li>
  );
}
