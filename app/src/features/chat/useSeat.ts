// The conversation an agent's chat is on (spec 40.7), and the agent as
// that conversation sees it: its state, context and background tasks are
// the conversation's, while the agent's own are the summary of all.

import { useEffect, useMemo } from "react";
import { mainConversationOf } from "../../lib/conversations";
import type { Bot } from "../../lib/protocol.gen";
import { useApp } from "../../store/context";
import { conversationOf, openConversationOf } from "../../store/conversations";
import type { ChatSeat } from "./seat";

export function useSeat(agent: Bot): { bot: Bot; seat: ChatSeat } {
  const open = useApp((state) => state.connection.kind === "open");
  const load = useApp((state) => state.loadConversations);
  const conversationId = useApp((state) => openConversationOf(state, agent.id));
  const conversation = useApp((state) => conversationOf(state, agent.id));
  useEffect(() => {
    if (open) {
      load(agent.id).catch(() => {});
    }
  }, [open, load, agent.id]);
  const main = conversationId === mainConversationOf(agent.id);
  const seat = useMemo(() => ({ conversationId, main }), [conversationId, main]);
  const bot = useMemo(() => {
    // An archived or paused agent shows as such in every conversation.
    if (!conversation || agent.state === "archived" || conversation.state === "archived") {
      return agent;
    }
    return {
      ...agent,
      state: conversation.state,
      generation: conversation.generation,
      context: conversation.context,
      background: conversation.background,
    };
  }, [agent, conversation]);
  return { bot, seat };
}
