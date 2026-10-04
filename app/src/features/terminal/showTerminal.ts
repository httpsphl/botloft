// How the chat asks for the bot's terminal (spec 15.1). Absent where there
// is no panel, so no button shows.

import { createContext } from "react";

export const ShowTerminal = createContext<(() => void) | null>(null);
