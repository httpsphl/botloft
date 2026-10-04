import {
  Ellipsis,
  Files,
  Globe,
  LayoutTemplate,
  PanelRight,
  Pause,
  Play,
  RotateCw,
  ShieldOff,
} from "lucide-react";
import { useT } from "../../i18n";
import type { Bot, Crew } from "../../lib/protocol.gen";
import { useApi, useApp } from "../../store/context";
import { Badge } from "../../ui/Badge";
import { Button } from "../../ui/Button";
import { Menu } from "../../ui/Menu";
import { attempt } from "../../ui/toast";
import { BotAvatar, moodOf } from "./BotAvatar";
import { BotStateBadge } from "./BotStateBadge";
import { useBotActions } from "./botActions";
import { ChiefBadge, isChief } from "./ChiefBadge";

/** The bot's name, state and actions. */
export function BotHeader({
  bot,
  crew,
  detailsOpen,
  onToggleDetails,
  filesOpen,
  freshFiles,
  onToggleFiles,
  browserOpen,
  browsing,
  asking,
  onToggleBrowser,
  screensOpen,
  drawing,
  onToggleScreens,
}: {
  bot: Bot;
  crew: Crew;
  detailsOpen: boolean;
  onToggleDetails(): void;
  filesOpen: boolean;
  /** Files that showed up since the owner last looked. */
  freshFiles: number;
  onToggleFiles(): void;
  browserOpen: boolean;
  /** The bot's browser is open (spec 21.8). */
  browsing: boolean;
  /** The bot asked the owner for a hand in it (spec 21.10). */
  asking: boolean;
  onToggleBrowser(): void;
  screensOpen: boolean;
  /** The bot is writing a screen (spec 22.5). */
  drawing: boolean;
  onToggleScreens(): void;
}) {
  const t = useT();
  const words = t.bots.header;
  const api = useApi();
  const putBot = useApp((state) => state.putBot);
  const chief = isChief(bot, crew);
  const actions = useBotActions(bot, crew);
  const stopped = bot.paused || crew.paused;

  const setPaused = (paused: boolean) =>
    attempt(paused ? words.failed.pause : words.failed.resume, async () =>
      putBot(await api.call("bots.setPaused", { botId: bot.id, paused })),
    );
  const restart = () =>
    attempt(words.failed.restart, async () =>
      putBot(await api.call("bots.restart", { botId: bot.id, fresh: false })),
    );

  return (
    // In a narrow window the handle gives way first, then the button labels.
    <header className="@container flex items-center gap-3 border-line border-b px-5 py-2.5">
      <BotAvatar
        color={bot.color}
        size={36}
        mood={moodOf(bot, crew.paused)}
        botId={bot.id}
        starting={bot.state === "launching"}
      />
      <div className="min-w-0 flex-1">
        <div className="flex items-baseline gap-2">
          <h1 className="truncate font-semibold text-lg tracking-tight">{bot.name}</h1>
          <span
            className="min-w-0 shrink-[100] truncate font-mono text-muted text-sm"
            data-selectable
          >
            @{bot.handle}
          </span>
        </div>
        <div className="mt-0.5 flex items-center gap-3 text-sm">
          <BotStateBadge bot={bot} crewPaused={crew.paused} />
          {chief && <ChiefBadge crew={crew} />}
          {bot.permissionMode === "bypass_permissions" && (
            <Badge tone="danger" icon={ShieldOff} title={t.chat.mode.badgeHint}>
              {t.chat.mode.badge}
            </Badge>
          )}
          <span className="truncate text-muted">{bot.role || words.noRole}</span>
        </div>
      </div>
      {bot.paused ? (
        <Button icon={Play} title={words.resume} onClick={() => setPaused(false)}>
          <Label>{words.resume}</Label>
        </Button>
      ) : (
        <Button icon={Pause} title={words.pause} onClick={() => setPaused(true)}>
          <Label>{words.pause}</Label>
        </Button>
      )}
      <Button icon={RotateCw} title={words.restart} disabled={stopped} onClick={restart}>
        <Label>{words.restart}</Label>
      </Button>
      <span className="relative">
        <Button
          variant={screensOpen ? "secondary" : "ghost"}
          icon={LayoutTemplate}
          label={
            screensOpen
              ? t.screens.hide
              : drawing
                ? `${t.screens.show}: ${t.screens.drawing(bot.name)}`
                : t.screens.show
          }
          aria-pressed={screensOpen}
          onClick={onToggleScreens}
        />
        {drawing && !screensOpen && (
          <span
            aria-hidden
            className="live-dot pointer-events-none absolute top-0.5 right-0.5"
            style={{ background: bot.color }}
          />
        )}
      </span>
      <span className="relative">
        <Button
          variant={browserOpen ? "secondary" : "ghost"}
          icon={Globe}
          label={
            browserOpen
              ? t.browser.hide
              : asking
                ? `${t.browser.show}: ${t.browser.help.needs(bot.name)}`
                : browsing
                  ? `${t.browser.show}: ${t.browser.browsing(bot.name)}`
                  : t.browser.show
          }
          aria-pressed={browserOpen}
          onClick={onToggleBrowser}
        />
        {(browsing || asking) && !browserOpen && (
          <span
            aria-hidden
            className="live-dot pointer-events-none absolute top-0.5 right-0.5"
            style={{ background: asking ? "var(--warn)" : "var(--work)" }}
          />
        )}
      </span>
      <span className="relative">
        <Button
          variant={filesOpen ? "secondary" : "ghost"}
          icon={Files}
          label={
            freshFiles > 0
              ? `${t.files.show}: ${t.files.fresh(freshFiles)}`
              : filesOpen
                ? t.files.hide
                : t.files.show
          }
          aria-pressed={filesOpen}
          onClick={onToggleFiles}
        />
        {freshFiles > 0 && (
          <span
            aria-hidden
            className="pointer-events-none absolute -top-1 -right-1 grid min-w-4 place-items-center rounded-full bg-work px-1 font-semibold text-[10px] text-canvas leading-4"
          >
            {freshFiles}
          </span>
        )}
      </span>
      <Button
        variant={detailsOpen ? "secondary" : "ghost"}
        icon={PanelRight}
        label={detailsOpen ? words.hideDetails : words.showDetails}
        aria-pressed={detailsOpen}
        onClick={onToggleDetails}
      />
      <Menu label={words.more} icon={Ellipsis} items={actions.items} />
      {actions.dialogs}
    </header>
  );
}

/** A button's words, read aloud always but shown only where they fit. */
function Label({ children }: { children: string }) {
  return <span className="sr-only @3xl:not-sr-only">{children}</span>;
}
