// The details beside an agent's chat (spec 15.1): who it is (mascot, name,
// role) with Edit, its instructions, what runs it and where it works, what
// it may do without asking, and the ways to what Settings keeps for it.

import { FolderOpen, Pencil, X } from "lucide-react";
import { type ReactNode, useState } from "react";
import { useT } from "../../i18n";
import type { Bot } from "../../lib/protocol.gen";
import { useHost } from "../../store/context";
import { Button } from "../../ui/Button";
import { SidePanel } from "../../ui/SidePanel";
import { attempt } from "../../ui/toast";
import { BotPermissions } from "../permissions/BotPermissions";
import { BotAvatar } from "./BotAvatar";
import { BotDialog } from "./BotDialog";
import { BotShortcuts } from "./BotShortcuts";

/** Lines of instructions shown before "Show all". */
const FOLDED_LINES = 6;

function Fact({ label, children }: { label: string; children: ReactNode }) {
  return (
    <div className="flex min-w-0 items-center gap-3 px-3 py-2">
      <dt className="w-20 shrink-0 text-muted text-xs">{label}</dt>
      <dd className="flex min-w-0 flex-1 items-center gap-1 text-sm">{children}</dd>
    </div>
  );
}

function Instructions({ bot, onEdit }: { bot: Bot; onEdit(): void }) {
  const words = useT().bots.details;
  const [open, setOpen] = useState(false);
  const long = bot.instructions.split("\n").length > FOLDED_LINES || bot.instructions.length > 420;
  return (
    <section aria-label={words.instructions} className="flex flex-col gap-2">
      <h3 className="font-semibold text-sm">{words.instructions}</h3>
      {bot.instructions ? (
        <div className="rounded-xl bg-bubble px-3 py-2.5">
          <p
            className={`whitespace-pre-wrap text-ink-soft text-sm leading-relaxed ${
              long && !open ? "line-clamp-6" : ""
            }`}
            data-selectable
          >
            {bot.instructions}
          </p>
          {long && (
            <button
              type="button"
              onClick={() => setOpen(!open)}
              className="mt-1 font-medium text-ink-soft text-xs hover:text-ink"
            >
              {open ? words.showLess : words.showAll}
            </button>
          )}
        </div>
      ) : (
        <div className="flex flex-col items-start gap-2 rounded-xl bg-bubble px-3 py-2.5">
          <p className="text-ink-soft text-sm">{words.noInstructions}</p>
          <Button size="sm" icon={Pencil} onClick={onEdit}>
            {words.write}
          </Button>
        </div>
      )}
    </section>
  );
}

export function Details({ bot, onClose }: { bot: Bot; onClose(): void }) {
  const t = useT();
  const words = t.bots.details;
  const host = useHost();
  const [editing, setEditing] = useState(false);
  const model =
    bot.agent === "claude"
      ? bot.model === "default" && bot.modelInUse
        ? `${t.chat.model.names.default} (${bot.modelInUse})`
        : t.chat.model.names[bot.model]
      : null;
  return (
    <SidePanel label={words.title(bot.name)} name="details" defaultWidth={340}>
      <header className="flex h-11 shrink-0 items-center justify-between pr-1.5 pl-4">
        <h2 className="font-semibold text-base">{words.heading}</h2>
        <div className="flex items-center gap-0.5">
          <Button
            variant="ghost"
            size="sm"
            icon={Pencil}
            label={words.edit(bot.name)}
            onClick={() => setEditing(true)}
          />
          <Button variant="ghost" size="sm" icon={X} label={words.close} onClick={onClose} />
        </div>
      </header>
      <div className="flex min-h-0 flex-col gap-5 overflow-y-auto px-4 pt-1 pb-4">
        <div className="flex items-center gap-3">
          <BotAvatar color={bot.color} size={44} still botId={bot.id} />
          <div className="min-w-0 flex-1">
            <p className="truncate font-semibold text-base">{bot.name}</p>
            <p className="truncate text-muted text-xs" data-selectable>
              @{bot.handle}
            </p>
            <p className="mt-0.5 text-ink-soft text-sm">{bot.role || words.noRole}</p>
          </div>
        </div>
        <Instructions bot={bot} onEdit={() => setEditing(true)} />
        <dl aria-label={words.about} className="m-0 flex flex-col rounded-xl bg-bubble py-1">
          <Fact label={words.engine}>
            <span className="truncate">{t.bots.dialog.agent.names[bot.agent]}</span>
          </Fact>
          {model && (
            <Fact label={words.model}>
              <span className="truncate">{model}</span>
            </Fact>
          )}
          <Fact label={words.folder}>
            <span className="min-w-0 flex-1 truncate font-mono text-xs" title={bot.workspace}>
              {bot.workspace}
            </span>
            <Button
              variant="ghost"
              size="sm"
              icon={FolderOpen}
              label={t.bots.header.openFolder}
              onClick={() =>
                attempt(t.bots.header.failed.openFolder, () => host.openPath(bot.workspace))
              }
            />
          </Fact>
          <Fact label={words.process}>
            <span className="truncate text-ink-soft">
              {bot.generation === null ? words.notStarted : words.generation(bot.generation)}
            </span>
          </Fact>
        </dl>
        <BotPermissions bot={bot} />
        <BotShortcuts bot={bot} />
      </div>
      {editing && <BotDialog bot={bot} onClose={() => setEditing(false)} />}
    </SidePanel>
  );
}
