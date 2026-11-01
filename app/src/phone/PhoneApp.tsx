// The phone's page once it is connected (spec 28.7): what waits for the
// owner, oldest first, and a small screen about this phone.

import { Settings } from "lucide-react";
import { useEffect, useRef, useState, useSyncExternalStore } from "react";
import { useT } from "../i18n";
import { Callout } from "../ui/Callout";
import { Confirm } from "../ui/Confirm";
import { ChatsList, CrewChips, isUnread } from "./ChatsList";
import { ChatView } from "./ChatView";
import type { PhoneApi, PhoneState } from "./client";
import { failureOf } from "./FailedRuns";
import { HomeTab } from "./HomeTab";
import { Notices } from "./NoticesControl";
import { type LockApi, PinNudge, PinSettings } from "./PinSettings";
import { PhoneButton } from "./parts";

export function usePhone(api: PhoneApi) {
  return useSyncExternalStore(api.subscribe, api.getState);
}

/** Safari on an iPhone gives notices only to a page on the Home Screen. */
function needsHomeScreen(): boolean {
  if (typeof navigator === "undefined") {
    return false;
  }
  const ios = /iPad|iPhone|iPod/.test(navigator.userAgent);
  const standalone = (navigator as Navigator & { standalone?: boolean }).standalone === true;
  return ios && !standalone && !window.matchMedia?.("(display-mode: standalone)").matches;
}

/**
 * The phone's own back gesture (or button) closes what was opened, instead
 * of leaving the app: opening pushes one step of history, and going back
 * pops it. Closing from the page takes the step back too.
 */
function useBackGesture(open: boolean, back: () => void) {
  const pushed = useRef(false);
  const latest = useRef(back);
  latest.current = back;
  useEffect(() => {
    if (open && !pushed.current) {
      history.pushState({ botloft: true }, "");
      pushed.current = true;
    } else if (!open && pushed.current) {
      pushed.current = false;
      history.back();
    }
  }, [open]);
  useEffect(() => {
    const onPop = () => {
      if (pushed.current) {
        pushed.current = false;
        latest.current();
      }
    };
    window.addEventListener("popstate", onPop);
    return () => window.removeEventListener("popstate", onPop);
  }, []);
}

export function PhoneApp({
  api,
  onGone,
  lock,
}: {
  api: PhoneApi;
  onGone?(): void;
  /** The PIN (spec 28.13); without it the page offers none. */
  lock?: LockApi;
}) {
  const t = useT().phone;
  const state = usePhone(api);
  const [view, setView] = useState<"inbox" | "phone">("inbox");
  const [tab, setTab] = useState<"home" | "chats">("home");
  const [crew, setCrew] = useState<string | null>(null);
  useBackGesture(state.convo !== null, () => void api.closeChat());
  useBackGesture(view === "phone", () => setView("inbox"));
  useFreshHome(api, state, tab === "home" && view === "inbox" && state.convo === null);

  if (state.session !== "ok") {
    return (
      <Shell>
        <div className="flex flex-col gap-4 pt-10">
          <h1 className="font-semibold text-xl">
            {state.session === "revoked" ? t.cut.revokedTitle : t.cut.leftTitle}
          </h1>
          <p className="text-ink-soft">{t.cut.body}</p>
          <PhoneButton
            look="primary"
            onClick={() => void api.disconnect().then(() => onGone?.())}
            className="flex-none"
          >
            {t.cut.ok}
          </PhoneButton>
        </div>
      </Shell>
    );
  }
  if (view === "phone") {
    return <ThisPhone api={api} name={state.name} lock={lock} back={() => setView("inbox")} />;
  }

  if (state.convo) {
    return (
      <Shell>
        <ChatView api={api} state={state} back={() => void api.closeChat()} />
      </Shell>
    );
  }

  const failures = state.chats.filter((line) => failureOf(line, state.seen) !== null).length;
  const waiting = state.approvals.length + state.questions.length + failures;
  const news = state.chats.some((line) => isUnread(line, state.seen));
  return (
    <Shell>
      <div className="sticky top-0 z-10 -mx-4 flex flex-col gap-3 bg-canvas px-4 pt-1 pb-2">
        <header className="flex items-center justify-between gap-3 py-1">
          <h1 className="font-semibold text-xl">{tab === "chats" ? t.tabs.chats : t.tabs.home}</h1>
          <button
            type="button"
            aria-label={t.inbox.thisPhone}
            onClick={() => setView("phone")}
            className="inline-flex h-11 w-11 items-center justify-center rounded-xl text-ink-soft"
          >
            <Settings aria-hidden size={20} />
          </button>
        </header>

        <div role="tablist" className="flex gap-2">
          <Tab on={tab === "home"} pick={() => setTab("home")}>
            {t.tabs.home}
            {waiting > 0 && ` (${waiting})`}
          </Tab>
          <Tab on={tab === "chats"} pick={() => setTab("chats")}>
            {t.tabs.chats}
            {news && (
              <span
                role="img"
                aria-label={t.tabs.newReply}
                className="ml-2 inline-block h-2.5 w-2.5 rounded-full bg-work"
              />
            )}
          </Tab>
        </div>
        {tab === "chats" && (
          <CrewChips chats={state.chats} seen={state.seen} crew={crew} pick={setCrew} />
        )}
      </div>

      {state.computer === "offline" && <Callout tone="warn" title={t.inbox.computerOff} />}
      {state.link !== "online" && <Callout tone="info" title={t.inbox.noConnection} />}
      {needsHomeScreen() && <Callout tone="info" title={t.settings.install} />}
      <Notices api={api} invite />
      {lock && <PinNudge lock={lock} open={() => setView("phone")} />}

      {tab === "chats" && (
        <ChatsList
          chats={state.chats}
          loaded={state.chatsLoaded}
          seen={state.seen}
          crew={crew}
          open={(botId) => void api.openChat(botId)}
        />
      )}

      {/* Kept while hidden, so a note half written on a card is still there. */}
      <div className={tab === "home" ? "contents" : undefined} hidden={tab !== "home"}>
        <HomeTab
          api={api}
          state={state}
          waiting={waiting}
          open={(botId) => {
            setTab("chats");
            void api.openChat(botId);
          }}
        />
      </div>
    </Shell>
  );
}

