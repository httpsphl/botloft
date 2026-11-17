// The list the menu of "/" shows over the composer (spec 38.11): each
// skill or command by name, what it does and where it comes from.

import { useEffect } from "react";
import { useT } from "../../i18n";
import type { Bot } from "../../lib/protocol.gen";
import { POPOVER } from "../../ui/surface";
import type { SlashMenuState } from "./useSlashMenu";

export function SlashMenu({ bot, menu }: { bot: Bot; menu: SlashMenuState }) {
  const words = useT().chat.slash;
  const { open, listId, active } = menu;

  // The highlighted row stays in sight while the arrows move it.
  useEffect(() => {
    if (open) {
      document.getElementById(`${listId}-${active}`)?.scrollIntoView?.({ block: "nearest" });
    }
  }, [open, listId, active]);

  if (!open) {
    return null;
  }
  const note = (text: string) => <p className="px-2.5 py-2 text-muted text-sm">{text}</p>;
  return (
    <div className={`absolute right-0 bottom-full left-0 mb-2 ${POPOVER}`}>
      {menu.commands === null ? (
        note(words.looking)
      ) : menu.commands.length === 0 ? (
        note(words.none(bot.name))
      ) : menu.matches.length === 0 ? (
        note(words.noMatch(menu.query))
      ) : (
        <div
          id={listId}
          role="listbox"
          aria-label={words.label}
          className="max-h-72 overflow-y-auto"
        >
          {menu.matches.map((command, index) => (
            <div
              key={command.name}
              id={`${listId}-${index}`}
              role="option"
              aria-selected={index === active}
              tabIndex={-1}
              onMouseDown={(event) => event.preventDefault()}
              onMouseMove={() => index !== active && menu.setActive(index)}
              onClick={() => menu.choose(command)}
              onKeyDown={(event) => {
                if (event.key === "Enter" || event.key === " ") {
                  event.preventDefault();
                  menu.choose(command);
                }
              }}
              className={`flex cursor-pointer items-baseline gap-2 rounded-lg px-2.5 py-1.5 ${
                index === active ? "bg-sunken" : ""
              }`}
            >
              <span className="shrink-0 font-medium text-sm">/{command.name}</span>
              <span className="min-w-0 flex-1 truncate text-muted text-xs">
                {command.description}
              </span>
              <span className="shrink-0 text-muted text-xs">{words.from[command.from]}</span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
