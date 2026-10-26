// Keys pasted in the conversations before they were kept apart (spec 32.7),
// in Settings: look for them, then keep each as an agent's key or delete it
// from Botloft. Only a key's start and end ever reach the app.

import { KeyRound, Save, Search, Trash2, X } from "lucide-react";
import { type FormEvent, useState } from "react";
import { useT } from "../../i18n";
import { when } from "../../lib/format";
import type { BotId, KeyHit, KeyScrub } from "../../lib/protocol.gen";
import { useApi, useApp } from "../../store/context";
import { Button } from "../../ui/Button";
import { Callout } from "../../ui/Callout";
import { Confirm } from "../../ui/Confirm";
import { SelectField, TextField } from "../../ui/Field";
import { attempt, notifyError } from "../../ui/toast";
import { Section } from "../account/settingsParts";
import { asKeyName } from "./KeyForm";

export function PastedKeys() {
  const t = useT().keys.scan;
  const api = useApi();
  const [hits, setHits] = useState<KeyHit[] | null>(null);
  const [busy, setBusy] = useState(false);

  const look = async () => {
    setBusy(true);
    await attempt(t.failed, async () => setHits(await api.call("keys.scan")));
    setBusy(false);
  };
  const scrub = async (one: KeyScrub) => {
    try {
      setHits(await api.call("keys.scrub", { hits: [one] }));
      return true;
    } catch (failure) {
      notifyError(t.scrubFailed, failure);
      return false;
    }
  };

  return (
    <Section title={t.title}>
      <p className="text-ink-soft text-sm leading-relaxed">{t.intro}</p>
      <div>
        <Button icon={Search} disabled={busy} onClick={look}>
          {hits === null ? t.run : t.again}
        </Button>
      </div>
      {hits !== null && hits.length === 0 && <p className="text-muted text-sm">{t.none}</p>}
      {hits !== null && hits.length > 0 && (
        <>
          <p className="font-medium text-sm">{t.found(hits.length)}</p>
          <ul aria-label={t.found(hits.length)} className="flex flex-col gap-2">
            {hits.map((hit) => (
              <Hit key={hit.id} hit={hit} onScrub={scrub} />
            ))}
          </ul>
          <Callout tone="warn" title={t.reachTitle}>
            {t.reach}
          </Callout>
        </>
      )}
    </Section>
  );
}

function Hit({ hit, onScrub }: { hit: KeyHit; onScrub(one: KeyScrub): Promise<boolean> }) {
  const t = useT().keys.scan;
  const bots = useApp((state) => state.bots);
  const [keeping, setKeeping] = useState(false);
  const [removing, setRemoving] = useState(false);
  const agents = hit.botIds.map((id) => bots[id]?.name ?? t.goneAgent);
  return (
    <li className="rounded-lg border border-line bg-sunken px-3 py-2.5">
      <p className="flex items-center gap-2">
        <KeyRound aria-hidden size={14} className="shrink-0 text-muted" />
        <span className="font-mono font-medium text-sm">{hit.preview}</span>
      </p>
      <p className="mt-0.5 text-ink-soft text-xs">
        {[agents.join(", "), t.places(hit.places), when(hit.lastAt)].join(" · ")}
      </p>
      {!keeping && (
        <div className="mt-2.5 flex flex-wrap gap-2">
          <Button size="sm" icon={Save} onClick={() => setKeeping(true)}>
            {t.keep}
          </Button>
          <Button size="sm" variant="ghost" icon={Trash2} onClick={() => setRemoving(true)}>
            {t.remove}
          </Button>
        </div>
      )}
      {keeping && (
        <KeepForm
          hit={hit}
          onCancel={() => setKeeping(false)}
          onKeep={(botId, name) => onScrub({ id: hit.id, keep: { botId, name } })}
        />
      )}
      {removing && (
        <Confirm
          title={t.removeTitle(hit.preview)}
          confirmLabel={t.remove}
          onClose={() => setRemoving(false)}
          onConfirm={() => onScrub({ id: hit.id, keep: null })}
        >
          {t.removeBody}
        </Confirm>
      )}
    </li>
  );
}

function KeepForm({
  hit,
  onCancel,
  onKeep,
}: {
  hit: KeyHit;
  onCancel(): void;
  onKeep(botId: BotId, name: string): Promise<boolean>;
}) {
  const words = useT().keys;
  const bots = useApp((state) => state.bots);
  // Only agents still here can keep a key.
  const choices = hit.botIds.filter((id) => bots[id]);
  const all = Object.values(bots).filter((bot) => !choices.includes(bot.id));
  const options = [...choices.map((id) => bots[id]), ...all].flatMap((bot) =>
    bot ? [{ value: bot.id, label: bot.name }] : [],
  );
  const [botId, setBotId] = useState(options[0]?.value ?? "");
  const [name, setName] = useState("");
  const [busy, setBusy] = useState(false);

  const keep = async (event: FormEvent) => {
    event.preventDefault();
    setBusy(true);
    await onKeep(botId as BotId, name);
    setBusy(false);
  };

  return (
    <form onSubmit={keep} className="mt-3 flex flex-col gap-3">
      <p className="font-medium text-sm">{words.scan.keepTitle}</p>
      <div className="grid grid-cols-2 gap-3">
        <SelectField
          label={words.scan.agent}
          options={options}
          value={botId}
          onChange={(event) => setBotId(event.target.value)}
        />
        <TextField
          label={words.form.name}
          hint={words.form.nameHint}
          value={name}
          maxLength={64}
          autoFocus
          onChange={(event) => setName(asKeyName(event.target.value))}
        />
      </div>
      <div className="flex flex-wrap gap-2">
        <Button type="submit" variant="primary" icon={Save} disabled={busy || !botId || !name}>
          {words.form.save}
        </Button>
        <Button type="button" variant="ghost" icon={X} onClick={onCancel}>
          {words.form.cancel}
        </Button>
      </div>
    </form>
  );
}
