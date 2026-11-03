// What a bot's mascot shows on top of its mood (spec 15.3), from one table:
// what it is doing, as a small drawing by its head (MascotBadge.tsx), and
// its face, as the shape of its eyes (mascot-look.css). Working, the tool
// it runs picks the drawing: a radar while it searches the web, a globe
// while it browses, a pencil while it writes a file, a terminal while it
// runs a command, a lens while it reads, a cursor while it works the
// desktop, and three dots while it thinks between tools; waiting for the
// owner, a "!". The face: focused while it writes, runs or reads, worried
// for a moment after a tool fails, sleepy after waiting two minutes.

import { useEffect, useState } from "react";
import type { Activity, BotId } from "../../lib/protocol.gen";
import { activityOf } from "../../store/app";
import { useOptionalApi, useOptionalApp } from "../../store/context";
import { toolKey } from "../chat/toolNames";
import type { Mood } from "./BotAvatar";

export type Doing = "search" | "browse" | "write" | "run" | "read" | "control" | "think" | "ask";
export type Face = "focus" | "worried" | "sleepy";

/** How long the face stays worried after a tool fails. */
export const WORRIED_MS = 5000;
/** How long the owner may keep a bot waiting before its eyes grow heavy. */
export const SLEEPY_AFTER_MS = 120_000;

const BY_KEY: Record<string, Doing> = {
  WebSearch: "search",
  WebFetch: "browse",
  Write: "write",
  Edit: "write",
  MultiEdit: "write",
  NotebookEdit: "write",
  Bash: "run",
  BashOutput: "run",
  KillShell: "run",
  PowerShell: "run",
  Read: "read",
  Grep: "read",
  Glob: "read",
  LS: "read",
};

/** What a tool shows the bot doing; any other tool, thinking. */
export function doingOf(tool: string): Doing {
  const key = toolKey(tool);
  const known = BY_KEY[key];
  if (known) return known;
  if (key.startsWith("browser_")) return "browse";
  if (key.startsWith("desktop_")) return "control";
  if (key.toLowerCase().includes("search")) return "search";
  return "think";
}

/** The drawing by the head, from the mood and the bot's last chat item. */
export function doingNow(mood: Mood | undefined, activity: Activity | null): Doing | null {
  if (mood === "waiting") return "ask";
  if (mood !== "working") return null;
  return activity?.kind === "tool" && activity.tool ? doingOf(activity.tool) : "think";
}

/** The face, from the mood, the drawing and what just happened. */
export function faceNow(
  mood: Mood | undefined,
  doing: Doing | null,
  { worried, longWait }: { worried: boolean; longWait: boolean },
): Face | null {
  if (worried && mood !== "idle" && mood !== "sleeping") return "worried";
  if (mood === "waiting" && longWait) return "sleepy";
  if (doing === "write" || doing === "run" || doing === "read") return "focus";
  return null;
}

/** True for `ms` after `on` turns true, and while it stays. */
function useAfter(on: boolean, ms: number): boolean {
  const [late, setLate] = useState(false);
  useEffect(() => {
    setLate(false);
    if (!on) return;
    const timer = setTimeout(() => setLate(true), ms);
    return () => clearTimeout(timer);
  }, [on, ms]);
  return on && late;
}

/** True for a moment after one of `botId`'s tools fails. */
function useWorried(botId: BotId | undefined): boolean {
  const api = useOptionalApi();
  const [worried, setWorried] = useState(false);
  useEffect(() => {
    if (!api || !botId) return;
    let timer: ReturnType<typeof setTimeout> | undefined;
    const stop = api.subscribe((event) => {
      if (event.name !== "chat.item") return;
      const { item } = event.params;
      if (item.botId !== botId || item.body.kind !== "tool" || item.body.status !== "failed") {
        return;
      }
      setWorried(true);
      clearTimeout(timer);
      timer = setTimeout(() => setWorried(false), WORRIED_MS);
    });
    return () => {
      stop();
      clearTimeout(timer);
    };
  }, [api, botId]);
  return worried;
}

/** What the mascot of `botId` in `mood` shows on top of it. */
export function useMascotLook(
  botId: BotId | undefined,
  mood: Mood | undefined,
): { doing: Doing | null; face: Face | null } {
  const activity = useOptionalApp((state) => (botId ? activityOf(state, botId) : null), null);
  const worried = useWorried(botId);
  const longWait = useAfter(mood === "waiting", SLEEPY_AFTER_MS);
  const doing = doingNow(mood, activity);
  return { doing, face: faceNow(mood, doing, { worried, longWait }) };
}
