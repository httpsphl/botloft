import { gsap } from "gsap";
import { afterEach, describe, expect, test } from "vitest";
import type { BotId, MessageId } from "../../lib/protocol.gen";
import { nearestPair, toss } from "./toss";

const SVG = "http://www.w3.org/2000/svg";

function mascot(botId: string, left: number, top: number): SVGSVGElement {
  const svg = document.createElementNS(SVG, "svg");
  svg.setAttribute("data-bot", botId);
  svg.setAttribute("data-color", "#5ec8ff");
  const poke = document.createElementNS(SVG, "g");
  poke.setAttribute("class", "avatar-poke");
  svg.append(poke);
  svg.getBoundingClientRect = () => new DOMRect(left, top, 32, 32);
  document.body.append(svg);
  return svg;
}

const scout = "bot_scout" as BotId;
const writer = "bot_writer" as BotId;
const flying = () => document.querySelectorAll(".avatar-toss");

afterEach(() => {
  gsap.globalTimeline.clear();
  document.body.replaceChildren();
  document.documentElement.removeAttribute("data-blurred");
});

describe("an ember between agents", () => {
  test("flies between the two mascots that sit nearest each other", () => {
    const sidebar = mascot(scout, 0, 100);
    const header = mascot(scout, 600, 0);
    const card = mascot(writer, 500, 0);
    mascot(writer, 0, 900);
    const pair = nearestPair(scout, writer);
    expect(pair?.[0].svg).toBe(header);
    expect(pair?.[1].svg).toBe(card);
    expect(sidebar).toBeDefined();
  });

  test("flies once a message, lands and is gone, and the other catches it", () => {
    mascot(scout, 0, 100);
    const catcher = mascot(writer, 0, 300);
    toss("msg_1" as MessageId, scout, writer);
    toss("msg_1" as MessageId, scout, writer);
    expect(flying()).toHaveLength(1);

    const flight = gsap.globalTimeline.getChildren(false).at(-1);
    flight?.progress(1);
    expect(flying()).toHaveLength(0);
    expect(gsap.getTweensOf(catcher.querySelector(".avatar-poke"))).not.toHaveLength(0);
  });

  test("none flies while the window has no focus, or with no partner on screen", () => {
    mascot(scout, 0, 100);
    toss("msg_2" as MessageId, scout, writer);
    expect(flying()).toHaveLength(0);

    mascot(writer, 0, 300);
    document.documentElement.setAttribute("data-blurred", "");
    toss("msg_3" as MessageId, scout, writer);
    expect(flying()).toHaveLength(0);
  });
});
