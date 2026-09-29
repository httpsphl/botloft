// The design area beside a bot's chat (spec 22.5): its HTML screens as
// artboards on a dotted board, the one being written glowing in the bot's
// color and growing as it writes. A click opens a screen on its own.

import { LayoutTemplate, Maximize2, Minimize2, Minus, Plus, X } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { useT } from "../../i18n";
import { fileSize } from "../../lib/format";
import type { Bot, Screen, ScreenDevice } from "../../lib/protocol.gen";
import { Button } from "../../ui/Button";
import { Callout } from "../../ui/Callout";
import { SidePanel } from "../../ui/SidePanel";
import { BotAvatar } from "../bots/BotAvatar";
import { DEVICES, LiveFrame } from "./LiveFrame";
import { scaleOf, useDevices, useWidth, useZoom, ZOOMS } from "./prefs";
import { ScreenFocus } from "./ScreenFocus";
import type { BotScreens } from "./useScreens";

/** Screens on the board before "Show more". */
const BOARD_MAX = 12;

export function ScreensPanel({
  bot,
  data,
  path,
  onPath,
  onClose,
}: {
  bot: Bot;
  data: BotScreens;
  /** The screen open on its own, or null for the board. */
  path: string | null;
  onPath(path: string | null): void;
  onClose(): void;
}) {
  const t = useT().screens;
  const [expanded, setExpanded] = useState(false);
  const [zoom, setZoom] = useZoom();
  const [deviceOf, choose] = useDevices();
  const board = useRef<HTMLDivElement>(null);
  const width = useWidth(board);
  const focused = path ? data.screens.find((screen) => same(screen.path, path)) : undefined;
  const widest = Math.max(
    DEVICES.mobile.width,
    ...data.screens.map((screen) => DEVICES[deviceOf(screen)].width),
  );
  const scale = scaleOf(zoom, width, widest);
  const step = (direction: 1 | -1) => {
    const levels: readonly number[] = ZOOMS;
    const at = levels.findIndex((level) => level >= scale - 0.001);
    const index = Math.min(
      levels.length - 1,
      Math.max(0, (at < 0 ? levels.length : at) + direction),
    );
    setZoom(levels[index] ?? scale);
  };

  return (
    <SidePanel label={t.panel(bot.name)} name="screens" defaultWidth={640} expanded={expanded}>
      <header className="flex h-11 shrink-0 items-center justify-between gap-2 border-line border-b pr-1.5 pl-4">
        <h2 className="font-semibold text-sm">{t.heading}</h2>
        <div className="flex items-center gap-0.5">
          {!path && data.screens.length > 0 && (
            <>
              <Button
                variant="ghost"
                size="sm"
                icon={Minus}
                label={t.zoomOut}
                onClick={() => step(-1)}
              />
              <button
                type="button"
                onClick={() => setZoom("fit")}
                title={t.fit}
                className="h-7 min-w-12 rounded-md px-1.5 font-mono text-muted text-xs tabular-nums transition-colors hover:bg-sunken hover:text-ink"
              >
                {Math.round(scale * 100)}%
              </button>
              <Button
                variant="ghost"
                size="sm"
                icon={Plus}
                label={t.zoomIn}
                onClick={() => step(1)}
              />
            </>
          )}
          <Button
            variant="ghost"
            size="sm"
            icon={expanded ? Minimize2 : Maximize2}
            label={expanded ? t.shrink : t.expand}
            aria-pressed={expanded}
            onClick={() => setExpanded(!expanded)}
          />
          <Button variant="ghost" size="sm" icon={X} label={t.close} onClick={onClose} />
        </div>
      </header>
      {focused ? (
        <ScreenFocus
          bot={bot}
          screen={focused}
          device={deviceOf(focused)}
          onDevice={(device: ScreenDevice) => choose(focused.path, device)}
          onBack={() => onPath(null)}
        />
      ) : path !== null ? (
        <Gone onBack={() => onPath(null)} />
      ) : (
        <div ref={board} className="design-board min-h-0 flex-1 overflow-auto p-6">
          <Board bot={bot} data={data} scale={scale} deviceOf={deviceOf} onOpen={onPath} />
        </div>
      )}
    </SidePanel>
  );
}

const same = (a: string, b: string) => a.toLowerCase() === b.toLowerCase();

