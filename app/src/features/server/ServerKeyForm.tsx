// "Connect to a server" (spec 42.6): the key `botloftd pair` printed there,
// what it says about the server, and the tunnel to open from here.

import { useState } from "react";
import { useT } from "../../i18n";
import { parseKey, server } from "../../shell/server";
import { Button } from "../../ui/Button";
import { SecretField } from "../../ui/Field";
import { TunnelCommand } from "./TunnelCommand";

export function ServerKeyForm({ onCancel }: { onCancel?: () => void }) {
  const t = useT();
  const f = t.server.form;
  const [key, setKey] = useState("");
  const found = parseKey(key);
  const invalid = key.trim() !== "" && !found;

  return (
    <form
      className="flex flex-col gap-4"
      onSubmit={(event) => {
        event.preventDefault();
        if (found) {
          server.set(found);
        }
      }}
    >
      <p className="text-ink-soft text-sm leading-relaxed">{f.intro}</p>
      <SecretField
        label={f.key}
        hint={invalid ? f.invalid : undefined}
        show={f.show}
        hide={f.hide}
        placeholder={f.placeholder}
        value={key}
        onChange={(event) => setKey(event.target.value)}
      />
      {found && (
        <div className="flex flex-col gap-2">
          <p className="font-medium text-sm">{f.found(found.name)}</p>
          <p className="text-ink-soft text-sm leading-relaxed">{f.tunnel}</p>
          <TunnelCommand server={found} />
        </div>
      )}
      <div className="flex gap-2">
        <Button type="submit" variant="primary" disabled={!found}>
          {f.connect}
        </Button>
        {onCancel && (
          <Button type="button" onClick={onCancel}>
            {t.common.cancel}
          </Button>
        )}
      </div>
    </form>
  );
}
