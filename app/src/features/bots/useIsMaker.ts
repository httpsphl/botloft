// Whether a bot is the Maker (spec 36.2), so its mascot shows as the Maker's
// white flame with its wand (BotAvatar `maker`, spec 36.3). Outside the app
// (no store), no bot is.

import type { BotId } from "../../lib/protocol.gen";
import { makerBot } from "../../store/app";
import { useOptionalApp } from "../../store/context";

export function useIsMaker(botId: BotId | undefined): boolean {
  return useOptionalApp((state) => botId !== undefined && makerBot(state)?.id === botId, false);
}
