// The bot's own browser beside its chat (spec 21.8): its tabs, the address
// and the page it is on, live, with its cursor and a line about what it
// just did. Watching starts when the panel opens and stops when it closes.
// The owner can take it into their own hands and give it back (spec 21.10),
// or open it in a window of its own to sign in (spec 21.11).

import { Globe, LoaderCircle, Maximize2, Minimize2, Moon, X } from "lucide-react";
import { type CSSProperties, useEffect, useRef, useState } from "react";
import { useT } from "../../i18n";
import type { Bot } from "../../lib/protocol.gen";
import { SYSTEM } from "../../lib/system";
import { useApp } from "../../store/context";
import { Badge } from "../../ui/Badge";
import { Button } from "../../ui/Button";
import { Callout } from "../../ui/Callout";
import { EmptyState } from "../../ui/EmptyState";
import { SidePanel } from "../../ui/SidePanel";
import { BotAvatar } from "../bots/BotAvatar";
import { ComputerFooter } from "../terminal/ComputerDock";
import { AddressBar } from "./AddressBar";
import { AskCallout, ControlPill, HeldNotes, TakeNotes } from "./HandsBars";
import { HandsLayer } from "./HandsLayer";
import { LessonArea, LessonDialog, useLesson } from "./Lesson";
import { LiveView, useCaption } from "./LiveView";
import { TabStrip } from "./TabStrip";
import { useBrowserView } from "./useBrowserView";
import { useHands } from "./useHands";
import { useFitPage, useRoom } from "./useRoom";
import { WindowBar } from "./WindowBar";

/** Around the page in the panel, in px. */
const PAD = 12;
/**
 * The bot's color around the page, in px: on the sides and top, and below,
 * where the pill that says who is in control hangs over its edge. The pill
 * reaches into it less than this, so it never covers the page.
 */
const STAGE = 10;
const STAGE_BOTTOM = 26;

