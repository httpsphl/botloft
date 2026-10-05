// "Connect a tool" (spec 25.2): the owner pastes its settings, sees what
// it is and what it can do on their computer, and says they trust it. A
// program (`stdio`) runs with the owner's powers, so it never saves without
// that "I understand".

import { useMemo, useState } from "react";
import { useT } from "../../i18n";
import { errorText } from "../../lib/api";
import type { McpSaveParams } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";
import { Button } from "../../ui/Button";
import { Callout } from "../../ui/Callout";
import { Details } from "../../ui/Details";
import { Dialog } from "../../ui/Dialog";
import { TextArea, TextField } from "../../ui/Field";
import { type PasteProblem, parsePasted } from "./paste";

/** The command line of a program, or the address, as one line. */
function whatItRuns(tool: McpSaveParams): string {
  return tool.kind === "stdio" ? [tool.command, ...tool.args].join(" ") : (tool.url ?? "");
}

export function AddToolDialog({ onClose }: { onClose(): void }) {
  const all = useT();
  const t = all.connections;
  const api = useApi();
  const [text, setText] = useState("");
  const [name, setName] = useState("");
  const [purpose, setPurpose] = useState("");
  const [trusted, setTrusted] = useState(false);
  const [busy, setBusy] = useState(false);
  const [failed, setFailed] = useState<string | null>(null);
  const pasted = useMemo(() => parsePasted(text, name), [text, name]);
  const tools = pasted.ok ? pasted.tools : [];
  const runsPrograms = tools.some((tool) => tool.kind === "stdio");

  const problem = (found: PasteProblem): string => {
    const words = t.problems;
    switch (found.reason) {
      case "unsupported":
        return words.unsupported(found.server);
      case "unknownField":
        return words.unknownField(found.server, found.field);
      case "badValue":
        return words.badValue(found.server, found.field);
      default:
        return words[found.reason];
    }
  };
  // Nothing is said about an empty box until something is in it.
  const complaint = !pasted.ok && pasted.reason !== "empty" ? problem(pasted) : null;
  const asksName = !pasted.ok && pasted.reason === "needsName";

  const connect = async () => {
    setBusy(true);
    setFailed(null);
    try {
      for (const tool of tools) {
        await api.call("mcp.save", {
          ...tool,
          description: tools.length === 1 ? purpose.trim() : "",
        });
      }
      onClose();
    } catch (error) {
      setFailed(errorText(error));
      setBusy(false);
    }
  };

  return (
    <Dialog
      title={t.adding.title}
      onClose={onClose}
      footer={
        <>
          <Button onClick={onClose}>{all.common.cancel}</Button>
          <Button
            variant="primary"
            disabled={tools.length === 0 || (runsPrograms && !trusted) || busy}
            onClick={connect}
          >
            {busy ? t.adding.connecting : t.adding.connect}
          </Button>
        </>
      }
    >
      <div className="flex flex-col gap-3">
        <TextArea
          label={t.adding.paste}
          hint={t.adding.pasteHint}
          rows={6}
          className="font-mono"
          value={text}
          onChange={(event) => {
            setText(event.target.value);
            setTrusted(false);
          }}
        />
        {(asksName || name) && (
          <TextField
            label={t.adding.name}
            hint={t.adding.nameHint}
            value={name}
            onChange={(event) => setName(event.target.value)}
          />
        )}
        {complaint && (
          <p role="alert" className="text-danger text-sm">
            {complaint}
          </p>
        )}
        {tools.length > 0 && (
          <div className="flex flex-col gap-2">
            <p className="text-ink-soft text-sm">{t.adding.found(tools.length)}</p>
            {tools.map((tool) => (
              <div key={tool.name} className="rounded-lg border border-line bg-sunken px-3 py-2">
                <p className="font-medium text-sm">{tool.name}</p>
                <p className="mt-0.5 text-ink-soft text-xs leading-relaxed">
                  {tool.kind === "stdio"
                    ? t.adding.program(tool.name)
                    : t.adding.address(tool.name)}
                </p>
                <Details label={t.adding.command}>{whatItRuns(tool)}</Details>
              </div>
            ))}
            {tools.length === 1 && (
              <TextField
                label={t.adding.purpose}
                hint={t.adding.purposeHint}
                value={purpose}
                onChange={(event) => setPurpose(event.target.value)}
              />
            )}
            {runsPrograms && (
              <label className="flex items-center gap-2 text-sm">
                <input
                  type="checkbox"
                  checked={trusted}
                  onChange={(event) => setTrusted(event.target.checked)}
                />
                {t.adding.understood}
              </label>
            )}
          </div>
        )}
        {failed && (
          <Callout tone="danger" title={t.adding.failed}>
            {failed}
          </Callout>
        )}
      </div>
    </Dialog>
  );
}