function Board({
  bot,
  data,
  scale,
  deviceOf,
  onOpen,
}: {
  bot: Bot;
  data: BotScreens;
  scale: number;
  deviceOf(screen: Screen): ScreenDevice;
  onOpen(path: string): void;
}) {
  const t = useT().screens;
  const [all, setAll] = useState(false);
  if (data.error && data.screens.length === 0) {
    return (
      <Callout tone="danger" title={t.loadFailed}>
        {data.error}
      </Callout>
    );
  }
  if (data.screens.length === 0 && !data.loading) {
    return (
      <div className="mx-auto flex max-w-80 flex-col items-center gap-3 rounded-2xl bg-panel/80 px-6 py-10 text-center">
        <span className="relative">
          <BotAvatar color={bot.color} size={44} mood="idle" />
          <LayoutTemplate
            aria-hidden
            size={18}
            className="absolute -right-2 -bottom-1 rounded-full bg-panel p-0.5 text-muted"
          />
        </span>
        <p className="font-semibold text-sm">{t.emptyTitle(bot.name)}</p>
        <p className="text-muted text-sm">{t.emptyBody}</p>
      </div>
    );
  }
  const shown = all ? data.screens : data.screens.slice(0, BOARD_MAX);
  return (
    <>
      <ul aria-label={t.board} className="flex flex-wrap items-start gap-x-8 gap-y-7">
        {shown.map((screen) => (
          <Artboard
            key={screen.path.toLowerCase()}
            bot={bot}
            screen={screen}
            device={deviceOf(screen)}
            scale={scale}
            bytes={data.sizes[screen.path.toLowerCase()]}
            onOpen={() => onOpen(screen.path)}
          />
        ))}
      </ul>
      {!all && data.screens.length > BOARD_MAX && (
        <div className="mt-6">
          <Button onClick={() => setAll(true)}>{t.more(data.screens.length - BOARD_MAX)}</Button>
        </div>
      )}
    </>
  );
}

function Artboard({
  bot,
  screen,
  device,
  scale,
  bytes,
  onOpen,
}: {
  bot: Bot;
  screen: Screen;
  device: ScreenDevice;
  scale: number;
  /** Written so far, while the bot writes it. */
  bytes: number | undefined;
  onOpen(): void;
}) {
  const t = useT().screens;
  const item = useRef<HTMLLIElement>(null);
  // The board comes to the screen the bot starts writing.
  useEffect(() => {
    if (screen.writing) {
      item.current?.scrollIntoView?.({ behavior: "smooth", block: "nearest", inline: "nearest" });
    }
  }, [screen.writing]);
  return (
    <li ref={item} className="shrink-0 animate-rise">
      <p className="mb-1.5 flex max-w-full items-center gap-2 text-xs">
        <span className="truncate font-medium text-ink-soft" title={screen.path}>
          {screen.name}
        </span>
        <span className="shrink-0 text-muted">{t.devices[device]}</span>
        {screen.writing && (
          <span
            className="inline-flex shrink-0 items-center gap-1 rounded-full px-1.5 py-px font-medium"
            style={{ color: bot.color, background: `${bot.color}1f` }}
          >
            <BotAvatar color={bot.color} size={12} mood="working" />
            {t.writing}
            {bytes !== undefined && bytes > 0 && (
              <span className="font-mono tabular-nums opacity-80">{fileSize(bytes)}</span>
            )}
          </span>
        )}
      </p>
      <div
        className="relative overflow-hidden rounded-md shadow-lift ring-1 ring-line transition-shadow"
        style={
          screen.writing
            ? { boxShadow: `0 0 0 2px ${bot.color}, 0 0 28px ${bot.color}55` }
            : undefined
        }
      >
        <LiveFrame url={screen.url} device={device} scale={scale} title={screen.name} />
        <button
          type="button"
          aria-label={t.open(screen.name)}
          onClick={onOpen}
          className="absolute inset-0 cursor-zoom-in focus-visible:outline-2 focus-visible:outline-work"
        />
      </div>
    </li>
  );
}

function Gone({ onBack }: { onBack(): void }) {
  const t = useT().screens;
  return (
    <div className="flex flex-col items-start gap-3 p-4">
      <Button variant="ghost" size="sm" onClick={onBack}>
        {t.back}
      </Button>
      <p role="alert" className="text-ink-soft text-sm">
        {t.gone}
      </p>
    </div>
  );
}
