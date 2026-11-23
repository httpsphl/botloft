// "Connect to a server" (spec 42.6): the key `botloftd pair` printed there,
// what it says about the server, and how this computer gets there: by
// itself at an address the owner may change, or through a tunnel the owner
// opens.

import { useState } from "react";
import { useT } from "../../i18n";
import { isHost, parseKey, server } from "../../shell/server";
import { Button } from "../../ui/Button";
import { SecretField, TextField } from "../../ui/Field";
import { TunnelCommand } from "./TunnelCommand";

export function ServerKeyForm({ onCancel }: { onCancel?: () => void }) {
  const t = useT();
  const f = t.server.form;
  const own = t.server.own;
  const [key, setKey] = useState("");
  /** The address the owner typed; null keeps the key's own. */
  const [address, setAddress] = useState<string | null>(null);
  const found = parseKey(key);
  const invalid = key.trim() !== "" && !found;
  const ssh = found?.ssh;
  const host = (address ?? ssh?.host ?? "").trim();
  const ready = found !== null && (!ssh || isHost(host));

  return (
    <form
      className="flex flex-col gap-4"
      onSubmit={(event) => {
        event.preventDefault();
        if (found && ready) {
          server.set(ssh ? { ...found, ssh: { ...ssh, host } } : found);
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
        onChange={(event) => {
          setKey(event.target.value);
          setAddress(null);
        }}
      />
      {found && (
        <div className="flex flex-col gap-2">
          <p className="font-medium text-sm">{f.found(found.name)}</p>
          {ssh ? (
            <>
              <p className="text-ink-soft text-sm leading-relaxed">{own.found(ssh.host)}</p>
              <TextField
                label={own.address}
                hint={own.addressHint}
                value={address ?? ssh.host}
                onChange={(event) => setAddress(event.target.value)}
              />
            </>
          ) : (
            <>
              <p className="text-ink-soft text-sm leading-relaxed">{f.tunnel}</p>
              <TunnelCommand server={found} />
            </>
          )}
          {/* Moving from this computer goes by a copy (spec 42.10). */}
          <p className="text-muted text-xs leading-relaxed">{f.move}</p>
        </div>
      )}
      <div className="flex gap-2">
        <Button type="submit" variant="primary" disabled={!ready}>
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
