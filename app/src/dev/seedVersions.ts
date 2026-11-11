// Dev only: a history of Scout's memory and instructions for the preview
// (spec 8.10): how it started, what Scout wrote, a change from outside and
// the owner's own edit.

import type { FakeBotloft } from "../lib/fake";
import type { BotId } from "../lib/protocol.gen";

const HOUR = 60 * 60_000;

export function seedVersions(fake: FakeBotloft, scout: BotId): void {
  const day = fake.now - 26 * HOUR;
  fake.versions.add(scout, "memory", "start", "# Notes\n\n- Sources: blogs and papers.\n", day);
  fake.versions.add(
    scout,
    "memory",
    "agent",
    "# Notes\n\n- Sources: blogs and papers.\n- The owner wants three sources per claim.\n",
    day + 3 * HOUR,
  );
  fake.versions.add(
    scout,
    "memory",
    "outside",
    "# Notes\n\n- Sources: papers first, then blogs.\n- The owner wants three sources per claim.\n",
    day + 20 * HOUR,
  );
  fake.versions.add(scout, "instructions", "start", "Find sources and summarize them.", day);
  fake.versions.add(
    scout,
    "instructions",
    "owner",
    "Find sources and summarize them. Link every source.",
    day + 22 * HOUR,
  );
}
