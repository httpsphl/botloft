// The form that saves one of an agent's keys (spec 32.5): in the card where
// the agent asks for it, with the name fixed, and in the Keys part of its
// details, to add or replace one. The value goes to the daemon and is
// forgotten here as soon as it is saved.

import { Save } from "lucide-react";
import { type FormEvent, type ReactNode, useState } from "react";
import { useT } from "../../i18n";
import { errorText } from "../../lib/api";
import type { AgentKey, Bot, ChatItemId } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";
import { Button } from "../../ui/Button";
import { SecretField, TextField } from "../../ui/Field";

/** The shape the daemon takes (spec 32.3), to enable Save before asking it. */
const NAME = /^[A-Z][A-Z0-9_]{1,63}$/;

/** Capitals and _ as the owner types: "github token" becomes "GITHUB_TOKEN". */
export function asKeyName(typed: string): string {
  return typed.toUpperCase().replace(/[\s-]+/g, "_");
}

export function KeyForm({
  bot,
  name: fixed,
  requestId,
  onSaved,
  actions,
}: {
  bot: Bot;
  /** The name when it is already chosen: asked for, or replaced. */
  name?: string;
  /** The agent's request this answers. */
  requestId?: ChatItemId;
  onSaved(keys: AgentKey[]): void;
  /** Buttons beside Save: Cancel, or Not now. */
  actions?: ReactNode;
}) {
  const t = useT().keys.form;
  const api = useApi();
  const [name, setName] = useState(fixed ?? "");
  const [value, setValue] = useState("");
  const [description, setDescription] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const ready = NAME.test(name) && value.trim().length > 0 && !busy;

  const save = async (event: FormEvent) => {
    event.preventDefault();
    if (!ready) {
      return;
    }
    setBusy(true);
    setError(null);
    try {
      const keys = await api.call("keys.set", {
        botId: bot.id,
        name,
        value,
        ...(description.trim() ? { description: description.trim() } : {}),
        ...(requestId ? { requestId } : {}),
      });
      setValue("");
      onSaved(keys);
    } catch (failure) {
      setError(`${t.saveFailed}: ${errorText(failure)}`);
    } finally {
      setBusy(false);
    }
  };

  return (
    <form onSubmit={save} className="flex flex-col gap-3">
      {fixed === undefined && (
        <TextField
          label={t.name}
          hint={t.nameHint}
          value={name}
          maxLength={64}
          autoFocus
          onChange={(event) => setName(asKeyName(event.target.value))}
        />
      )}
      <SecretField
        label={fixed === undefined ? t.value : `${t.value} ${fixed}`}
        hint={t.valueHint}
        show={t.show}
        hide={t.hide}
        value={value}
        // A card that arrives in the chat leaves the cursor where it is.
        autoFocus={fixed !== undefined && requestId === undefined}
        onChange={(event) => setValue(event.target.value)}
      />
      <TextField
        label={t.description}
        placeholder={t.descriptionPlaceholder}
        value={description}
        maxLength={200}
        onChange={(event) => setDescription(event.target.value)}
      />
      {error && (
        <p role="alert" className="text-danger text-xs">
          {error}
        </p>
      )}
      <p className="text-muted text-xs">{t.restarts(bot.name)}</p>
      <div className="flex flex-wrap gap-2">
        <Button type="submit" variant="primary" icon={Save} disabled={!ready}>
          {t.save}
        </Button>
        {actions}
      </div>
    </form>
  );
}
