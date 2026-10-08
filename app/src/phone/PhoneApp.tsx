// The phone's page once it is connected (spec 28.7): what waits for the
// owner, oldest first, and a small screen about this phone.

import { Settings, Smartphone } from "lucide-react";
import { useState, useSyncExternalStore } from "react";
import { useT } from "../i18n";
import { Callout } from "../ui/Callout";
import { Confirm } from "../ui/Confirm";
import { ApprovalView } from "./ApprovalView";
import type { PhoneApi } from "./client";
import { PhoneButton } from "./parts";
import { QuestionView } from "./QuestionView";

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

export function PhoneApp({ api, onGone }: { api: PhoneApi; onGone?(): void }) {
  const t = useT().phone;
  const state = usePhone(api);
  const [view, setView] = useState<"inbox" | "phone">("inbox");

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
    return <ThisPhone api={api} name={state.name} back={() => setView("inbox")} />;
  }

  const waiting = state.approvals.length + state.questions.length;
  return (
    <Shell>
      <header className="flex items-center justify-between gap-3 py-2">
        <h1 className="font-semibold text-xl">{t.inbox.title}</h1>
        <button
          type="button"
          aria-label={t.inbox.thisPhone}
          onClick={() => setView("phone")}
          className="inline-flex h-11 w-11 items-center justify-center rounded-xl text-ink-soft"
        >
          <Settings aria-hidden size={20} />
        </button>
      </header>

      {state.computer === "offline" && <Callout tone="warn" title={t.inbox.computerOff} />}
      {state.link !== "online" && <Callout tone="info" title={t.inbox.noConnection} />}
      {needsHomeScreen() && <Callout tone="info" title={t.settings.install} />}

      <div className="flex flex-col gap-4">
        {state.approvals.map((card) => (
          <ApprovalView
            key={card.approvalId}
            card={card}
            sending={state.sending.includes(card.approvalId)}
            answer={(allow, note) => api.answerApproval(card.approvalId, allow, note)}
          />
        ))}
        {state.questions.map((card) => (
          <QuestionView
            key={card.questionId}
            card={card}
            sending={state.sending.includes(card.questionId)}
            answer={(text) => api.answerQuestion(card.questionId, text)}
            dismiss={() => api.dismissQuestion(card.questionId)}
          />
        ))}
        {waiting === 0 && (
          <div className="flex flex-col items-center gap-2 py-16 text-center">
            <Smartphone aria-hidden size={28} className="text-muted" />
            {state.loaded || state.computer === "offline" ? (
              <>
                <p className="font-medium text-base">{t.inbox.empty}</p>
                <p className="text-muted text-sm">{t.inbox.emptyBody}</p>
              </>
            ) : (
              <p className="text-muted text-sm">{t.inbox.loading}</p>
            )}
          </div>
        )}
      </div>
    </Shell>
  );
}

function Shell({ children }: { children: React.ReactNode }) {
  return (
    <main className="mx-auto flex min-h-full w-full max-w-md flex-col gap-4 px-4 pt-[max(1rem,env(safe-area-inset-top))] pb-[max(2rem,env(safe-area-inset-bottom))]">
      {children}
    </main>
  );
}

function ThisPhone({ api, name, back }: { api: PhoneApi; name: string; back(): void }) {
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
