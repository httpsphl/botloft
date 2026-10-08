// Notices on this phone (spec 28.8): an invitation on the inbox while they
// are off, and the whole control on the page about this phone.

import { useEffect, useState } from "react";
import { useT } from "../i18n";
import type { PhoneApi } from "./client";
import type { NoticeState } from "./notices";
import { PhoneButton } from "./parts";

export function Notices({ api, invite = false }: { api: PhoneApi; invite?: boolean }) {
  const n = useT().phone.settings.notices;
  const [state, setState] = useState<NoticeState | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    let live = true;
    void api.noticeState().then((now) => live && setState(now));
    return () => {
      live = false;
    };
  }, [api]);

  const change = async (work: () => Promise<NoticeState>) => {
    setBusy(true);
    setState(await work());
    setBusy(false);
  };

  // The Home Screen has its own words, and nothing else is for the inbox.
  if (state === null || state === "home-screen" || (invite && state !== "off")) {
    return null;
  }
  return (
    <div className="flex flex-col gap-2">
      {!invite && state === "on" && <p className="text-ink-soft text-sm">{n.on}</p>}
      {!invite && state === "off" && <p className="text-ink-soft text-sm">{n.off}</p>}
      {state === "blocked" && <p className="text-ink-soft text-sm">{n.blocked}</p>}
      {state === "unsupported" && <p className="text-muted text-sm">{n.unsupported}</p>}
      {state === "off" && (
        <PhoneButton
          look={invite ? "quiet" : "primary"}
          disabled={busy}
          onClick={() => change(api.turnOnNotices)}
          className="flex-none"
        >
          {n.turnOn}
        </PhoneButton>
      )}
      {!invite && state === "on" && (
        <PhoneButton
          disabled={busy}
          onClick={() => change(api.turnOffNotices)}
          className="flex-none"
        >
          {n.turnOff}
        </PhoneButton>
      )}
    </div>
  );
}
