// Bots calling each other (spec 15.3): a message one bot sends another is
// a call until the other picks it up, and stays a moment after, saying so.
// The store keeps who called whom; whether it was picked up comes from the
// message's delivery, which every connection loads again.

import type { BotId, CrewId, Delivery, Message, MessageId } from "../lib/protocol.gen";

export interface Call {
  messageId: MessageId;
  crewId: CrewId;
  fromBotId: BotId;
  toBotId: BotId;
  at: number;
}

/** A call as it shows: still ringing, or picked up at `readAt`. */
export interface LiveCall extends Call {
  readAt: number | null;
}

/** How long "picked it up" stays after the other bot read the message. */
export const PICKED_UP_MS = 4000;
/** A call nobody picked up in this long is old news, not a call. */
const RING_MS = 60 * 60_000;

interface CallState {
  calls: Record<MessageId, Call>;
  deliveries: Record<MessageId, Delivery>;
}

/** The call as it stands at `now`, or `null` once it is over. */
function live(call: Call, deliveries: CallState["deliveries"], now: number): LiveCall | null {
  const delivery = deliveries[call.messageId];
  if (delivery?.state === "dead") {
    return null;
  }
  const readAt = delivery?.readAt ?? null;
  if (readAt === null ? now - call.at > RING_MS : now - readAt > PICKED_UP_MS) {
    return null;
  }
  return { ...call, readAt };
}

/** A new message: a call if one bot sent it to another. Over calls leave. */
export function withCall(state: CallState, message: Message, now = Date.now()) {
  const { fromBotId, toBotId } = message;
  if (message.fromKind !== "bot" || !fromBotId || fromBotId === toBotId) {
    return null;
  }
  const calls: Record<MessageId, Call> = {};
  for (const call of Object.values(state.calls)) {
    if (live(call, state.deliveries, now)) {
      calls[call.messageId] = call;
    }
  }
  calls[message.id] = {
    messageId: message.id,
    crewId: message.crewId,
    fromBotId,
    toBotId,
    at: message.createdAt,
  };
  return { calls };
}

/**
 * The calls going on, oldest first, one per pair of bots (its latest),
 * kept to those `keep` wants.
 */
export function liveCalls(
  state: CallState,
  now: number,
  keep: (call: Call) => boolean = () => true,
): LiveCall[] {
  const byPair = new Map<string, LiveCall>();
  for (const call of Object.values(state.calls)) {
    const shown = keep(call) ? live(call, state.deliveries, now) : null;
    const pair = `${call.fromBotId}>${call.toBotId}`;
    const before = byPair.get(pair);
    if (shown && (!before || before.at <= shown.at)) {
      byPair.set(pair, shown);
    }
  }
  return [...byPair.values()].sort((a, b) => a.at - b.at);
}

/** When the next shown call ends by itself, to look again then. */
export function nextChange(calls: LiveCall[], now: number): number | null {
  const ends = calls.map((call) =>
    call.readAt === null ? call.at + RING_MS : call.readAt + PICKED_UP_MS,
  );
  return ends.length ? Math.max(0, Math.min(...ends) - now) + 1 : null;
}
