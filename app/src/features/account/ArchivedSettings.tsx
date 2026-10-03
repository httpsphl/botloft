// Settings, "Archived": the bots and crews the owner archived, which the
// rest of the app does not show, with a way to delete each for good
// (spec 7.6).

import { Users } from "lucide-react";
import { type ReactNode, useEffect, useState } from "react";
import { useT } from "../../i18n";
import { errorText } from "../../lib/api";
import { fromNow } from "../../lib/format";
import type { Archive, Bot, Crew } from "../../lib/protocol.gen";
import { useApi, useApp } from "../../store/context";
import { Button } from "../../ui/Button";
import { Callout } from "../../ui/Callout";
import { IconBadge } from "../../ui/ChatCard";
import { BotAvatar } from "../bots/BotAvatar";
import { DeleteBot } from "../bots/DeleteBot";
import { DeleteCrew } from "../crews/DeleteCrew";
import { Section } from "./settingsParts";

type Asking = { bot: Bot } | { crew: Crew } | null;

export function ArchivedSettings() {
  const s = useT().account.settings;
  const api = useApi();
  const active = useApp((state) => state.crews);
  const [archive, setArchive] = useState<Archive | null>(null);
  const [failed, setFailed] = useState<string | null>(null);
  const [asking, setAsking] = useState<Asking>(null);

  useEffect(() => {
    let alive = true;
    api.call("archive.list").then(
      (list) => alive && setArchive(list),
      (error) => alive && setFailed(errorText(error)),
    );
    return () => {
      alive = false;
    };
  }, [api]);

  if (failed) {
    return (
      <Section title={s.archived}>
        <Callout tone="danger" title={s.archivedLoadFailed}>
          {failed}
        </Callout>
      </Section>
    );
  }
  if (!archive) {
    return <Section title={s.archived}>{null}</Section>;
  }

  // A bot of an archived crew goes with its crew; the others are listed.
  const botsOf = (crew: Crew) => archive.bots.filter((bot) => bot.crewId === crew.id);
  const bots = archive.bots.filter((bot) => !archive.crews.some((crew) => crew.id === bot.crewId));
  const forget = (gone: Asking) =>
    setArchive({
      crews: archive.crews.filter((crew) => !(gone && "crew" in gone && gone.crew.id === crew.id)),
      bots: archive.bots.filter(
        (bot) =>
          !(gone && "bot" in gone && gone.bot.id === bot.id) &&
          !(gone && "crew" in gone && gone.crew.id === bot.crewId),
      ),
    });

  return (
    <Section title={s.archived}>
      <p className="text-ink-soft text-sm leading-relaxed">{s.archivedIntro}</p>
      {archive.crews.length + bots.length === 0 ? (
        <p className="text-muted text-sm">{s.archivedEmpty}</p>
      ) : (
        <ul aria-label={s.archived} className="flex flex-col divide-y divide-line">
          {archive.crews.map((crew) => (
            <Row
              key={crew.id}
              icon={<IconBadge icon={Users} tone="quiet" />}
              name={crew.name}
              line={s.archivedCrew(botsOf(crew).length, fromNow(crew.archivedAt ?? 0))}
              onDelete={() => setAsking({ crew })}
            />
          ))}
          {bots.map((bot) => (
            <Row
              key={bot.id}
              icon={<BotAvatar color={bot.color} size={28} />}
              name={bot.name}
              line={s.archivedBot(active[bot.crewId]?.name ?? "", fromNow(bot.archivedAt ?? 0))}
              onDelete={() => setAsking({ bot })}
            />
          ))}
        </ul>
      )}
      {asking && "bot" in asking && (
        <DeleteBot
          bot={asking.bot}
          onClose={() => setAsking(null)}
          onDeleted={() => forget(asking)}
        />
      )}
      {asking && "crew" in asking && (
        <DeleteCrew
          crew={asking.crew}
          bots={botsOf(asking.crew).length}
          onClose={() => setAsking(null)}
          onDeleted={() => forget(asking)}
        />
      )}
    </Section>
  );
}

function Row({
  icon,
  name,
  line,
  onDelete,
}: {
  icon: ReactNode;
  name: string;
  line: string;
  onDelete(): void;
}) {
  const s = useT().account.settings;
  return (
    <li className="flex items-center gap-3 py-2">
      {icon}
      <div className="min-w-0 flex-1">
        <p className="truncate font-medium text-sm">{name}</p>
        <p className="truncate text-muted text-xs">{line}</p>
      </div>
      <Button variant="danger" size="sm" label={s.deleteArchivedOne(name)} onClick={onDelete}>
        {s.deleteArchived}
      </Button>
    </li>
  );
}
