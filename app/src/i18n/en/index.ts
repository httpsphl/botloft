// English, the reference: every other language has exactly this shape.

import { account } from "./account";
import { alerts } from "./alerts";
import { bots } from "./bots";
import { browser } from "./browser";
import { catalog } from "./catalog";
import { chat } from "./chat";
import { common } from "./common";
import { connections } from "./connections";
import { crews } from "./crews";
import { crewTemplates } from "./crewTemplates";
import { desktop } from "./desktop";
import { files } from "./files";
import { messages } from "./messages";
import { onboarding } from "./onboarding";
import { phone } from "./phone";
import { questions } from "./questions";
import { routines } from "./routines";
import { screens } from "./screens";
import { search } from "./search";
import { setup } from "./setup";
import { shell } from "./shell";
import { terminal } from "./terminal";
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
  catalog,
  chat,
  connections,
  crews,
  crewTemplates,
  desktop,
  files,
  messages,
  phone,
  questions,
  routines,
  screens,
  search,
  setup,
  terminal,
  tools,
};

export type Messages = typeof en;
