// The cards waiting for the owner in a bot's chat (spec 15.3): when one is
// out of view, say above an approval that newer messages pushed up, a pill
// over the chat's end says so, and a click brings the card into view with
// a band behind it (search.css), the oldest first.

import { ArrowDown, ArrowUp, Hand } from "lucide-react";
import { type RefObject, useEffect, useMemo, useState } from "react";
import { useT } from "../../i18n";
import type { ChatItem } from "../../lib/protocol.gen";
import { isWaiting } from "./rows";

/** How much of a card counts as seen: this many pixels, or all of a smaller one. */
const SEEN_PX = 48;

/** As long as the band's animation (search.css). */
const FOCUS_MS = 2400;

interface Away {
  id: string;
  /** Above the view, so the arrow points up. */
  above: boolean;
}

const same = (a: Away[], b: Away[]) =>
  a.length === b.length &&
  a.every((each, index) => each.id === b[index]?.id && each.above === b[index]?.above);

function cardOf(scroller: HTMLElement, id: string): Element | null {
  return scroller.querySelector(`[data-item="${id}"]`);
}

/** The waiting cards out of view, in the chat's order. */
function useAway(scroller: RefObject<HTMLDivElement | null>, ids: string): Away[] {
  const [away, setAway] = useState<Away[]>([]);
  useEffect(() => {
    const element = scroller.current;
    const waiting = ids ? ids.split(" ") : [];
    if (!element || waiting.length === 0) {
      setAway([]);
      return;
    }
    let frame = 0;
    const measure = () => {
      cancelAnimationFrame(frame);
      frame = requestAnimationFrame(() => {
        const view = element.getBoundingClientRect();
        const out: Away[] = [];
        for (const id of waiting) {
          const card = cardOf(element, id)?.getBoundingClientRect();
          if (!card) {
            continue;
          }
          const shown = Math.min(card.bottom, view.bottom) - Math.max(card.top, view.top);
          if (shown < Math.min(SEEN_PX, card.height)) {
            out.push({ id, above: card.top < view.top });
          }
        }
        setAway((before) => (same(before, out) ? before : out));
      });
    };
    measure();
    element.addEventListener("scroll", measure, { passive: true });
    window.addEventListener("resize", measure);
    return () => {
      cancelAnimationFrame(frame);
      element.removeEventListener("scroll", measure);
      window.removeEventListener("resize", measure);
    };
  }, [scroller, ids]);
  return away;
}

export function WaitingPill({
  items,
  scroller,
}: {
  items: ChatItem[];
  scroller: RefObject<HTMLDivElement | null>;
}) {
  const t = useT().chat.view;
  const ids = useMemo(
    () =>
      items
        .filter(isWaiting)
        .map((item) => item.id)
        .join(" "),
    [items],
  );
  const away = useAway(scroller, ids);
  const first = away[0];
  if (!first) {
    return null;
  }
  const show = () => {
    const card = scroller.current && cardOf(scroller.current, first.id);
    if (!card) {
      return;
    }
    card.scrollIntoView?.({ block: "center" });
    card.classList.add("search-focus");
    setTimeout(() => card.classList.remove("search-focus"), FOCUS_MS);
  };
  const Arrow = first.above ? ArrowUp : ArrowDown;
  return (
    <button
      type="button"
      title={t.showWaiting}
      onClick={show}
      className="flex animate-rise items-center gap-1.5 rounded-full border border-warn/45 bg-panel py-1 pr-2.5 pl-3 font-medium text-sm text-warn shadow-sm transition-colors hover:bg-sunken"
    >
      <Hand aria-hidden size={14} className="shrink-0" />
      {t.waiting(away.length)}
      <Arrow aria-hidden size={14} className="shrink-0" />
    </button>
  );
}
