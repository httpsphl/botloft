// English, the reference: every other language has exactly this shape.

import { account } from "./account";
import { bots } from "./bots";
import { browser } from "./browser";
import { chat } from "./chat";
import { common } from "./common";
import { crews } from "./crews";
import { files } from "./files";
import { messages } from "./messages";
import { onboarding } from "./onboarding";
import { routines } from "./routines";
import { screens } from "./screens";
import { shell } from "./shell";
import { updates } from "./updates";

export const en = {
  common,
  shell,
  onboarding,
  updates,
  account,
  bots,
  browser,
  chat,
  crews,
  files,
  messages,
  routines,
  screens,
};

export type Messages = typeof en;
