// One thing the owner let an agent do (spec 15.1), the same in the agent's
// details and on the Permissions page: what it is, with a button that takes
// it back so the agent asks again the next time.

import { Globe, Monitor, Network, X } from "lucide-react";
import type { ReactNode } from "react";
import { useT } from "../../i18n";
import { day } from "../../lib/format";
import type { AllowRule, Bot, BrowserSite, CrewAccess, DesktopGrant } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";
import { Button } from "../../ui/Button";
import { attempt } from "../../ui/toast";
import { toolTitle } from "../chat/toolNames";
import { RealInput } from "../desktop/RealInput";
import { Unattended } from "../desktop/Unattended";

function Row({
  icon,
  remove,
  onRemove,
  children,
}: {
  icon?: ReactNode;
  /** The take-back button's name, for whoever cannot see the X. */
  remove: string;
  onRemove(): void;
  children: ReactNode;
}) {
  return (
    <li className="flex items-start gap-2 rounded-lg border border-line bg-sunken py-1.5 pr-1 pl-2.5">
      {icon}
      <span className="min-w-0 flex-1 text-xs">{children}</span>
      <Button variant="ghost" size="sm" icon={X} label={remove} onClick={onRemove} />
    </li>
  );
}

const ICON = "mt-0.5 shrink-0 text-muted";

/** An app on the desktop, or the whole desktop, with its options. */
export function DesktopGrantRow({ bot, grant }: { bot: Bot; grant: DesktopGrant }) {
  const words = useT().desktop.grants;
  const api = useApi();
  const app = grant.scope === "desktop" ? words.whole : (grant.appName ?? "");
  return (
    <Row
      icon={<Monitor aria-hidden size={14} className={ICON} />}
      remove={words.remove(app)}
      onRemove={() =>
        attempt(words.removeFailed, () => api.call("desktop.revoke", { grantId: grant.id }))
      }
    >
      <span className="block">{app}</span>
      <span className="block text-ink-soft">{grant.level === "act" ? words.act : words.see}</span>
      {grant.level === "act" && <RealInput bot={bot} grant={grant} app={app} />}
      <Unattended bot={bot} grant={grant} app={app} />
      {grant.appPath && (
        <span className="block break-all font-mono text-muted" data-selectable>
          {grant.appPath}
        </span>
      )}
    </Row>
  );
}

/** A site the agent uses in its browser without asking (spec 21.5). */
export function SiteRow({ site }: { site: BrowserSite }) {
  const words = useT().permissions;
  const api = useApi();
  return (
    <Row
      icon={<Globe aria-hidden size={14} className={ICON} />}
      remove={words.siteRemove(site.site)}
      onRemove={() =>
        attempt(words.siteRemoveFailed, () =>
          api.call("sites.revoke", { botId: site.botId, site: site.site }),
        )
      }
    >
      <span className="block break-all font-mono" data-selectable>
        {site.site}
      </span>
      <span className="block text-muted">{words.allowedOn(day(site.allowedAt))}</span>
    </Row>
  );
}

/** An "Always allow" (spec 10.1): the action, then the command, site or file. */
export function RuleRow({ rule }: { rule: AllowRule }) {
  const t = useT();
  const words = t.bots.details;
  const api = useApi();
  const title = toolTitle(rule.scope.toolName, t.tools);
  const what = rule.scope.value ? `${title}: ${rule.scope.value}` : title;
  return (
    <Row
      remove={words.alwaysRemove(what)}
      onRemove={() =>
        attempt(words.alwaysRemoveFailed, () => api.call("rules.delete", { ruleId: rule.id }))
      }
    >
      <span className="block text-ink-soft">{title}</span>
      {rule.scope.value && (
        <span className="block break-all font-mono" data-selectable>
          {rule.scope.value}
        </span>
      )}
    </Row>
  );
}

/** Another crew, or one agent of it, reached for good (spec 10.4). */
export function CrewAccessRow({
  access,
  onRevoked,
}: {
  access: CrewAccess;
  /** The agent's other crews after this one went. */
  onRevoked?(left: CrewAccess[]): void;
}) {
  const words = useT().bots.details;
  const api = useApi();
  const what = access.targetName
    ? words.crewsBot(access.targetName, access.crewName)
    : words.crewsWhole(access.crewName);
  const kinds = [
    access.talk && words.crewsKinds.talk,
    access.edit ? words.crewsKinds.edit : access.read && words.crewsKinds.read,
  ].filter(Boolean);
  return (
    <Row
      icon={<Network aria-hidden size={14} className={ICON} />}
      remove={words.crewsRemove(what)}
      onRemove={() =>
        attempt(words.crewsRemoveFailed, async () => {
          const left = await api.call("crewAccess.revoke", { accessId: access.id });
          onRevoked?.(left);
        })
      }
    >
      <span className="block">{what}</span>
      <span className="block text-ink-soft">{kinds.join(" · ")}</span>
    </Row>
  );
}
