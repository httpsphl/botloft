// Settings, "General": where the experimental agents' programs are. Botloft
// looks in the usual places; when someone installed one somewhere else (mise,
// npm, a folder of their own), they say where here (spec 30).

import { useState } from "react";
import { useT } from "../../i18n";
import type { AgentCheck } from "../../lib/protocol.gen";
import { useApi, useApp } from "../../store/context";
import { Button } from "../../ui/Button";
import { TextField } from "../../ui/Field";
import { attempt } from "../../ui/toast";

const NO_CHECKS: AgentCheck[] = [];

export function AgentPrograms() {
  const checks = useApp((state) => state.system?.agentChecks ?? NO_CHECKS);
  if (checks.length === 0) {
    return null;
  }
  return (
    <div className="flex flex-col gap-4">
      {checks.map((check) => (
        <Program key={check.agent} check={check} />
      ))}
    </div>
  );
}

function Program({ check }: { check: AgentCheck }) {
  const t = useT();
  const s = t.account.settings;
  const api = useApi();
  const system = useApp((state) => state.system);
  const putSystem = useApp((state) => state.putSystem);
  const name = t.bots.dialog.agent.names[check.agent];
  const [text, setText] = useState("");
  const [busy, setBusy] = useState(false);

  const found = check.path !== null;
  const status = found
    ? check.version === null
      ? s.programFoundNoVersion(check.path ?? "")
      : s.programFound(check.version, check.path ?? "")
    : s.programMissing(name);

  const save = async (path: string) => {
    setBusy(true);
    const saved = await attempt(s.programFailed, async () => {
      const next = await api.call("agents.setPath", { agent: check.agent, path });
      if (system) {
        putSystem({
          ...system,
          agentChecks: system.agentChecks.map((have) => (have.agent === next.agent ? next : have)),
        });
      }
    });
    setBusy(false);
    if (saved) {
      setText("");
    }
  };

  return (
    <div className="flex flex-col gap-2">
      <p className="font-medium text-sm">{name}</p>
      <p
        className={`text-xs leading-relaxed ${found ? "text-muted" : "text-warn"}`}
        data-selectable
      >
        {status}
      </p>
      <div className="flex items-end gap-2">
        <div className="min-w-0 flex-1">
          <TextField
            label={s.programPath(name)}
            value={text}
            placeholder={s.programPlaceholder}
            onChange={(event) => setText(event.target.value)}
          />
        </div>
        <Button
          variant="secondary"
          disabled={busy || text.trim() === ""}
          onClick={() => save(text)}
        >
          {s.programSave}
        </Button>
      </div>
      <p className="text-muted text-xs leading-relaxed">{s.programHint}</p>
    </div>
  );
}
