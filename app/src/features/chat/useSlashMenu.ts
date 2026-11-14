// The menu of "/" in the composer (spec 38.11): while the message is only
// "/" and part of a name, it lists the skills and plugin commands the agent
// can run, and the arrows, Enter and Tab pick one. Esc closes it until the
// text changes. Only agents on Claude Code have it.

import { type KeyboardEvent, useEffect, useId, useState } from "react";
import type { Bot, SlashCommand } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";
import { matching, slashQuery } from "./slash";

export interface SlashMenuState {
  open: boolean;
  query: string;
  /** Null while they are read. */
  commands: SlashCommand[] | null;
  matches: SlashCommand[];
  active: number;
  listId: string;
  setActive(index: number): void;
  choose(command: SlashCommand): void;
}

export function useSlashMenu(bot: Bot, text: string, setText: (text: string) => void) {
  const api = useApi();
  const listId = useId();
  const [commands, setCommands] = useState<SlashCommand[] | null>(null);
  /** The highlighted row, for the query it was chosen on. */
  const [pick, setPick] = useState({ query: "", index: 0 });
  const [closedAt, setClosedAt] = useState<string | null>(null);
  const query = bot.agent === "claude" ? slashQuery(text) : null;
  const open = query !== null && closedAt !== text;

  // Read each time it opens: skills may have been given since.
  useEffect(() => {
    if (!open) {
      return;
    }
    let alive = true;
    setCommands(null);
    api.call("bots.commands", { botId: bot.id }).then(
      (listed) => alive && setCommands(listed.commands),
      // An older daemon has no menu: nothing to list.
      () => alive && setCommands([]),
    );
    return () => {
      alive = false;
    };
  }, [open, api, bot.id]);

  const matches = commands && query !== null ? matching(commands, query) : [];
  const active = pick.query === query ? Math.min(pick.index, Math.max(matches.length - 1, 0)) : 0;
  const setActive = (index: number) => setPick({ query: query ?? "", index });
  const choose = (command: SlashCommand) => setText(`/${command.name} `);

  /** True when the key was the menu's. */
  const onKeyDown = (event: KeyboardEvent<HTMLTextAreaElement>): boolean => {
    if (!open) {
      return false;
    }
    if (event.key === "Escape") {
      event.preventDefault();
      setClosedAt(text);
      return true;
    }
    if (matches.length === 0 || event.nativeEvent.isComposing) {
      return false;
    }
    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      event.preventDefault();
      const step = event.key === "ArrowDown" ? 1 : matches.length - 1;
      setActive((active + step) % matches.length);
      return true;
    }
    if ((event.key === "Enter" || event.key === "Tab") && !event.shiftKey) {
      event.preventDefault();
      const command = matches[active];
      if (command) {
        choose(command);
      }
      return true;
    }
    return false;
  };

  // The field stays a text box; the list it controls is read out through
  // the highlighted row.
  const inputProps =
    bot.agent === "claude"
      ? {
          "aria-controls": open ? listId : undefined,
          "aria-autocomplete": "list" as const,
          "aria-activedescendant": open && matches.length > 0 ? `${listId}-${active}` : undefined,
        }
      : {};

  const menu: SlashMenuState = {
    open,
    query: query ?? "",
    commands,
    matches,
    active,
    listId,
    setActive,
    choose,
  };
  return { menu, onKeyDown, inputProps };
}
