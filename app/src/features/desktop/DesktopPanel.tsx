// A bot's use of the owner's desktop beside its chat (spec 24.9): the
// window it read or acted in last, live, what it did, and Stop. Stopped,
// the panel says so and offers to let it go on.

import { Monitor, Play, Square, X } from "lucide-react";
import type { CSSProperties } from "react";
import { useT } from "../../i18n";
import type { Bot, DesktopAction } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";
import { Badge } from "../../ui/Badge";
import { Button } from "../../ui/Button";
import { Callout } from "../../ui/Callout";
import { EmptyState } from "../../ui/EmptyState";
import { SidePanel } from "../../ui/SidePanel";
import { attempt } from "../../ui/toast";
import { BotAvatar } from "../bots/BotAvatar";
import { ComputerFooter } from "../terminal/ComputerDock";
import { useDesktopView } from "./useDesktopView";

type Words = ReturnType<typeof useT>["desktop"]["panel"];

/** What the bot did, in the owner's language. */
export function actionText(action: DesktopAction | null, t: Words): string {
  if (!action) {
    return t.read;
  }
  const target = action.target.trim();
  switch (action.kind) {
    case "click":
      return target ? t.click(target) : t.clickSomething;
    case "type":
      return target ? t.type(target) : t.typeSomething;
    case "select":
      return t.select(action.option ?? "");
    case "scroll":
      return target ? t.scroll(target) : t.scrollSomething;
  }
}

export function DesktopPanel({ bot, onClose }: { bot: Bot; onClose(): void }) {
  const t = useT().desktop.panel;
  const api = useApi();
  const { state, frame } = useDesktopView(bot.id);
  const stopped = state?.stopped === true;
  const window = state?.window ?? null;
  const stop = () => attempt(t.stopFailed, () => api.call("desktop.stop", { botId: bot.id }));
  const resume = () => attempt(t.resumeFailed, () => api.call("desktop.resume", { botId: bot.id }));

  return (
    <SidePanel label={t.label(bot.name)} name="computer" defaultWidth={600}>
      <header className="flex h-11 shrink-0 items-center justify-between border-line border-b pr-1.5 pl-4">
        <h2 className="flex items-center gap-2 font-semibold text-sm">
          {t.heading}
          {stopped ? (
            <Badge tone="warn">{t.stoppedBadge}</Badge>
          ) : (
            frame &&
            window && (
              <Badge tone="danger">
                <span aria-hidden className="live-dot" />
                {t.live}
              </Badge>
            )
          )}
        </h2>
        <div className="flex items-center gap-0.5">
          {state && !stopped && (
            <Button variant="ghost" size="sm" icon={Square} onClick={() => void stop()}>
              {t.stop}
            </Button>
          )}
          <Button variant="ghost" size="sm" icon={X} label={t.close} onClick={onClose} />
        </div>
      </header>
      <div className="flex min-h-0 flex-1 flex-col gap-3 overflow-y-auto p-3">
        {stopped && (
          <Callout tone="warn" title={t.stoppedTitle(bot.name)}>
            <p>{t.stoppedBody(bot.name)}</p>
            <div className="mt-2.5">
              <Button variant="primary" size="sm" icon={Play} onClick={() => void resume()}>
                {t.resume}
              </Button>
            </div>
          </Callout>
        )}
        {state && !window && (
          <EmptyState
            color={bot.color}
            icon={Monitor}
            title={t.emptyTitle(bot.name)}
            body={t.emptyBody}
          />
        )}
        {window && (
          <>
            <p className="flex min-w-0 items-baseline gap-2 text-sm">
              <span className="shrink-0 font-medium">{window.app}</span>
              <span className="truncate text-muted" data-selectable>
                {window.title}
              </span>
            </p>
            <div
              className="bot-stage grid min-h-40 flex-1 place-items-center p-2.5"
              style={{ "--bot": bot.color } as CSSProperties}
            >
              {frame ? (
                <img
                  src={`data:image/jpeg;base64,${frame.data}`}
                  alt={window.title}
                  draggable={false}
                  className="max-h-full max-w-full select-none rounded-lg object-contain shadow-lift"
                />
              ) : (
                <p className="text-sm text-white/80">{t.waiting}</p>
              )}
            </div>
            <p aria-live="polite" className="flex items-center gap-2 text-ink-soft text-sm">
              <BotAvatar color={bot.color} size={18} mood={stopped ? "sleeping" : "working"} />
              <span className="truncate">{actionText(state?.action ?? null, t)}</span>
            </p>
          </>
        )}
        <p className="mt-auto text-muted text-xs">{t.shortcut}</p>
      </div>
      <ComputerFooter />
    </SidePanel>
  );
}
