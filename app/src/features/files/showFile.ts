// How a tool line in the chat asks for its file to be shown in the files
// panel (spec 15.1). Absent where there is no panel, so no button shows.

import { createContext } from "react";

export const ShowFile = createContext<((path: string) => void) | null>(null);
