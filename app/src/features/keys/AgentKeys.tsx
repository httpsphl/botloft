// The Keys part of an agent's details (spec 32.5): each key by name, with
// what it is for, when it changed, and whether its value is on this
// computer; replace, delete, and add one. A value never comes back here.

import { KeyRound, Plus, RefreshCw, Trash2 } from "lucide-react";
import { useEffect, useState } from "react";
import { useT } from "../../i18n";
import { when } from "../../lib/format";
import type { AgentKey, Bot } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";
import { Badge } from "../../ui/Badge";
import { Button } from "../../ui/Button";
import { Confirm } from "../../ui/Confirm";
import { notifyError } from "../../ui/toast";
import { KeyForm } from "./KeyForm";

/** The agent's keys, read when shown and again when it gets one it asked for. */
function useKeys(botId: Bot["id"]) {
  const api = useApi();
  const [keys, setKeys] = useState<AgentKey[] | null>(null);
  useEffect(() => {
    let alive = true;
    const read = () =>
      api.call("keys.list", { botId }).then(
        (listed) => alive && setKeys(listed),
        // An older daemon has no keys: the part stays empty.
        () => alive && setKeys([]),
      );
    setKeys(null);
    read();
    const stop = api.subscribe((event) => {
      if (
        event.name === "chat.item" &&
        event.params.item.botId === botId &&
        event.params.item.body.kind === "key_request" &&
        event.params.item.body.status === "saved"
      ) {
        read();
      }
    });
    return () => {
      alive = false;
      stop();
    };
  }, [api, botId]);
  return [keys, setKeys] as const;
}

/** Which form is open: a new key, or replacing one by name. */
type Editing = { kind: "add" } | { kind: "replace"; name: string } | null;

export function AgentKeys({ bot }: { bot: Bot }) {
  const t = useT().keys;
  const words = t.details;
  const api = useApi();
  const [keys, setKeys] = useKeys(bot.id);
  const [editing, setEditing] = useState<Editing>(null);
  const [removing, setRemoving] = useState<string | null>(null);

  const saved = (listed: AgentKey[]) => {
    setKeys(listed);
    setEditing(null);
  };
  const cancel = (
    <Button type="button" variant="ghost" onClick={() => setEditing(null)}>
      {t.form.cancel}
    </Button>
  );

  return (
    <div>
      <dt className="text-muted text-xs">{words.title}</dt>
      <dd className="mt-1 flex flex-col gap-1.5">
        {keys !== null && keys.length === 0 && editing === null && (
          <p className="text-ink-soft text-xs leading-relaxed">{words.none(bot.name)}</p>
        )}
        {keys !== null && keys.length > 0 && (
          <ul aria-label={words.title} className="flex flex-col gap-1.5">
            {keys.map((key) =>
              editing?.kind === "replace" && editing.name === key.name ? (
                <li key={key.name} className="rounded-lg border border-line bg-sunken p-2.5">
                  <KeyForm bot={bot} name={key.name} onSaved={saved} actions={cancel} />
                </li>
              ) : (
                <li
                  key={key.name}
                  className="flex items-center gap-2 rounded-lg border border-line bg-sunken py-1.5 pr-1 pl-2.5"
                >
                  <KeyRound aria-hidden size={14} className="shrink-0 text-muted" />
                  <span className="min-w-0 flex-1 text-xs">
                    <span className="flex items-center gap-1.5">
                      <span className="truncate font-mono font-medium">{key.name}</span>
                      {key.missing && (
                        <Badge tone="warn" title={words.missingHint}>
                          {words.missing}
                        </Badge>
                      )}
                    </span>
                    <span className="block truncate text-ink-soft">
                      {[key.description, words.changed(when(key.setAt))]
                        .filter(Boolean)
                        .join(" · ")}
                    </span>
                  </span>
                  <Button
                    variant="ghost"
                    size="sm"
                    icon={RefreshCw}
                    label={words.replace(key.name)}
                    onClick={() => setEditing({ kind: "replace", name: key.name })}
                  />
                  <Button
                    variant="ghost"
                    size="sm"
                    icon={Trash2}
                    label={words.remove(key.name)}
                    onClick={() => setRemoving(key.name)}
                  />
                </li>
              ),
            )}
          </ul>
        )}
        {editing?.kind === "add" ? (
          <div className="rounded-lg border border-line bg-sunken p-2.5">
            <KeyForm bot={bot} onSaved={saved} actions={cancel} />
          </div>
        ) : (
          keys !== null && (
            <div>
              <Button size="sm" icon={Plus} onClick={() => setEditing({ kind: "add" })}>
                {words.add}
              </Button>
            </div>
          )
        )}
        <details className="text-xs">
          <summary className="cursor-pointer text-muted hover:text-ink-soft">
            {words.howItWorks}
          </summary>
          <p className="mt-1 text-ink-soft leading-relaxed">{words.explain(bot.name)}</p>
        </details>
      </dd>
      {removing !== null && (
        <Confirm
          title={words.removeTitle(removing)}
          confirmLabel={words.removeConfirm}
          onClose={() => setRemoving(null)}
          onConfirm={async () => {
            try {
              setKeys(await api.call("keys.delete", { botId: bot.id, name: removing }));
            } catch (failure) {
              notifyError(words.removeFailed, failure);
            }
          }}
        >
          {words.removeBody(bot.name)}
        </Confirm>
      )}
    </div>
  );
}
