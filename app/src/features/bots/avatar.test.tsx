import { act, cleanup, render } from "@testing-library/react";
import { afterEach, describe, expect, test, vi } from "vitest";
import cheerFrames from "../../mascot-cheer.css?raw";
import flameFrames from "../../mascot-flame.css?raw";
import { BOTLOFT_COLOR, BotAvatar, CHEER_MS } from "./BotAvatar";
import { BotStateBadge } from "./BotStateBadge";
import { ListAvatar } from "./ListAvatar";
import { BODY, DRAWN_IN, OUTLINE, SHADES } from "./mascotArt";
import { retint, toHex, toHsl } from "./retint";

afterEach(cleanup);

/** The commands of a path, which must match for the browser to morph it. */
const commands = (d: string) => d.replace(/[^MCZ]/g, "");

const channels = (hex: string): [number, number, number] => {
  const at = (index: number) => Number.parseInt(hex.slice(index, index + 2), 16);
  return [at(1), at(3), at(5)];
};

describe("retint", () => {
  test("hex and hsl round-trip", () => {
    for (const hex of ["#ff7a59", "#5ec8ff", "#a48bff", "#9be564", "#e8d5b0", "#121212"]) {
      expect(toHex(...toHsl(hex))).toBe(hex);
    }
  });

  test("the color it was drawn in, Botloft's own, keeps the artwork as drawn", () => {
    const drawnIn = toHex(DRAWN_IN.h, DRAWN_IN.s, DRAWN_IN.l);
    for (const color of [drawnIn, BOTLOFT_COLOR]) {
      for (const { fill } of SHADES) {
        const [r, g, b] = channels(retint(fill, color));
        const [r0, g0, b0] = channels(fill.toLowerCase());
        expect(Math.abs(r - r0) + Math.abs(g - g0) + Math.abs(b - b0)).toBeLessThanOrEqual(6);
      }
    }
  });

  test("white leaves no hue", () => {
    const [r, g, b] = channels(retint(BODY, "#ffffff"));
    expect(r).toBe(g);
    expect(g).toBe(b);
  });
});

