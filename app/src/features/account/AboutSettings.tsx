// Settings, "About": the versions, for when something needs reporting,
// and a way to look for a new Botloft now instead of waiting for the
// title bar (spec 15.5).

import { RefreshCw } from "lucide-react";
import { useState } from "react";
import { useT } from "../../i18n";
import type { AppUpdate } from "../../lib/host";
import { useApp, useHost } from "../../store/context";
import { Button } from "../../ui/Button";
import { UpdateDialog } from "../updates/UpdateButton";
import { Section } from "./settingsParts";

type Check =
  | { stage: "idle" }
  | { stage: "checking" }
  | { stage: "latest" }
  | { stage: "found"; update: AppUpdate }
  | { stage: "failed" };

export function AboutSettings() {
  const s = useT().account.settings;
  const host = useHost();
  const system = useApp((state) => state.system);
  const [check, setCheck] = useState<Check>({ stage: "idle" });
  const [open, setOpen] = useState(false);

  const look = () => {
    setCheck({ stage: "checking" });
    host.checkForUpdate().then(
      (update) => setCheck(update ? { stage: "found", update } : { stage: "latest" }),
      () => setCheck({ stage: "failed" }),
    );
  };

  const said = {
    idle: null,
    checking: s.checking,
    latest: s.upToDate,
    found: check.stage === "found" ? s.updateFound(check.update.version) : null,
    failed: s.checkFailed,
  }[check.stage];

  return (
    <Section title={s.about}>
      <ul className="text-ink-soft text-sm leading-relaxed" data-selectable>
        {system && <li>{s.botloft(system.daemonVersion)}</li>}
        {system?.claudeVersion && <li>{s.claudeCode(system.claudeVersion)}</li>}
        {system?.account.claude?.email && <li>{system.account.claude.email}</li>}
      </ul>
      <div className="flex flex-wrap items-center gap-3">
        {check.stage === "found" ? (
          <Button variant="primary" onClick={() => setOpen(true)}>
            {s.seeUpdate}
          </Button>
        ) : (
          <Button icon={RefreshCw} onClick={look} disabled={check.stage === "checking"}>
            {s.checkUpdates}
          </Button>
        )}
        <p role="status" className="text-muted text-sm">
          {said}
        </p>
      </div>
      {open && check.stage === "found" && (
        <UpdateDialog update={check.update} onClose={() => setOpen(false)} />
      )}
    </Section>
  );
}
