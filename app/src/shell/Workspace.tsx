// The connected app: title bar, crews on the left, the selection on the
// right.

import { ChevronRight, LoaderCircle, TriangleAlert } from "lucide-react";
import type { ReactNode } from "react";
import { BotAvatar } from "../features/bots/BotAvatar";
import { BotView } from "../features/bots/BotView";
import { CrewsOverview } from "../features/crews/CrewsOverview";
import { CrewView } from "../features/crews/CrewView";
import { Sidebar } from "../features/crews/Sidebar";
import { AwayNotice } from "../features/desktop/AwayNotice";
import { useLocaleToDaemon } from "../features/desktop/useLocaleToDaemon";
import { FailedDeliveries } from "../features/messages/FailedDeliveries";
import { ClaudeCodeHelp } from "../features/onboarding/ClaudeCodeHelp";
import { SignInButton } from "../features/onboarding/SignIn";
import { Welcome } from "../features/onboarding/Welcome";
import { QuestionBox } from "../features/questions/QuestionBox";
import { RoutinesPage } from "../features/routines/RoutinesPage";
import { SearchPage } from "../features/search/SearchPage";
import { useSearchKey } from "../features/search/searchKey";
import { UpdateButton } from "../features/updates/UpdateButton";
import { useT } from "../i18n";
import { useApp } from "../store/context";
import { Callout } from "../ui/Callout";
import { useBotAlerts } from "./alerts";
import { useAttentionMark } from "./attention";
import { useFolderNotices } from "./folders";
import { Rail } from "./Rail";
import { SidebarSlot, SidebarToggle } from "./sidebarToggle";
import { useOpenAtSignIn } from "./signIn";
import { useAppSounds } from "./sounds";
import { TitleBar } from "./TitleBar";
import { useTray } from "./tray";

export function Workspace() {
  const t = useT();
  useLocaleToDaemon();
  const loaded = useApp((state) => state.loaded);
  const loadError = useApp((state) => state.loadError);
  const hasCrews = useApp((state) => Object.keys(state.crews).length > 0);
  const runtimeError = useApp((state) => state.system?.runtimeError ?? null);
  const signedOut = useApp((state) => state.system?.claudeSignedIn === false);
  useAttentionMark();
  useTray();
  useBotAlerts();
  useAppSounds();
  useFolderNotices();
  useOpenAtSignIn();
  useSearchKey();

  let main: ReactNode;
  if (loadError) {
    main = (
      <div className="p-6">
        <Callout tone="danger" title={t.shell.loadFailed}>
          {loadError}
        </Callout>
      </div>
    );
  } else if (!loaded) {
    main = null;
  } else if (!hasCrews) {
    // The sidebar stays, with the account area, before the first crew too.
    main = (
      <div className="flex min-h-0 flex-1">
        <Rail />
        <Sidebar />
        <Welcome />
      </div>
    );
  } else {
    main = (
      <div className="flex min-h-0 flex-1">
        <Rail />
        <SidebarSlot>
          <Sidebar />
        </SidebarSlot>
        <main className="flex min-w-0 flex-1 flex-col">
          {runtimeError ? (
            <div className="border-line border-b p-3">
              <Callout tone="danger" title={t.shell.botsCantStart}>
                <ClaudeCodeHelp error={runtimeError} />
              </Callout>
            </div>
          ) : (
            signedOut && (
              <div className="border-line border-b p-3">
                <Callout tone="danger" title={t.shell.signIn.title}>
                  {t.shell.signIn.body}
                  <SignInButton />
                </Callout>
              </div>
            )
          )}
          <AwayNotice />
          <Selection />
        </main>
      </div>
    );
  }

  return (
    <div className="flex h-full flex-col">
      <TitleBar
        status={
          <>
            <UpdateButton />
            <FailedDeliveries />
            <ConnectionStatus />
          </>
        }
      >
        {hasCrews && <SidebarToggle />}
        {hasCrews && <Breadcrumb />}
      </TitleBar>
      {main}
    </div>
  );
}

/** What the main pane shows; another bot, crew or page rises in. */
function Selection() {
  const page = useApp((state) => state.page);
  const crew = useApp((state) => (state.selectedCrewId ? state.crews[state.selectedCrewId] : null));
  const bot = useApp((state) => (state.selectedBotId ? state.bots[state.selectedBotId] : null));
  let view: ReactNode;
  if (page === "questions") {
    view = <QuestionBox />;
  } else if (page === "search") {
    view = <SearchPage />;
  } else if (page === "routines") {
    view = <RoutinesPage />;
  } else if (crew && bot) {
    view = <BotView bot={bot} crew={crew} />;
  } else if (crew) {
    view = <CrewView crew={crew} />;
  } else {
    view = <CrewsOverview />;
  }
  return (
    <div
      key={page ?? bot?.id ?? crew?.id ?? "crews"}
      className="pane-rise flex min-h-0 flex-1 flex-col"
    >
      {view}
    </div>
  );
}

function Breadcrumb() {
  const t = useT();
  const crew = useApp((state) => (state.selectedCrewId ? state.crews[state.selectedCrewId] : null));
  const bot = useApp((state) => (state.selectedBotId ? state.bots[state.selectedBotId] : null));
  const selectCrew = useApp((state) => state.selectCrew);
  const page = useApp((state) => state.page);
  if (page) {
    return (
      <>
        <ChevronRight aria-hidden size={14} className="text-muted" />
        <span className="truncate text-ink-soft">
          {page === "questions"
            ? t.questions.box.label
            : page === "routines"
              ? t.routines.page.title
              : t.search.label}
        </span>
      </>
    );
  }
  if (!crew) {
    // Only shown with crews: the overview is where "no crew" leads.
    return (
      <>
        <ChevronRight aria-hidden size={14} className="text-muted" />
        <span className="truncate text-ink-soft">{t.crews.sidebar.label}</span>
      </>
    );
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
  const t = useT();
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
      {lost ? t.shell.connection.lost : t.shell.connection.reconnecting}
    </span>
  );
}
