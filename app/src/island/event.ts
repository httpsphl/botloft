// What the main window hears when the island asks to open an agent's chat
// (spec 31.3): the agent's id, or `""` to just bring the window forward.
export const OPEN_BOT_EVENT = "botloft://open-bot";

// What the island hears from the app (spec 31.6): the owner held the right
// Ctrl long enough, and the main window came to the front or left it.
export const SUMMON_EVENT = "botloft://summon";
export const FRONT_EVENT = "botloft://front";

// The island follows the main window's language (spec 31.3): the main
// window tells it, and it asks once when it starts.
export const LOCALE_EVENT = "botloft://locale";
export const LOCALE_ASK_EVENT = "botloft://locale-ask";
