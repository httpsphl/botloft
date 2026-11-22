// Settings, "Where your agents run" (spec 42.6): this computer or a server,
// the way to switch, and the devices with a key of their own.

import { useState } from "react";
import { useT } from "../../i18n";
import { server, useServer } from "../../shell/server";
import { useApp } from "../../store/context";
import { Button } from "../../ui/Button";
import { Confirm } from "../../ui/Confirm";
import { Dialog } from "../../ui/Dialog";
import { Section } from "../account/settingsParts";
import { DeviceList } from "./DeviceList";
import { ServerKeyForm } from "./ServerKeyForm";

export function WhereSettings() {
  const t = useT().server;
  const w = t.where;
  const chosen = useServer();
  const onServer = useApp((state) => state.system?.server ?? false);
  const [adding, setAdding] = useState(false);
  const [leaving, setLeaving] = useState(false);

  return (
    <Section title={w.title}>
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div className="min-w-0">
          <p className="font-medium text-sm">{chosen ? w.onServer(chosen.name) : w.thisComputer}</p>
          <p className="text-muted text-xs leading-relaxed">
            {chosen ? w.onServerHint : w.thisComputerHint}
          </p>
        </div>
        {chosen ? (
          <Button onClick={() => setLeaving(true)}>{w.toThisComputer}</Button>
        ) : (
          <Button onClick={() => setAdding(true)}>{w.toServer}</Button>
        )}
      </div>
      <DeviceList always={chosen !== null || onServer} />
      {adding && (
        <Dialog title={t.form.title} onClose={() => setAdding(false)}>
          <ServerKeyForm onCancel={() => setAdding(false)} />
        </Dialog>
      )}
      {leaving && (
        <Confirm
          title={w.toThisComputerTitle}
          confirmLabel={w.toThisComputer}
          onConfirm={async () => server.clear()}
          onClose={() => setLeaving(false)}
        >
          {w.toThisComputerText}
        </Confirm>
      )}
    </Section>
  );
}
