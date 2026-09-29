// The bot's own browser beside its chat (spec 21.8): the page it is on,
// live, with its cursor and a line about what it just did. Watching starts
// when the panel opens and stops when it closes. The owner can take it into
// their own hands and give it back (spec 21.10).

import { ExternalLink, Globe, LoaderCircle, Lock, Maximize2, Minimize2, X } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { useT } from "../../i18n";
import type { Bot, BrowserState } from "../../lib/protocol.gen";
import { useApp, useHost } from "../../store/context";
import { Button } from "../../ui/Button";
import { Callout } from "../../ui/Callout";
import { SidePanel } from "../../ui/SidePanel";
import { attempt } from "../../ui/toast";
import { BotAvatar } from "../bots/BotAvatar";
import { AskCallout, HeldBar, TakeBar } from "./HandsBars";
import { HandsLayer } from "./HandsLayer";
import { LiveView, useCaption } from "./LiveView";
import { useBrowserView } from "./useBrowserView";
import { useHands } from "./useHands";

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
  const hands = useHands(bot, state);
  const [focused, setFocused] = useState(false);
  const ask = state?.ask ?? null;
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
    <SidePanel label={t.panel(bot.name)} name="browser" defaultWidth={560} expanded={expanded}>
      <header className="flex h-11 shrink-0 items-center justify-between border-line border-b pr-1.5 pl-4">
        <h2 className="flex items-center gap-2 font-semibold text-sm">
          {t.heading}
          {live && (
            <span className="inline-flex items-center gap-1.5 rounded-full bg-danger/10 px-2 py-0.5 font-medium text-danger text-xs">
              <span aria-hidden className="live-dot" />
              {t.live}
            </span>
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
      {state && (status === "open" || status === "starting") && <AddressBar state={state} />}
      <div className="flex min-h-0 flex-1 flex-col overflow-y-auto p-3">
        {status === "failed" ? (
          <Callout tone="danger" title={t.failedTitle}>
            {t.failedBody}
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
          <Empty bot={bot} />
        ) : (
          <>
            {live && ask && !hands.held && <AskCallout bot={bot} task={ask} hands={hands} />}
            {hands.held && <HeldBar bot={bot} task={ask} focused={focused} hands={hands} />}
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
                  title={status === "closed" ? t.closed : t.starting}
                  body={status === "closed" ? t.closedBody : null}
                />
              )}
            </div>
            {status === "open" && caption && !hands.held && (
              <p
                key={action?.at}
                aria-live="polite"
                className="mt-3 flex animate-rise items-center gap-2 text-ink-soft text-sm"
              >
                <BotAvatar color={bot.color} size={18} mood="working" />
                <span className="truncate">{caption}</span>
              </p>
            )}
            {live && !hands.held && !ask && <TakeBar bot={bot} hands={hands} />}
          </>
        )}
      </div>
    </SidePanel>
  );
}

function AddressBar({ state }: { state: BrowserState }) {
  const t = useT().browser;
  const host = useHost();
  const url = state.url && state.url !== "about:blank" ? state.url : null;
  const web = url !== null && /^https?:\/\//i.test(url);
  const Icon = state.loading ? LoaderCircle : url?.startsWith("https://") ? Lock : Globe;
  return (
    <div className="flex h-11 shrink-0 items-center gap-2 border-line border-b px-3">
      <div className="flex min-w-0 flex-1 items-center gap-2 rounded-full bg-sunken px-3 py-1.5">
        <Icon
          aria-label={state.loading ? t.loading : undefined}
          aria-hidden={!state.loading}
          size={13}
          className={`shrink-0 ${state.loading ? "animate-spin text-work" : "text-muted"}`}
        />
        <input
          readOnly
          type="text"
          aria-label={t.address}
          title={url ?? undefined}
          value={url ?? state.title ?? ""}
          className="min-w-0 flex-1 truncate bg-transparent font-mono text-ink-soft text-xs outline-none"
        />
      </div>
      {state.tabs > 1 && <span className="shrink-0 text-muted text-xs">{t.tabs(state.tabs)}</span>}
      {web && url && (
        <Button
          variant="ghost"
          size="sm"
          icon={ExternalLink}
          label={t.openOutside}
          onClick={() => attempt(t.openFailed, () => host.openUrl(url))}
        />
      )}
    </div>
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
    <div className="flex flex-col items-center gap-3 px-6 py-14 text-center">
      <span className="relative">
        <BotAvatar color={bot.color} size={44} mood="idle" />
        <Globe
          aria-hidden
          size={18}
          className="absolute -right-2 -bottom-1 rounded-full bg-panel p-0.5 text-muted"
        />
      </span>
      <p className="font-semibold text-sm">{t.emptyTitle(bot.name)}</p>
      <p className="max-w-80 text-muted text-sm">{t.emptyBody}</p>
    </div>
  );
}
