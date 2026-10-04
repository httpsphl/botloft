// A bot's mascot in a list (the sidebar, a crew's cards), with the bot's
// mood, its glances and, for a bot just created, its pop in (BotAvatar.tsx).

import { useState } from "react";
import type { Bot } from "../../lib/protocol.gen";
import { APP_OPENED } from "../../ui/motion";
import { BotAvatar, moodOf } from "./BotAvatar";

/**
 * A bot's mascot in a list (the sidebar, a crew's cards): it moves while
 * the bot is awake, idle too, since the owner reads an idle bot by its
 * slow flame and glances. A sleeping one keeps its look but does not
 * move: a morphing flame repaints every frame, and a stopped bot has
 * nothing to show.
 */
export function ListAvatar({
  bot,
  crewPaused,
  size,
}: {
  bot: Pick<Bot, "color" | "state" | "paused"> & Partial<Pick<Bot, "id" | "createdAt">>;
  crewPaused: boolean;
  size: number;
}) {
  const mood = moodOf(bot, crewPaused);
  const still = mood === "sleeping";
  // A bot created a moment ago, while the app was open, pops in.
  const [arriving] = useState(
    () =>
      bot.createdAt !== undefined &&
      bot.createdAt > APP_OPENED &&
      Date.now() - bot.createdAt < ARRIVING_FOR_MS,
  );
  return (
    <BotAvatar
      color={bot.color}
      size={size}
      mood={mood}
      still={still}
      botId={bot.id}
      arriving={arriving}
      starting={bot.state === "launching"}
    />
  );
}

/** How long after its creation a bot's mascot still pops in when it shows. */
const ARRIVING_FOR_MS = 10_000;
