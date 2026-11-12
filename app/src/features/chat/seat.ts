// Which conversation of the agent the chat is on (spec 40.7): what its
// controls act on. A call about the main one names none, as before
// conversations existed.

import { createContext, useContext } from "react";
import type { ConversationId } from "../../lib/protocol.gen";

export interface ChatSeat {
  conversationId: ConversationId;
  main: boolean;
}

export const SeatContext = createContext<ChatSeat | null>(null);

/** `{ conversationId }` for a call about a conversation that is not the main one. */
export function seatParam(seat: ChatSeat | null): { conversationId?: ConversationId } {
  return seat && !seat.main ? { conversationId: seat.conversationId } : {};
}

/** The same, for the conversation of the chat around the caller. */
export function useSeatParam(): { conversationId?: ConversationId } {
  return seatParam(useContext(SeatContext));
}
