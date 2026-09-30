// The address row of a bot's browser (spec 21.8): reloading the page, where
// it is, and opening it in the owner's own browser. With the browser in
// the owner's hands the address is theirs to type (spec 21.10).

import { ExternalLink, Globe, LoaderCircle, Lock, RotateCw } from "lucide-react";
import { type RefObject, useEffect, useState } from "react";
import { useT } from "../../i18n";
import type { Bot, BrowserState } from "../../lib/protocol.gen";
import { useApi, useHost } from "../../store/context";
import { Button } from "../../ui/Button";
import { attempt } from "../../ui/toast";
import type { Hands } from "./useHands";

export function AddressBar({
  bot,
  state,
  hands,
  field,
}: {
  bot: Pick<Bot, "id">;
  state: BrowserState;
  hands: Hands;
  /** The address field, for whoever sends the owner to it. */
  field: RefObject<HTMLInputElement | null>;
}) {
  const t = useT().browser;
  const api = useApi();
  const host = useHost();
  const url = state.url && state.url !== "about:blank" ? state.url : null;
  const web = url !== null && /^https?:\/\//i.test(url);
  const Icon = state.loading ? LoaderCircle : url?.startsWith("https://") ? Lock : Globe;
  const held = hands.held;
  const tab = state.tabs.find((tab) => tab.active)?.id;
  // What the owner is typing; `null` shows where the page is.
  const [draft, setDraft] = useState<string | null>(null);
  // biome-ignore lint/correctness/useExhaustiveDependencies: another tab, or the bot's turn again, drops what was typed
  useEffect(() => setDraft(null), [tab, held]);

  const go = () => {
    const typed = draft?.trim();
    setDraft(null);
    field.current?.blur();
    if (typed) {
      void hands.open(typed);
    }
  };

  return (
    <div className="flex h-11 shrink-0 items-center gap-1 border-line border-b px-1.5">
      <Button
        variant="ghost"
        size="sm"
        icon={RotateCw}
        label={t.reload}
        disabled={state.status !== "open"}
        onClick={() => attempt(t.reloadFailed, () => api.call("browser.reload", { botId: bot.id }))}
      />
      <div
        className={`flex min-w-0 flex-1 items-center gap-2 rounded-full bg-sunken px-3 py-1.5 ${
          held ? "focus-within:ring-2 focus-within:ring-accent/35" : ""
        }`}
      >
        <Icon
          aria-label={state.loading ? t.loading : undefined}
          aria-hidden={!state.loading}
          size={13}
          className={`shrink-0 ${state.loading ? "animate-spin text-work" : "text-muted"}`}
        />
        <input
          ref={field}
          type="text"
          readOnly={!held}
          aria-label={t.address}
          title={url ?? undefined}
          placeholder={held ? t.addressHint : undefined}
          autoComplete="off"
          spellCheck={false}
          value={draft ?? url ?? (held ? "" : (state.title ?? ""))}
          onChange={(event) => setDraft(event.target.value)}
          onFocus={(event) => held && event.currentTarget.select()}
          onKeyDown={(event) => {
            if (!held) {
              return;
            }
            if (event.key === "Enter") {
              event.preventDefault();
              go();
            } else if (event.key === "Escape") {
              setDraft(null);
              event.currentTarget.blur();
            }
          }}
          className="min-w-0 flex-1 truncate bg-transparent font-mono text-ink-soft text-xs outline-none"
        />
      </div>
      {web && url && (
        <Button
          variant="ghost"
          size="sm"
          icon={ExternalLink}
          label={t.openOutside}
          onClick={() => attempt(t.openFailed, () => host.openUrl(url))}
        />
      )}
    </div>
  );
}
