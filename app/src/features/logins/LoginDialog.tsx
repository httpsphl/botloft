// One site login, opened from Settings > Logins (spec 35.4): who may use
// it, an agent per switch crew by crew, signing in again, and removing it.

import { LogIn, Trash2 } from "lucide-react";
import { useState } from "react";
import { useShallow } from "zustand/react/shallow";
import { useT } from "../../i18n";
import { fromNow } from "../../lib/format";
import type { Bot, SiteLogin } from "../../lib/protocol.gen";
import { useApi, useApp } from "../../store/context";
import { Button } from "../../ui/Button";
import { Confirm } from "../../ui/Confirm";
import { Dialog } from "../../ui/Dialog";
import { attempt } from "../../ui/toast";
import { AgentSwitches } from "../connections/AgentSwitches";
import { SiteTile } from "./SiteTile";

export function LoginDialog({ login, onClose }: { login: SiteLogin; onClose(): void }) {
  const t = useT().logins;
  const api = useApi();
  const bots = useApp(
    useShallow((state) => Object.values(state.bots).filter((bot) => bot.archivedAt === null)),
  );
  const [removing, setRemoving] = useState(false);
  const site = login.site;
  const set = (bot: Bot, on: boolean) => {
    const botIds = on ? [...login.botIds, bot.id] : login.botIds.filter((id) => id !== bot.id);
    return attempt(t.failed, () => api.call("logins.share", { site, botIds }));
  };

  return (
    <Dialog
      title={site}
      onClose={onClose}
      footer={
        <>
          <Button
            icon={LogIn}
            onClick={() =>
              attempt(t.openFailed, async () => {
                await api.call("logins.open", { address: site });
                onClose();
              })
            }
          >
            {t.again}
          </Button>
          <Button variant="danger" icon={Trash2} onClick={() => setRemoving(true)}>
            {t.remove}
          </Button>
        </>
      }
    >
      <div className="flex flex-col gap-5">
        <div className="flex items-start gap-3">
          <SiteTile site={site} size={48} />
          <div className="flex min-w-0 flex-col gap-1">
            <p className="text-muted text-xs">{t.signedAt(fromNow(login.signedAt))}</p>
            <p className="text-ink-soft text-sm leading-relaxed">{t.warning}</p>
          </div>
        </div>
        <section className="flex flex-col gap-2">
          <h3 className="font-semibold text-sm">{t.who}</h3>
          <AgentSwitches
            bots={bots}
            uses={(bot) => login.botIds.includes(bot.id)}
            onChange={set}
            label={(bot) => t.canUse(bot.name)}
            empty={t.noAgents}
          />
        </section>
      </div>
      {removing && (
        <Confirm
          title={t.removing.title(site)}
          confirmLabel={t.removing.confirm}
          onClose={() => setRemoving(false)}
          onConfirm={() =>
            attempt(t.removeFailed, async () => {
              await api.call("logins.remove", { site });
              onClose();
            })
          }
        >
          {t.removing.text}
        </Confirm>
      )}
    </Dialog>
  );
}
