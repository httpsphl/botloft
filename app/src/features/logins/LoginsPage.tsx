// Settings, "Logins" (spec 35.4): sign in to a site once, in a window of
// Botloft's own, and choose which agents use the login. A card opens who
// may use it.

import { AppWindow, CircleAlert, CircleCheck } from "lucide-react";
import { useState } from "react";
import { useT } from "../../i18n";
import { fromNow } from "../../lib/format";
import type { SiteLogin } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";
import { Button } from "../../ui/Button";
import { attempt } from "../../ui/toast";
import { LoginDialog } from "./LoginDialog";
import { SiteTile } from "./SiteTile";
import { useLogins } from "./useLogins";

const CARD =
  "flex w-full min-w-0 gap-3 rounded-2xl bg-bubble p-4 text-left transition-colors hover:bg-line/60";

export function LoginsPage() {
  const t = useT().logins;
  const api = useApi();
  const { list, closed } = useLogins();
  const [address, setAddress] = useState("");
  const [opened, setOpened] = useState<string | null>(null);
  const open = (event: React.FormEvent) => {
    event.preventDefault();
    const typed = address.trim();
    if (typed === "") {
      return;
    }
    void attempt(t.openFailed, async () => {
      await api.call("logins.open", { address: typed });
      setAddress("");
    });
  };
  const login: SiteLogin | undefined = list?.logins.find((each) => each.site === opened);

  return (
    <div className="flex flex-col gap-5">
      <p className="max-w-[65ch] text-ink-soft text-sm leading-relaxed">{t.intro}</p>

      <form onSubmit={open} className="flex flex-wrap items-end gap-2">
        <label className="flex min-w-56 flex-1 flex-col gap-1.5">
          <span className="font-semibold text-sm">{t.signIn}</span>
          <input
            value={address}
            onChange={(event) => setAddress(event.target.value)}
            placeholder={t.placeholder}
            aria-label={t.address}
            disabled={list?.window != null}
            className="h-9 w-full rounded-full border border-transparent bg-sunken px-4 text-ink text-sm outline-none transition-colors placeholder:text-muted focus:border-line-strong focus:bg-panel"
          />
        </label>
        <Button
          type="submit"
          variant="primary"
          icon={AppWindow}
          disabled={address.trim() === "" || list?.window != null}
        >
          {t.open}
        </Button>
      </form>

      {list?.window && (
        <p role="status" className="flex items-start gap-2 rounded-xl bg-bubble p-3 text-sm">
          <AppWindow aria-hidden size={16} className="mt-0.5 shrink-0 text-work" />
          {t.inWindow(list.window)}
        </p>
      )}
      {!list?.window && closed && (
        <p role="status" className="flex items-start gap-2 rounded-xl bg-bubble p-3 text-sm">
          {closed.found ? (
            <CircleCheck aria-hidden size={16} className="mt-0.5 shrink-0 text-ok" />
          ) : (
            <CircleAlert aria-hidden size={16} className="mt-0.5 shrink-0 text-warn" />
          )}
          {closed.found ? t.signedIn(closed.site) : t.nothing(closed.site)}
        </p>
      )}

      {list !== null &&
        (list.logins.length === 0 ? (
          <p className="text-muted text-sm">{t.none}</p>
        ) : (
          <ul
            aria-label={t.list}
            className="grid grid-cols-[repeat(auto-fill,minmax(17rem,1fr))] gap-3"
          >
            {list.logins.map((each) => (
              <li key={each.site} className="flex">
                <button type="button" className={CARD} onClick={() => setOpened(each.site)}>
                  <SiteTile site={each.site} />
                  <span className="flex min-w-0 flex-1 flex-col gap-0.5">
                    <span className="break-words font-semibold">{each.site}</span>
                    <span className="text-muted text-xs">{t.signedAt(fromNow(each.signedAt))}</span>
                    <span className="mt-1 text-ink-soft text-xs">
                      {each.botIds.length > 0 ? t.agents(each.botIds.length) : t.noAgent}
                    </span>
                  </span>
                </button>
              </li>
            ))}
          </ul>
        ))}

      {login && <LoginDialog login={login} onClose={() => setOpened(null)} />}
    </div>
  );
}
