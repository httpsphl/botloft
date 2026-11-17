// Settings > Instructions for all (spec 7.10): what the owner writes for
// every agent to follow. Saved with a button, not as the owner types: each
// save starts the available agents again so they read it.

import { useEffect, useState } from "react";
import { useT } from "../../i18n";
import { FIELD_LIMITS } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";
import { Button } from "../../ui/Button";
import { TextArea } from "../../ui/Field";
import { attempt, notify, notifyError } from "../../ui/toast";

export function InstructionsSettings() {
  const words = useT().instructions;
  const api = useApi();
  // What the agents have now; `null` until it is read.
  const [saved, setSaved] = useState<string | null>(null);
  const [text, setText] = useState("");
  const [saving, setSaving] = useState(false);
  const loadFailed = words.loadFailed;

  useEffect(() => {
    let live = true;
    api.call("instructions.get").then(
      (current) => {
        if (live) {
          setSaved(current.text);
          setText(current.text);
        }
      },
      (failure) => live && notifyError(loadFailed, failure),
    );
    return () => {
      live = false;
    };
  }, [api, loadFailed]);

  const changed = saved !== null && text.trim() !== saved;
  const tooLong = text.length > FIELD_LIMITS.instructions;

  const save = async () => {
    setSaving(true);
    const done = await attempt(words.saveFailed, async () => {
      const result = await api.call("instructions.set", { text });
      setSaved(result.text);
      setText(result.text);
    });
    setSaving(false);
    if (done) {
      notify(words.saved);
    }
  };

  return (
    <div className="flex flex-col gap-4 rounded-2xl bg-bubble p-4">
      <p className="text-ink-soft text-sm">{words.intro}</p>
      <TextArea
        label={words.label}
        value={text}
        rows={10}
        max={FIELD_LIMITS.instructions}
        placeholder={words.placeholder}
        disabled={saved === null}
        onChange={(event) => setText(event.target.value)}
      />
      <div>
        <Button variant="primary" disabled={!changed || tooLong || saving} onClick={save}>
          {words.save}
        </Button>
      </div>
      <details className="text-xs">
        <summary className="cursor-pointer text-muted hover:text-ink-soft">
          {words.howItWorks}
        </summary>
        <p className="mt-1 text-ink-soft leading-relaxed">{words.explain}</p>
      </details>
    </div>
  );
}
