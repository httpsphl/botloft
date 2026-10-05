// Edit a connected tool's name and what it is for (spec 25.7). What it
// runs and its passwords cannot be seen again, so changing them means
// connecting the tool anew.

import { useState } from "react";
import { useT } from "../../i18n";
import { errorText } from "../../lib/api";
import type { McpServer } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";
import { Button } from "../../ui/Button";
import { Callout } from "../../ui/Callout";
import { Dialog } from "../../ui/Dialog";
import { TextField } from "../../ui/Field";

export function EditToolDialog({ server, onClose }: { server: McpServer; onClose(): void }) {
  const t = useT();
  const words = t.connections;
  const api = useApi();
  const [name, setName] = useState(server.name);
  const [purpose, setPurpose] = useState(server.description);
  const [busy, setBusy] = useState(false);
  const [failed, setFailed] = useState<string | null>(null);

  const save = async () => {
    setBusy(true);
    setFailed(null);
    try {
      // An empty value keeps the one the daemon holds under that name.
      await api.call("mcp.save", {
        serverId: server.id,
        name: name.trim(),
        kind: server.kind,
        url: server.url,
        command: server.command,
        args: server.args,
        headers: Object.fromEntries(server.headerNames.map((each) => [each, ""])),
        env: Object.fromEntries(server.envNames.map((each) => [each, ""])),
        description: purpose.trim(),
      });
      onClose();
    } catch (error) {
      setFailed(errorText(error));
      setBusy(false);
    }
  };

  return (
    <Dialog
      title={words.editing.title(server.name)}
      onClose={onClose}
      footer={
        <>
          <Button onClick={onClose}>{t.common.cancel}</Button>
          <Button variant="primary" disabled={!name.trim() || busy} onClick={save}>
            {words.editing.save}
          </Button>
        </>
      }
    >
      <div className="flex flex-col gap-3">
        <TextField
          label={words.adding.name}
          value={name}
          onChange={(event) => setName(event.target.value)}
        />
        <TextField
          label={words.adding.purpose}
          hint={words.adding.purposeHint}
          value={purpose}
          onChange={(event) => setPurpose(event.target.value)}
        />
        <p className="text-muted text-xs">{words.editing.note}</p>
        {failed && (
          <Callout tone="danger" title={words.editing.failed}>
            {failed}
          </Callout>
        )}
      </div>
    </Dialog>
  );
}
