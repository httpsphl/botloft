// The fake daemon's instructions for every agent (spec 7.10): trimmed and
// held to the same limit as an agent's own, like the daemon does.

import type { FakeBotloft, Handlers } from "./fake";
import { invalid } from "./fakeRules";
import { FIELD_LIMITS } from "./protocol.gen";

export function instructionsHandlers(
  fake: FakeBotloft,
): Pick<Handlers, "instructions.get" | "instructions.set"> {
  return {
    "instructions.get": () => ({ text: fake.ownerInstructions }),
    "instructions.set": ({ text }) => {
      const trimmed = text.trim();
      if (trimmed.length > FIELD_LIMITS.instructions) {
        throw invalid(`instructions must be at most ${FIELD_LIMITS.instructions} characters`);
      }
      fake.ownerInstructions = trimmed;
      return { text: trimmed };
    },
  };
}