export function BrowserPanel({
  bot,
  take = 0,
  onClose,
}: {
  bot: Bot;
  /** Counts up each time the owner asks to open it in their hands. */
  take?: number;
  onClose(): void;
}) {
  const t = useT().browser;
  const state = useApp((app) => app.browsers[bot.id]) ?? null;
  const status = state?.status ?? "closed";
  const { frame, action, watched } = useBrowserView(bot, true);
  const [expanded, setExpanded] = useState(false);
  const caption = useCaption(bot, action);
  const live = status === "open" && frame !== null;
  // Nobody uses it: its pages stand still until someone does (spec 21.2).
  const resting = status === "open" && state?.resting === true;
  const hands = useHands(bot, state);
  const lesson = useLesson(bot, state);
  const [focused, setFocused] = useState(false);
  const ask = state?.ask ?? null;
  const windowed = state?.window === true;
  // The page takes the shape of the room it has on the desk (spec 21.3), so
  // it fills it at its own size, with no bands of the desk beside it.
  const [room, stage] = useRoom();
  useFitPage(bot, room, watched);
  // A new tab is blank: the owner says where it goes.
  const address = useRef<HTMLInputElement>(null);
  const addTab = async () => {
    if (await hands.newTab()) {
      address.current?.focus();
    }
  };
  // Taking needs the watch first: the daemon ties the hands to it.
  const taken = useRef(0);
  useEffect(() => {
    if (take > taken.current && watched && status === "open") {
      taken.current = take;
      if (!hands.held) {
        void hands.take();
      }
    }
  }, [take, watched, status, hands.held, hands.take]);

  return (
    <SidePanel label={t.panel(bot.name)} name="computer" defaultWidth={600} expanded={expanded}>
      <header className="flex h-11 shrink-0 items-center justify-between border-line border-b pr-1.5 pl-4">
        <h2 className="flex items-center gap-2 font-semibold text-sm">
          {t.heading}
          {resting ? (
            <Badge tone="quiet" icon={Moon} title={t.restingWhy(bot.name)}>
              {t.resting}
            </Badge>
          ) : (
            live && (
              <Badge tone="danger">
                <span aria-hidden className="live-dot" />
                {t.live}
              </Badge>
            )
          )}
        </h2>
        <div className="flex items-center gap-0.5">
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
      {state && (status === "open" || status === "starting") && (
        <>
          {state.tabs.length > 0 && <TabStrip state={state} hands={hands} onAdd={addTab} />}
          <AddressBar bot={bot} state={state} hands={hands} field={address} />
        </>
      )}
      <div
        className="@container flex min-h-0 flex-1 flex-col overflow-y-auto"
        style={{ padding: PAD }}
      >
        {windowed && <WindowBar bot={bot} />}
        {status === "failed" ? (
          <Callout tone="danger" title={t.failedTitle}>
            {t.failedBody(SYSTEM)}
            {state?.error && (
              <details className="mt-2 text-xs">
                <summary>{t.details}</summary>
                <p className="mt-1 font-mono" data-selectable>
                  {state.error}
                </p>
              </details>
            )}
          </Callout>
        ) : status === "closed" && !frame ? (
          !windowed && <Empty bot={bot} />
        ) : (
          <>
            {live && ask && !hands.held && <AskCallout bot={bot} task={ask} hands={hands} />}
            <div
              className="bot-stage flex min-h-40 flex-1 flex-col"
              data-dim={status === "closed" || undefined}
              style={
                {
                  "--bot": bot.color,
                  padding: `${STAGE}px ${STAGE}px ${STAGE_BOTTOM}px`,
                } as CSSProperties
              }
            >
              <div
                ref={stage}
                className="grid min-h-0 flex-1 place-items-center"
                style={{ containerType: "size" }}
              >
                <div className="relative">
                  <LiveView
                    bot={bot}
                    frame={frame}
                    action={action}
                    dim={status === "closed"}
                    held={hands.held}
                  >
                    {hands.held && frame && (
                      <HandsLayer
                        label={t.hands.screen(bot.name)}
                        keysLabel={t.hands.typing}
                        width={frame.width}
                        height={frame.height}
                        send={hands.send}
                        onFocus={setFocused}
                      />
                    )}
                  </LiveView>
                  {(status !== "open" || !frame) && (
                    <Overlay
                      busy={status === "starting" || status === "open"}
                      title={
                        windowed ? t.window.title : status === "closed" ? t.closed : t.starting
                      }
                      body={status === "closed" && !windowed ? t.closedBody : null}
                    />
                  )}
                </div>
              </div>
            </div>
            {(hands.held || (live && !ask)) && (
              <ControlPill key={String(hands.held)} bot={bot} hands={hands} lesson={lesson} />
            )}
            {status === "open" && !hands.held && (
              // Its place is kept while empty, so the page never moves for it.
              <p aria-live="polite" className="mt-2 flex h-5 justify-center text-ink-soft text-sm">
                {caption && (
                  <span key={action?.at} className="flex animate-rise items-center gap-2">
                    <BotAvatar color={bot.color} size={18} mood="working" />
                    <span className="truncate">{caption}</span>
                  </span>
                )}
              </p>
            )}
            {hands.held && <LessonArea lesson={lesson} />}
            {hands.held && lesson.steps === null && (
              <HeldNotes
                bot={bot}
                task={ask}
                tabs={state?.tabs.length ?? 0}
                focused={focused}
                hands={hands}
                onTeach={() => void lesson.start()}
              />
            )}
            {live && !hands.held && !ask && <TakeNotes bot={bot} hands={hands} />}
          </>
        )}
      </div>
      <ComputerFooter />
      {lesson.done && <LessonDialog bot={bot} lesson={lesson} />}
    </SidePanel>
  );
}

function Overlay({ busy, title, body }: { busy: boolean; title: string; body: string | null }) {
  return (
    <div className="absolute inset-0 grid place-items-center rounded-xl">
      <div className="flex animate-fade flex-col items-center gap-1.5 rounded-xl bg-panel/90 px-4 py-3 text-center shadow-lift">
        <p className="flex items-center gap-2 font-medium text-sm">
          {busy && <LoaderCircle aria-hidden size={14} className="animate-spin text-work" />}
          {title}
        </p>
        {body && <p className="max-w-64 text-muted text-xs">{body}</p>}
      </div>
    </div>
  );
}

function Empty({ bot }: { bot: Bot }) {
  const t = useT().browser;
  return (
    <EmptyState color={bot.color} icon={Globe} title={t.emptyTitle(bot.name)} body={t.emptyBody} />
  );
}
