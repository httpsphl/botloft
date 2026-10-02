// How the chat asks for a file to be shown in the files panel (spec 15.1).
// Absent where there is no panel, so no button shows. A file the chat
// already describes comes along, for one the panel's list leaves out.

import { createContext } from "react";
import type { BotFile } from "../../lib/protocol.gen";

export const ShowFile = createContext<((path: string, file?: BotFile) => void) | null>(null);
