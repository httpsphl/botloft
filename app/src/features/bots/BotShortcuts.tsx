// The ways from an agent's details to what is set for it elsewhere (spec
// 15.1): its desktop apps, the owner's Chrome, keys, tools and skills each
// have a page in Settings, and a row here says what it has and opens that
// page on it.

import {
  AppWindow,
  ChevronRight,
  KeySquare,
  type LucideIcon,
  Monitor,
  Plug,
  Sparkles,
} from "lucide-react";
import { useT } from "../../i18n";
import type { Bot } from "../../lib/protocol.gen";
import { useApp } from "../../store/context";
import { useConnections } from "../connections/useConnections";
import { useDesktopGrants } from "../desktop/useDesktopGrants";
import { useKeys } from "../keys/AgentKeys";
import { usePermissions } from "../permissions/usePermissions";
import { useSkills } from "../skills/useSkills";

function Shortcut({
  icon: Icon,
  title,
  summary,
  onOpen,
}: {
  icon: LucideIcon;
  title: string;
  /** What it has now; null while it is read. */
  summary: string | null;
  onOpen(): void;
}) {
  return (
    <li>
      <button
        type="button"
        onClick={onOpen}
        className="flex w-full min-w-0 items-center gap-3 rounded-lg px-2 py-2 text-left transition-colors hover:bg-line/60"
      >
        <span
          aria-hidden
          className="grid size-7 shrink-0 place-items-center rounded-md bg-panel text-ink-soft"
        >
          <Icon size={15} />
        </span>
        <span className="min-w-0 flex-1">
          <span className="block font-medium text-sm">{title}</span>
          <span className="block truncate text-muted text-xs">{summary ?? "…"}</span>
        </span>
        <ChevronRight aria-hidden size={15} className="shrink-0 text-muted" />
      </button>
    </li>
  );
}

const names = (list: string[], none: string) => (list.length === 0 ? none : list.join(", "));

export function BotShortcuts({ bot }: { bot: Bot }) {
  const t = useT();
  const words = t.bots.details.shortcuts;
  const openPermissions = useApp((state) => state.openPermissions);
  const openSettingsPart = useApp((state) => state.openSettingsPart);
  const grants = useDesktopGrants(bot.id);
  const permissions = usePermissions();
  const chrome = permissions ? permissions.chrome.some((grant) => grant.botId === bot.id) : null;
  const [keys] = useKeys(bot.id);
  const tools = useConnections();
  const { skills } = useSkills(t.skills.shareFailed);
  const claude = bot.agent === "claude";
  const mine = tools?.bots.find((link) => link.botId === bot.id);
  const toolNames = tools
    ? [
        // claude.ai's tools reach only agents on Claude Code (spec 34.4).
        ...(claude ? bot.connectors : []),
        ...tools.servers
          .filter((server) => mine?.serverIds.includes(server.id))
          .map((server) => server.name),
      ]
    : null;
  const skillNames = skills
    ?.filter((skill) => skill.botIds.includes(bot.id))
    .map((skill) => skill.name);

  return (
    <section aria-label={words.title} className="flex flex-col gap-2">
      <h3 className="font-semibold text-sm">{words.title}</h3>
      <ul className="flex flex-col gap-0.5 rounded-xl bg-bubble p-1">
        <Shortcut
          icon={Monitor}
          title={words.desktop}
          summary={grants === null ? null : words.desktopCount(grants.length)}
          onOpen={() => openPermissions({ tab: "desktop", botId: bot.id })}
        />
        {claude && (
          <Shortcut
            icon={AppWindow}
            title={t.chrome.details.label}
            summary={chrome === null ? null : chrome ? words.chromeOn : words.chromeOff}
            onOpen={() => openPermissions({ tab: "chrome", botId: bot.id })}
          />
        )}
        <Shortcut
          icon={KeySquare}
          title={words.keys}
          summary={keys === null ? null : words.keysCount(keys.length)}
          onOpen={() => openSettingsPart("keys", bot.id)}
        />
        <Shortcut
          icon={Plug}
          title={words.tools}
          summary={toolNames === null ? null : names(toolNames, words.none)}
          onOpen={() => openSettingsPart("tools")}
        />
        {claude && (
          <Shortcut
            icon={Sparkles}
            title={words.skills}
            summary={skillNames === undefined ? null : names(skillNames, words.none)}
            onOpen={() => openSettingsPart("skills")}
          />
        )}
      </ul>
    </section>
  );
}