describe("mascot", () => {
  test("every shading layer has its own color, which keys it", () => {
    expect(new Set(SHADES.map((shade) => shade.fill)).size).toBe(SHADES.length);
  });

  test("the outline is the body and the drop of fire above it", () => {
    expect(commands(OUTLINE).match(/M/g)).toHaveLength(2);
  });

  test("every flame frame keeps the commands of the outline, so it morphs", () => {
    const frames = [...flameFrames.matchAll(/d: path\(\s*"([^"]+)"/g)].map(
      (match) => match[1] ?? "",
    );
    expect(frames.length).toBeGreaterThan(30);
    for (const frame of frames) {
      expect(commands(frame)).toBe(commands(OUTLINE));
    }
  });

  test("the eyes glance together, and the shading keeps its holes", () => {
    const { container } = render(<BotAvatar color="#ff7a59" mood="idle" />);
    expect(container.querySelectorAll(".avatar-look .avatar-eye")).toHaveLength(2);
    const shades = container.querySelectorAll("[filter] path");
    expect(shades).toHaveLength(SHADES.length);
    for (const shade of shades) {
      expect(shade.getAttribute("fill-rule")).toBe("evenodd");
    }
  });

  test("each avatar clips to its own outline", () => {
    const { container } = render(
      <>
        <BotAvatar color="#ff7a59" />
        <BotAvatar color="#5ec8ff" />
      </>,
    );
    const clips = [...container.querySelectorAll("clipPath")].map((clip) => clip.id);
    expect(new Set(clips).size).toBe(2);
    const used = [...container.querySelectorAll("[clip-path]")].map((g) =>
      g.getAttribute("clip-path"),
    );
    expect(used).toEqual(clips.map((id) => `url(#${id})`));
  });

  test("the mood drives the motion, and only a working bot throws embers", () => {
    const { container, rerender } = render(<BotAvatar color="#ff7a59" mood="idle" />);
    const svg = () => container.querySelector("svg");
    expect(svg()?.dataset.mood).toBe("idle");
    expect(container.querySelectorAll(".avatar-spark")).toHaveLength(0);

    rerender(<BotAvatar color="#ff7a59" mood="working" />);
    expect(container.querySelectorAll(".avatar-spark")).toHaveLength(3);

    rerender(<BotAvatar color="#ff7a59" />);
    expect(svg()?.dataset.mood).toBeUndefined();
  });

  test("cheers once when its bot finishes, not when it shows up idle", () => {
    vi.useFakeTimers();
    try {
      const { container, rerender } = render(<BotAvatar color="#ff7a59" mood="idle" />);
      const cheering = () => container.querySelector("svg")?.hasAttribute("data-cheer");
      expect(container.querySelectorAll(".avatar-happy")).toHaveLength(2);
      expect(cheering()).toBe(false);

      rerender(<BotAvatar color="#ff7a59" mood="working" />);
      expect(cheering()).toBe(false);
      rerender(<BotAvatar color="#ff7a59" mood="idle" />);
      expect(cheering()).toBe(true);
      act(() => vi.advanceTimersByTime(CHEER_MS));
      expect(cheering()).toBe(false);

      rerender(<BotAvatar color="#ff7a59" mood="waiting" />);
      rerender(<BotAvatar color="#ff7a59" mood="idle" />);
      expect(cheering()).toBe(false);
    } finally {
      vi.useRealTimers();
    }
  });

  test("shut and smiling eyes lean with the line through the open ones", () => {
    const { container } = render(<BotAvatar color="#ff7a59" mood="sleeping" />);
    for (const arc of container.querySelectorAll(".avatar-closed, .avatar-happy")) {
      const turn = arc.parentElement?.getAttribute("transform")?.match(/^rotate\((-?[\d.]+) /);
      expect(Number(turn?.[1])).toBeCloseTo(-14.3, 0);
    }
  });

  test("the cheer ends at rest, so less motion shows none of it", () => {
    const end = (name: string) =>
      cheerFrames
        .match(new RegExp(`@keyframes ${name} \\{([\\s\\S]*?)\\n\\}`))?.[1]
        ?.split(/100% \{/)
        .at(-1);
    expect(end("cheer-hop")).toContain("transform: translateY(0) scale(1, 1);");
    expect(end("cheer-open")).toMatch(/opacity: 1;\s*clip-path: inset\(0\);/);
    expect(end("cheer-smile")).toContain("opacity: 0;");
  });

  test("framed sits on the icon square without the thin edge", () => {
    const { container } = render(<BotAvatar color={BOTLOFT_COLOR} framed />);
    expect(container.querySelector("svg")?.getAttribute("class")).toContain("bg-[#0b0b0b]");
    expect(container.querySelector('[stroke="var(--avatar-edge)"]')).toBeNull();
  });
});

describe("state badge", () => {
  test("idle shows nothing and working runs its line", () => {
    const { container, rerender } = render(
      <BotStateBadge bot={{ state: "idle", paused: false }} />,
    );
    expect(container.textContent).toBe("");

    rerender(<BotStateBadge bot={{ state: "busy", paused: false }} />);
    expect(container.querySelector(".state-trace")?.getAttribute("pathLength")).toBe("100");
  });

  test("fades in when the bot starts working, not when it was already working", () => {
    const { container, rerender } = render(
      <BotStateBadge bot={{ state: "busy", paused: false }} />,
    );
    expect(container.querySelector(".animate-fade")).toBeNull();

    rerender(<BotStateBadge bot={{ state: "idle", paused: false }} />);
    rerender(<BotStateBadge bot={{ state: "busy", paused: false }} />);
    expect(container.querySelector(".animate-fade")).not.toBeNull();
  });
});

describe("ListAvatar", () => {
  const bot = (state: "idle" | "busy" | "offline") => ({
    color: BOTLOFT_COLOR,
    state,
    paused: false,
  });

  test("a sleeping bot keeps its look but does not move; an idle or working one moves", () => {
    const { container, rerender } = render(
      <ListAvatar bot={bot("idle")} crewPaused={false} size={32} />,
    );
    const svg = () => container.querySelector("svg") as SVGElement;
    expect(svg().dataset.mood).toBe("idle");
    expect(svg().hasAttribute("data-still")).toBe(false);

    rerender(<ListAvatar bot={bot("offline")} crewPaused={false} size={32} />);
    expect(svg().dataset.mood).toBe("sleeping");
    expect(svg().hasAttribute("data-still")).toBe(true);

    rerender(<ListAvatar bot={bot("busy")} crewPaused={false} size={32} />);
    expect(svg().dataset.mood).toBe("working");
    expect(svg().hasAttribute("data-still")).toBe(false);
  });
});
