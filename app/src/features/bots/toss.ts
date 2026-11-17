// An ember between bots (spec 15.3): when one bot writes to another, an
// ember in the writer's color flies in an arc from its mascot to the
// other's, which hops as it catches it, while the two glance at each other
// (useGlance.ts). Of all the pairs of their mascots on screen, the nearest
// one plays it, once a message. Played with GSAP, on top of everything,
// and gone when it lands. Not while the mascots stand still (eyeMoments.ts).

import { gsap } from "gsap";
import type { BotId, MessageId } from "../../lib/protocol.gen";
import { resting } from "./eyeMoments";
import { EMBER } from "./mascotArt";
import { hop } from "./pointer";
import { retint } from "./retint";

/** The messages whose ember already flew, newest last. */
const tossed: MessageId[] = [];
const REMEMBER = 50;

interface Spot {
  svg: SVGSVGElement;
  x: number;
  y: number;
  size: number;
}

function spotsOf(botId: BotId): Spot[] {
  return [...document.querySelectorAll<SVGSVGElement>(`svg[data-bot="${botId}"]`)].flatMap(
    (svg) => {
      const box = svg.getBoundingClientRect();
      if (box.width === 0 || box.height === 0) return [];
      return [{ svg, x: box.left + box.width / 2, y: box.top + box.height / 2, size: box.width }];
    },
  );
}

/** The two mascots of `from` and `to` that sit nearest each other on screen. */
export function nearestPair(from: BotId, to: BotId): [Spot, Spot] | null {
  let best: [Spot, Spot] | null = null;
  let shortest = Number.POSITIVE_INFINITY;
  for (const a of spotsOf(from)) {
    for (const b of spotsOf(to)) {
      const distance = Math.hypot(b.x - a.x, b.y - a.y);
      if (distance >= 1 && distance < shortest) {
        shortest = distance;
        best = [a, b];
      }
    }
  }
  return best;
}

/** Throws an ember from `from` to `to` for message `id`, once. */
export function toss(id: MessageId, from: BotId, to: BotId): void {
  if (tossed.includes(id) || resting()) return;
  const pair = nearestPair(from, to);
  if (!pair) return;
  tossed.push(id);
  if (tossed.length > REMEMBER) tossed.shift();
  const [a, b] = pair;
  const color = a.svg.dataset.color ?? "#ff7a59";
  const size = Math.max(10, Math.min(28, a.size * 0.45));
  const ember = document.createElementNS("http://www.w3.org/2000/svg", "svg");
  ember.setAttribute("viewBox", "-30 -36 60 84");
  ember.setAttribute("aria-hidden", "true");
  ember.setAttribute("class", "avatar-toss");
  Object.assign(ember.style, {
    width: `${size}px`,
    height: `${size * 1.4}px`,
    filter: `drop-shadow(0 0 ${size / 3}px ${retint(EMBER.light, color)})`,
  });
  const path = document.createElementNS("http://www.w3.org/2000/svg", "path");
  path.setAttribute("d", EMBER.d);
  path.setAttribute("fill", retint(EMBER.base, color));
  ember.append(path);
  document.body.append(ember);

  // The arc bows across the line between them: up over a row of cards,
  // out to the right of a column, as in the sidebar.
  const distance = Math.hypot(b.x - a.x, b.y - a.y);
  let across = { x: -(b.y - a.y) / distance, y: (b.x - a.x) / distance };
  if (across.y > 0.01 || (Math.abs(across.y) <= 0.01 && across.x < 0)) {
    across = { x: -across.x, y: -across.y };
  }
  const lift = Math.max(30, distance * 0.3);
  const bend = { x: (a.x + b.x) / 2 + across.x * lift, y: (a.y + b.y) / 2 + across.y * lift };
  const flight = { t: 0 };
  const place = () => {
    const t = flight.t;
    const u = 1 - t;
    const x = u * u * a.x + 2 * u * t * bend.x + t * t * b.x;
    const y = u * u * a.y + 2 * u * t * bend.y + t * t * b.y;
    const scale = 0.45 + Math.sin(Math.PI * t) * 0.45;
    ember.style.transform = `translate(${x - size / 2}px, ${y - size * 0.7}px) scale(${scale})`;
  };
  place();
  gsap.to(flight, {
    t: 1,
    duration: Math.max(0.45, Math.min(0.9, distance / 900)),
    ease: "power1.inOut",
    onUpdate: place,
    onComplete: () => {
      ember.remove();
      hop(b.svg, 0.5);
    },
  });
}