/** How long the Home waits after an agent starts or stops before asking again. */
const SETTLE_MS = 1500;

/**
 * The Home's numbers are asked for again when it shows, when the page comes
 * back to the front, and a moment after an agent starts or stops working.
 */
function useFreshHome(api: PhoneApi, state: PhoneState, showing: boolean) {
  const working = state.chats.filter((line) => line.state === "busy").length;
  const online = state.link === "online";
  // biome-ignore lint/correctness/useExhaustiveDependencies: an agent that starts or stops asks again
  useEffect(() => {
    if (!showing || !online) {
      return;
    }
    const timer = setTimeout(() => void api.askHome(), SETTLE_MS);
    return () => clearTimeout(timer);
  }, [api, showing, online, working]);
  useEffect(() => {
    if (!showing) {
      return;
    }
    const onShow = () => {
      if (document.visibilityState === "visible") {
        void api.askHome();
      }
    };
    document.addEventListener("visibilitychange", onShow);
    return () => document.removeEventListener("visibilitychange", onShow);
  }, [api, showing]);
}

function Tab({ on, pick, children }: { on: boolean; pick(): void; children: React.ReactNode }) {
  return (
    <button
      type="button"
      role="tab"
      aria-selected={on}
      onClick={pick}
      className={`inline-flex h-11 flex-1 items-center justify-center rounded-xl border font-medium text-base ${on ? "border-ink bg-ink text-canvas" : "border-line-strong bg-panel text-ink"}`}
    >
      {children}
    </button>
  );
}

function Shell({ children }: { children: React.ReactNode }) {
  return (
    <main className="mx-auto flex min-h-dvh w-full max-w-md flex-col gap-4 px-4 pt-[max(1rem,env(safe-area-inset-top))] pb-[max(2rem,env(safe-area-inset-bottom))]">
      {children}
    </main>
  );
}

function ThisPhone({
  api,
  name,
  lock,
  back,
}: {
  api: PhoneApi;
  name: string;
  lock: LockApi | undefined;
  back(): void;
}) {
  const t = useT().phone;
  const [leaving, setLeaving] = useState(false);
  return (
    <Shell>
      <header className="flex items-center gap-3 py-2">
        <PhoneButton onClick={back} className="flex-none">
          {t.settings.back}
        </PhoneButton>
        <h1 className="font-semibold text-xl">{t.settings.title}</h1>
      </header>
      <p className="text-ink-soft">
        <span className="text-muted">{t.settings.name}: </span>
        {name}
      </p>
      {needsHomeScreen() && <Callout tone="info" title={t.settings.install} />}
      <p className="text-ink-soft text-sm">{t.chats.reach}</p>
      <Notices api={api} />
      {lock && <PinSettings lock={lock} />}
      <PhoneButton look="danger" onClick={() => setLeaving(true)} className="flex-none">
        {t.settings.disconnect}
      </PhoneButton>
      {leaving && (
        <Confirm
          title={t.settings.disconnectTitle}
          confirmLabel={t.settings.disconnect}
          onConfirm={() => api.disconnect()}
          onClose={() => setLeaving(false)}
        >
          {t.settings.disconnectText}
        </Confirm>
      )}
    </Shell>
  );
}
