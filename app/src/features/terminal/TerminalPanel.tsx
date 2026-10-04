// The bot's terminal beside its chat (spec 15.1): the commands it ran, as
// a terminal window on the bot's desk, live, each with what the bot said it
// is for, what it printed and whether it failed. Read from the chat: the
// commands are its `Bash` and `PowerShell` calls.

import { LoaderCircle, X } from "lucide-react";
import { useEffect, useRef } from "react";
import { useT } from "../../i18n";
import type { Bot, ToolItem } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";
import { Button } from "../../ui/Button";
import { SidePanel } from "../../ui/SidePanel";
import { commandOf, isCommand } from "../chat/command";
import { useChat } from "../chat/useChat";
import { ComputerFooter } from "./ComputerDock";

function Command({ tool }: { tool: ToolItem }) {
  const t = useT().terminal;
  const command = commandOf(tool.input);
  return (
    <li className="terminal-entry">
      {tool.explanation && <p className="terminal-comment"># {tool.explanation}</p>}
      <p className="whitespace-pre-wrap break-all">
        <span aria-hidden className="terminal-prompt">
          ❯{" "}
        </span>
        {command.text}
      </p>
      {tool.status === "running" ? (
        <p className="terminal-comment flex items-center gap-1.5">
          <LoaderCircle aria-hidden size={11} className="animate-spin" />
          {t.running}
        </p>
      ) : (
        tool.output && (
          <pre
            className={`whitespace-pre-wrap break-all ${
              tool.status === "failed" ? "terminal-failed" : "terminal-output"
            }`}
          >
            {tool.output}
          </pre>
        )
      )}
    </li>
  );
}

/**
 * The commands each terminal showed last, so coming back to it (from the
 * dock) shows them at once while the chat loads again.
 */
const lastShown = new WeakMap<object, Map<string, { id: string; tool: ToolItem }[]>>();

export function TerminalPanel({ bot, onClose }: { bot: Bot; onClose(): void }) {
  const t = useT().terminal;
  const chat = useChat(bot.id);
  const api = useApi();
  const shown = lastShown.get(api) ?? new Map<string, { id: string; tool: ToolItem }[]>();
  lastShown.set(api, shown);
  const loaded = chat.items.flatMap((item) =>
    item.body.kind === "tool" && isCommand(item.body.name)
      ? [{ id: item.id, tool: item.body }]
      : [],
  );
  const commands = chat.loading && loaded.length === 0 ? (shown.get(bot.id) ?? []) : loaded;
  if (!chat.loading) {
    shown.set(bot.id, loaded);
  }
  const screen = useRef<HTMLDivElement>(null);
  const last = commands.at(-1);
  // Follows the newest command, as a terminal does.
  // biome-ignore lint/correctness/useExhaustiveDependencies: a new or finished command scrolls
  useEffect(() => {
    const element = screen.current;
    if (element) {
      element.scrollTop = element.scrollHeight;
    }
  }, [last?.id, last?.tool.status]);

  return (
    <SidePanel label={t.panel(bot.name)} name="computer" defaultWidth={600}>
      <header className="flex h-11 shrink-0 items-center justify-between border-line border-b pr-1.5 pl-4">
        <h2 className="font-semibold text-sm">{t.heading}</h2>
        <Button variant="ghost" size="sm" icon={X} label={t.close} onClick={onClose} />
      </header>
      <div className="flex min-h-0 flex-1 flex-col p-3">
        <div
          className="bot-stage flex min-h-0 flex-1 flex-col p-2.5"
          style={{ "--bot": bot.color } as React.CSSProperties}
        >
          <div className="terminal-window flex min-h-0 flex-1 flex-col overflow-hidden rounded-xl">
            <div className="terminal-bar flex h-7 shrink-0 items-center gap-1.5 px-3">
              {[0, 1, 2].map((dot) => (
                <span key={dot} aria-hidden className="terminal-light" />
              ))}
              <span className="ml-2 truncate text-[11px]">{t.title(bot.name)}</span>
            </div>
            <div
              ref={screen}
              role="log"
              aria-label={t.panel(bot.name)}
              className="min-h-0 flex-1 overflow-y-auto px-3 py-2.5 font-mono text-xs leading-relaxed"
              data-selectable
            >
              {chat.loading && commands.length === 0 ? (
                <p className="terminal-comment">{t.loading}</p>
              ) : commands.length === 0 ? (
                <p className="terminal-comment">{t.empty(bot.name)}</p>
              ) : (
                <ol className="flex flex-col gap-3">
                  {commands.map(({ id, tool }) => (
                    <Command key={id} tool={tool} />
                  ))}
                </ol>
              )}
            </div>
          </div>
        </div>
      </div>
      <ComputerFooter />
    </SidePanel>
  );
}
