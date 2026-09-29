// English, the reference: every other language has exactly this shape.

import { account } from "./account";
import { bots } from "./bots";
import { chat } from "./chat";
import { common } from "./common";
import { crews } from "./crews";
import { messages } from "./messages";
import { onboarding } from "./onboarding";
import { routines } from "./routines";
import { shell } from "./shell";
import { updates } from "./updates";

export const en = {
  common,
  shell,
  onboarding,
  updates,
  account,
  bots,
  chat,
  crews,
  messages,
  routines,
};

export type Messages = typeof en;
