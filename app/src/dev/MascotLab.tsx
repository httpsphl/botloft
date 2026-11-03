// A page to look over the mascot's looks (`?lab`, dev only): the Maker in
// its sizes and moods, every badge of what a bot is doing, every face, and
// the fidgets. The buttons carry the moods' own names, as in the code:
// nothing here reaches the owner.

import { type ReactNode, useRef, useState } from "react";
import { BotAvatar, type Mood } from "../features/bots/BotAvatar";
import { type Fidget, fidget } from "../features/bots/fidgets";
import type { Doing, Face } from "../features/bots/mascotLook";

const MOODS: Mood[] = ["working", "idle", "waiting", "awake", "tired"];
const DOINGS: Doing[] = ["search", "browse", "write", "run", "read", "control", "think", "ask"];
const FACES: Face[] = ["focus", "worried", "sleepy"];
const FIDGETS: Fidget[] = ["bob", "stretch", "sway"];

function Row({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section style={{ display: "grid", gap: 12 }}>
      <h2 style={{ fontFamily: "monospace", fontSize: 13, opacity: 0.6 }}>{title}</h2>
      <div style={{ display: "flex", flexWrap: "wrap", alignItems: "flex-end", gap: 28 }}>
        {children}
      </div>
    </section>
  );
}

function Pick<T extends string>({
  all,
  now,
  set,
}: {
  all: readonly T[];
  now: T | null;
  set: (value: T) => void;
}) {
  return (
    <div style={{ display: "flex", gap: 6 }}>
      {all.map((value) => (
        <button
          key={value}
          type="button"
          onClick={() => set(value)}
          style={{
            fontFamily: "monospace",
            fontSize: 12,
            padding: "4px 10px",
            borderRadius: 8,
            border: "1px solid var(--line-strong)",
            background: value === now ? "var(--accent)" : "var(--panel)",
            color: value === now ? "#fff" : "var(--ink)",
          }}
        >
          {value}
        </button>
      ))}
    </div>
  );
}

export function MascotLab() {
  const [mood, setMood] = useState<Mood>("working");
  const fidgeting = useRef<HTMLDivElement>(null);
  /** From working to idle: the mascot cheers. */
  const cheer = () => {
    setMood("working");
    setTimeout(() => setMood("idle"), 120);
  };
  return (
    <main
      style={{
        padding: 32,
        display: "grid",
        gap: 36,
        background: "var(--canvas)",
        color: "var(--ink)",
        minHeight: "100vh",
      }}
    >
      <Row title="maker">
        <div style={{ display: "grid", gap: 8 }}>
          <Pick all={MOODS} now={mood} set={setMood} />
          <Pick all={["cheer"] as const} now={null} set={cheer} />
        </div>
      </Row>
      <Row title="maker · 28 · 48 · 96 · scene 120">
        <BotAvatar color="#ff7a59" size={28} mood={mood} maker />
        <BotAvatar color="#ff7a59" size={48} mood={mood} maker />
        <BotAvatar color="#ff7a59" size={96} mood={mood} maker />
        <BotAvatar color="#ff7a59" size={120} mood={mood} maker scene />
      </Row>
      <Row title="doing">
        {DOINGS.map((doing) => (
          <div key={doing} style={{ display: "grid", justifyItems: "center", gap: 4 }}>
            <BotAvatar
              color="#5ec8ff"
              size={64}
              mood={doing === "ask" ? "waiting" : "working"}
              doing={doing}
            />
            <code style={{ fontSize: 11, opacity: 0.6 }}>{doing}</code>
          </div>
        ))}
        <BotAvatar color="#4ade5c" size={32} mood="working" doing="write" />
        <BotAvatar color="#ffcf3f" size={24} mood="working" doing="search" />
      </Row>
      <Row title="face">
        {FACES.map((face) => (
          <div key={face} style={{ display: "grid", justifyItems: "center", gap: 4 }}>
            <BotAvatar color="#8b6bff" size={72} mood="awake" face={face} />
            <code style={{ fontSize: 11, opacity: 0.6 }}>{face}</code>
          </div>
        ))}
      </Row>
      <Row title="fidget">
        <div ref={fidgeting} style={{ display: "flex", gap: 24 }}>
          {["#ff5fa8", "#3fe0c5", "#ff7a59"].map((color) => (
            <BotAvatar key={color} color={color} size={72} mood="awake" />
          ))}
        </div>
        <Pick
          all={FIDGETS}
          now={null}
          set={(kind) => {
            for (const svg of fidgeting.current?.querySelectorAll("svg") ?? []) fidget(svg, kind);
          }}
        />
      </Row>
    </main>
  );
}
