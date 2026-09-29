// English, the reference: every other language has exactly this shape.

import { account } from "./account";
import { alerts } from "./alerts";
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
import { tools } from "./tools";
import { updates } from "./updates";

export const en = {
  common,
  shell,
  onboarding,
  updates,
  account,
  alerts,
  bots,
  browser,
  chat,
  crews,
  files,
  messages,
  routines,
  screens,
  tools,
};

export type Messages = typeof en;
