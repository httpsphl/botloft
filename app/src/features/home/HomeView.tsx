// The home (spec 15.1), where the app opens: what waits for the owner in
// every crew on top, answered right here; who works and who replied; what
// the crews did today; then the crews.

import { Hand, Plus } from "lucide-react";
import { type ReactNode, useState } from "react";
import { useShallow } from "zustand/react/shallow";
import { useT } from "../../i18n";
import type { Bot } from "../../lib/protocol.gen";
import { crewList } from "../../store/app";
import { useApp } from "../../store/context";
import { openQuestions } from "../../store/questions";
import { Button } from "../../ui/Button";
import { CrewCard } from "../crews/CrewCard";
import { CrewDialog } from "../crews/CrewDialog";
import { orderedCrews, useCrewOrder } from "../crews/crewOrder";
import { actionableDead } from "../messages/FailedDeliveries";
import { BotLines, useUnread, useWorking } from "./BotLines";
import { NeedsYou } from "./NeedsYou";
import { Today } from "./Today";
import { useRequests } from "./useRequests";

const signedOut = (bots: Record<string, Bot>) =>
  Object.values(bots).filter((bot) => bot.state === "auth_error" && !bot.archivedAt);

export function HomeView() {
  const t = useT();
  const words = t.home;
  const { requests, keys, drop } = useRequests();
  // Oldest first, like the requests they sit with.
  const questions = [...useApp(openQuestions)].reverse();
  const dead = useApp(useShallow(actionableDead));
  const out = useApp(useShallow((state) => signedOut(state.bots)));
  const working = useWorking();
  const unread = useUnread();
  const crews = orderedCrews(useApp(useShallow(crewList)), useCrewOrder());
  const [creating, setCreating] = useState(false);
  const waiting =
    (requests?.length ?? 0) + (keys?.length ?? 0) + questions.length + dead.length + out.length;

  return (
    <section aria-label={words.label} className="flex min-h-0 flex-1 flex-col">
      <header className="flex items-center gap-3 border-line border-b px-5 py-3.5">
        <div className="min-w-0 flex-1">
          <h1 className="truncate font-semibold text-xl tracking-tight">{words.title}</h1>
          <p className="mt-0.5 text-muted text-sm">{words.summary(waiting, working.length)}</p>
        </div>
        <Button variant="primary" icon={Plus} onClick={() => setCreating(true)}>
          {t.crews.newCrew}
        </Button>
      </header>
      <div className="min-h-0 flex-1 overflow-y-auto p-5">
        <div className="mx-auto flex max-w-3xl flex-col gap-8">
          <section aria-labelledby="home-needs-you">
            <h2
              id="home-needs-you"
              className="mb-3 flex items-center gap-1.5 font-semibold text-sm text-warn"
            >
              <Hand aria-hidden size={14} />
              {words.needsYou}
            </h2>
            {requests && (
              <NeedsYou
                requests={requests}
                questions={questions}
                keys={keys ?? []}
                dead={dead}
                signedOut={out}
                onAnswered={drop}
              />
            )}
          </section>
          {working.length > 0 && (
            <Part id="home-working" title={words.working}>
              <BotLines bots={working} time="since" />
            </Part>
          )}
          {unread.length > 0 && (
            <Part id="home-replies" title={words.replies}>
              <BotLines bots={unread} time="reply" />
            </Part>
          )}
          <Part id="home-today" title={words.today.title}>
            <Today />
          </Part>
          <Part id="home-crews" title={words.crews}>
            <ul className="grid grid-cols-[repeat(auto-fill,minmax(220px,1fr))] gap-3">
              {crews.map((crew) => (
                <CrewCard key={crew.id} crew={crew} />
              ))}
            </ul>
          </Part>
        </div>
      </div>
      {creating && <CrewDialog onClose={() => setCreating(false)} />}
    </section>
  );
}

/** A part of the home under a quiet title. */
function Part({ id, title, children }: { id: string; title: string; children: ReactNode }) {
  return (
    <section aria-labelledby={id}>
      <h2 id={id} className="mb-3 font-semibold text-muted text-sm">
        {title}
      </h2>
      {children}
    </section>
  );
}
