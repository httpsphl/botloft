import {
  Ellipsis,
  Files,
  Globe,
  LayoutTemplate,
  PanelRight,
  Pause,
  Play,
  ShieldOff,
} from "lucide-react";
import { useT } from "../../i18n";
import type { Bot, Crew } from "../../lib/protocol.gen";
import { useApi, useApp } from "../../store/context";
import { Badge } from "../../ui/Badge";
import { Button, FitLabel } from "../../ui/Button";
import { Menu } from "../../ui/Menu";
import { attempt } from "../../ui/toast";
import { ConversationControls } from "../chat/ConversationControls";
import { OverviewIcon } from "../overview/icon";
import { BotAvatar, moodOf } from "./BotAvatar";
import { BotStateBadge } from "./BotStateBadge";
import { useBotActions } from "./botActions";
import { ChiefBadge, isChief } from "./ChiefBadge";
import { useMascotLook } from "./mascotLook";
import { useIsMaker } from "./useIsMaker";

/** The bot's name, state and actions. */
export function BotHeader({
  bot,
  crew,
  overviewOpen,
  onToggleOverview,
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
  overviewOpen: boolean;
  onToggleOverview(): void;
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
  const mood = moodOf(bot, crew.paused);
  const look = useMascotLook(bot.id, mood);
  const maker = useIsMaker(bot.id);

  const setPaused = (paused: boolean) =>
    attempt(paused ? words.failed.pause : words.failed.resume, async () =>
      putBot(await api.call("bots.setPaused", { botId: bot.id, paused })),
    );

  return (
    // In a narrow window the handle gives way first, then the button labels.
    // The tabs under it draw the line that closes it.
    <header className="@container flex items-center gap-3 px-5 pt-2.5 pb-1.5">
      <BotAvatar
        color={bot.color}
        size={32}
        mood={mood}
        doing={look.doing}
        face={look.face}
        maker={maker}
        botId={bot.id}
        starting={bot.state === "launching"}
      />
      <div className="min-w-0 flex-1">
        <div className="flex items-baseline gap-2">
          <h1 className="truncate font-semibold text-base tracking-tight">{bot.name}</h1>
          <span
            className="min-w-0 shrink-[100] truncate font-mono text-muted text-xs"
            data-selectable
          >
            @{bot.handle}
          </span>
        </div>
        <div className="flex items-center gap-3 text-xs">
          <BotStateBadge bot={bot} crewPaused={crew.paused} sleep />
          {chief && <ChiefBadge crew={crew} />}
          {bot.permissionMode === "bypass_permissions" && (
            <Badge tone="danger" icon={ShieldOff} title={t.chat.mode.badgeHint}>
              {t.chat.mode.badge}
            </Badge>
          )}
          <span className="truncate text-muted">{bot.role || words.noRole}</span>
        </div>
      </div>
      <ConversationControls bot={bot} />
      {bot.paused ? (
        <Button variant="ghost" icon={Play} title={words.resume} onClick={() => setPaused(false)}>
          <FitLabel>{words.resume}</FitLabel>
        </Button>
      ) : (
        <Button variant="ghost" icon={Pause} title={words.pause} onClick={() => setPaused(true)}>
          <FitLabel>{words.pause}</FitLabel>
        </Button>
      )}
      {/* The panels beside the chat, one group, the open ones raised. */}
      <fieldset aria-label={words.panels} className="m-0 flex items-center gap-0.5 border-0 p-0">
        <span className="relative">
          <Button
            size="sm"
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
            size="sm"
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
            size="sm"
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
          size="sm"
          variant={overviewOpen ? "secondary" : "ghost"}
          icon={OverviewIcon}
          label={overviewOpen ? t.overview.hide : t.overview.show}
          aria-pressed={overviewOpen}
          onClick={onToggleOverview}
        />
        <Button
          size="sm"
          variant={detailsOpen ? "secondary" : "ghost"}
          icon={PanelRight}
          label={detailsOpen ? words.hideDetails : words.showDetails}
          aria-pressed={detailsOpen}
          onClick={onToggleDetails}
        />
      </fieldset>
      <Menu label={words.more} icon={Ellipsis} items={actions.items} />
      {actions.dialogs}
    </header>
  );
}
