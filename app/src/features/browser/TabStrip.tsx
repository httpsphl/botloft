// The tabs of a bot's browser (spec 21.8): what it has open, by title and
// site, in the order they opened. With the browser in the owner's hands
// they switch tabs and open a new one (spec 21.10); until then they only
// show.

import { Plus } from "lucide-react";
import { useEffect, useRef } from "react";
import { useT } from "../../i18n";
import type { BrowserState, BrowserTab } from "../../lib/protocol.gen";
import { Button } from "../../ui/Button";
import type { Hands } from "./useHands";

/** A tab's site: the host without `www.`, or the file's name. */
export function siteOf(url: string): string {
  try {
    if (/^https?:\/\//i.test(url)) {
      return new URL(url).hostname.replace(/^www\./, "");
    }
    if (/^file:/i.test(url)) {
      return decodeURIComponent(url.split(/[?#]/)[0]?.split("/").pop() ?? "");
    }
  } catch {
    // Not an address anyone can read: the tab shows its title alone.
  }
  return "";
}

export function TabStrip({
  state,
  hands,
  onAdd,
}: {
  state: BrowserState;
  hands: Hands;
  /** The owner asked for a new tab. */
  onAdd(): void;
}) {
  const t = useT().browser.tabs;
  const held = hands.held;
  const list = useRef<HTMLDivElement>(null);
  const active = state.tabs.find((tab) => tab.active)?.id;
  // The active tab stays in sight when there are more than fit.
  // biome-ignore lint/correctness/useExhaustiveDependencies: looked for again when another tab is the active one
  useEffect(() => {
    const selected = list.current?.querySelector('[aria-selected="true"]');
    selected?.scrollIntoView?.({ block: "nearest", inline: "nearest" });
  }, [active]);

  return (
    <div className="flex h-11 shrink-0 items-center gap-1 border-line border-b pr-1.5 pl-2">
      <div
        ref={list}
        role="tablist"
        aria-label={t.label}
        className="flex min-w-0 flex-1 gap-1 overflow-x-auto [scrollbar-width:none]"
      >
        {state.tabs.map((tab) => (
          <TabButton
            key={tab.id}
            tab={tab}
            held={held}
            onPick={() => void hands.switchTab(tab.id)}
          />
        ))}
      </div>
      <Button
        variant="ghost"
        size="sm"
        icon={Plus}
        label={t.add}
        title={held ? t.add : t.takeFirst}
        aria-disabled={!held}
        className={held ? "" : "cursor-default opacity-45"}
        onClick={() => held && onAdd()}
      />
    </div>
  );
}

function TabButton({ tab, held, onPick }: { tab: BrowserTab; held: boolean; onPick(): void }) {
  const t = useT().browser.tabs;
  const blank = tab.url === "" || tab.url === "about:blank";
  const host = siteOf(tab.url);
  const title = blank ? t.blank : tab.title || host || tab.url;
  // Under the title, unless the site is all there is to show.
  const site = host === title ? "" : host;
  const hint = [title, blank ? null : tab.url, held ? null : t.takeFirst].filter(Boolean);
  const look = tab.active
    ? "bg-sunken text-ink"
    : held
      ? "text-muted hover:bg-sunken/60 hover:text-ink"
      : "text-muted";
  return (
    <button
      type="button"
      role="tab"
      aria-selected={tab.active}
      aria-disabled={!held}
      aria-label={site ? `${title}, ${site}` : title}
      title={hint.join("\n")}
      onClick={() => held && !tab.active && onPick()}
      className={`flex h-9 min-w-20 max-w-44 flex-1 flex-col justify-center rounded-lg px-2.5 text-left transition-colors ${look} ${held ? "" : "cursor-default"}`}
    >
      <span className="truncate font-medium text-xs leading-4">{title}</span>
      {site && <span className="truncate text-[11px] text-muted leading-3.5">{site}</span>}
    </button>
  );
}
