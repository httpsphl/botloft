// A bot's mascot in a list (the sidebar, a crew's cards), with the bot's
// mood, what it is doing and its face (mascotLook.ts), its glances and, for
// a bot just created, its pop in (BotAvatar.tsx).

import { useState } from "react";
import type { Bot } from "../../lib/protocol.gen";
import { APP_OPENED } from "../../ui/motion";
import { BotAvatar, moodOf } from "./BotAvatar";
import { useMascotLook } from "./mascotLook";
import { useIsMaker } from "./useIsMaker";

/** A bot's mascot in a list (the sidebar, a crew's cards). */
export function ListAvatar({
  bot,
  crewPaused,
  size,
}: {
  bot: Pick<Bot, "color" | "state" | "paused"> & Partial<Pick<Bot, "id" | "createdAt">>;
  crewPaused: boolean;
  size: number;
}) {
  // A bot created a moment ago, while the app was open, pops in.
  const [arriving] = useState(
    () =>
      bot.createdAt !== undefined &&
      bot.createdAt > APP_OPENED &&
      Date.now() - bot.createdAt < ARRIVING_FOR_MS,
  );
  const mood = moodOf(bot, crewPaused);
  const { doing, face } = useMascotLook(bot.id, mood);
  const maker = useIsMaker(bot.id);
  return (
    <BotAvatar
      color={bot.color}
      size={size}
      mood={mood}
      doing={doing}
      face={face}
      maker={maker}
      botId={bot.id}
      arriving={arriving}
      starting={bot.state === "launching"}
    />
  );
}

/** How long after its creation a bot's mascot still pops in when it shows. */
const ARRIVING_FOR_MS = 10_000;
