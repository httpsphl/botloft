// Settings, "Tools" (spec 34.7): the tools of the owner's claude.ai account
// and the ones the owner pasted, as cards with who uses each, and the
// tools the owner could connect on claude.ai. A card opens who may use it.

import { Check, Plus, RefreshCw, Search } from "lucide-react";
import { useState } from "react";
import { useT } from "../../i18n";
import { clock } from "../../lib/format";
import type { McpServer } from "../../lib/protocol.gen";
import { useApp, useHost } from "../../store/context";
import { Button } from "../../ui/Button";
import { Menu } from "../../ui/Menu";
import { attempt } from "../../ui/toast";
import { AddToolDialog } from "./AddToolDialog";
import { CONNECTORS_PAGE, useConnectorList } from "./connectorList";
import { type Opened, ToolDialog } from "./ToolDialog";
import { ToolTile } from "./ToolTile";
import { KNOWN_TOOLS, knownTool, TOOL_KINDS } from "./toolCatalog";
import { useConnections } from "./useConnections";

type Tab = "yours" | "discover";

const CARD =
  "flex w-full min-w-0 gap-3 rounded-2xl bg-bubble p-4 text-left transition-colors hover:bg-line/60";

export function ToolsPage() {
  const t = useT().connections;
  const p = t.page;
  const host = useHost();
  const bots = useApp((state) => state.bots);
  const overview = useConnections();
  const { list, looking, refresh } = useConnectorList({ fresh: true });
  const [tab, setTab] = useState<Tab>("yours");
  const [query, setQuery] = useState("");
  const [opened, setOpened] = useState<Opened | null>(null);
  const [adding, setAdding] = useState(false);

  const wanted = query.trim().toLowerCase();
  const matches = (name: string) => wanted === "" || name.toLowerCase().includes(wanted);
  const about = (name: string) => {
    const known = knownTool(name);
    return known ? t.about[known.key] : null;
  };
  const account = (list?.connectors ?? []).filter((connector) => matches(connector.name));
  const own = (overview?.servers ?? []).filter((server) => matches(server.name));
  const users = (name: string) =>
    Object.values(bots).filter((bot) => bot.connectors.includes(name));
  const ownUsers = (server: McpServer) =>
    overview?.bots.filter((link) => link.serverIds.includes(server.id)).length ?? 0;
  const had = new Set((list?.connectors ?? []).map((connector) => knownTool(connector.name)?.key));
  const discover = KNOWN_TOOLS.filter((tool) => !had.has(tool.key) && matches(tool.name));
  const openClaudeAi = () => attempt(t.claudeAi.openFailed, () => host.openUrl(CONNECTORS_PAGE));

  return (
    <div className="flex flex-col gap-5">
      <div className="flex flex-wrap items-center gap-3">
        <div role="tablist" aria-label={t.title} className="flex gap-1 rounded-full bg-bubble p-1">
          {(["yours", "discover"] as const).map((each) => (
            <button
              key={each}
              type="button"
              role="tab"
              aria-selected={tab === each}
              onClick={() => setTab(each)}
              className={`h-8 rounded-full px-4 font-medium text-sm transition-colors ${tab === each ? "bg-ink text-canvas" : "text-ink-soft hover:text-ink"}`}
            >
              {p[each]}
            </button>
          ))}
        </div>
        <label className="relative min-w-48 flex-1">
          <span className="sr-only">{p.search}</span>
          <Search
            aria-hidden
            size={15}
            className="pointer-events-none absolute top-1/2 left-3.5 -translate-y-1/2 text-muted"
          />
          <input
            type="search"
            value={query}
            placeholder={p.search}
            onChange={(event) => setQuery(event.target.value)}
            className="h-9 w-full rounded-full border border-transparent bg-sunken pr-4 pl-9 text-ink text-sm outline-none transition-colors placeholder:text-muted focus:border-line-strong focus:bg-panel"
          />
        </label>
        <Menu
          label={p.add}
          text={p.add}
          icon={Plus}
          items={[
            { label: p.addClaudeAi, onSelect: () => void openClaudeAi() },
            { label: p.addPaste, onSelect: () => setAdding(true) },
          ]}
        />
      </div>

      {tab === "yours" ? (
        <>
          <div className="flex flex-wrap items-center gap-2 text-muted text-xs">
            {list && list.checkedAt > 0 && <span>{p.checked(clock(list.checkedAt))}</span>}
            <Button
              variant="ghost"
              size="sm"
              icon={RefreshCw}
              disabled={looking}
              aria-busy={looking}
              onClick={() => void refresh()}
            >
              {t.claudeAi.refresh}
            </Button>
          </div>
          {list === null ? (
            <p className="text-muted text-sm">{t.claudeAi.looking}</p>
          ) : account.length + own.length === 0 ? (
            <p className="text-muted text-sm">
              {wanted ? p.nothingFound(query.trim()) : p.noneYet}
            </p>
          ) : (
            <ul
              aria-label={p.yours}
              className="grid grid-cols-[repeat(auto-fill,minmax(17rem,1fr))] gap-3"
            >
              {account.map((connector) => {
                const count = users(connector.name).length;
                return (
                  <li key={connector.name} className="flex">
                    <button
                      type="button"
                      className={CARD}
                      onClick={() => setOpened({ kind: "claude", connector })}
                    >
                      <ToolTile name={connector.name} />
                      <span className="flex min-w-0 flex-1 flex-col gap-0.5">
                        <span className="flex flex-wrap items-baseline gap-x-2">
                          <span className="break-words font-semibold">{connector.name}</span>
                          <span className="text-muted text-xs">{p.fromClaudeAi}</span>
                        </span>
                        {about(connector.name) && (
                          <span className="line-clamp-2 text-ink-soft text-sm">
                            {about(connector.name)}
                          </span>
                        )}
                        <span className="mt-1 flex flex-wrap items-center gap-x-2 text-xs">
                          {connector.state === "connected" ? (
                            <span className="inline-flex items-center gap-1 text-ok">
                              <Check aria-hidden size={13} />
                              {p.connected}
                            </span>
                          ) : (
                            <span className="text-warn">{t.states[connector.state]}</span>
                          )}
                          <span className="text-muted">
                            {count > 0 ? p.agents(count) : p.noAgent}
                          </span>
                        </span>
                      </span>
                    </button>
                  </li>
                );
              })}
              {own.map((server) => {
                const count = ownUsers(server);
                return (
                  <li key={server.id} className="flex">
                    <button
                      type="button"
                      className={CARD}
                      onClick={() => setOpened({ kind: "own", server })}
                    >
                      <ToolTile name={server.name} />
                      <span className="flex min-w-0 flex-1 flex-col gap-0.5">
                        <span className="flex flex-wrap items-baseline gap-x-2">
                          <span className="break-words font-semibold">{server.name}</span>
                          <span className="text-muted text-xs">{p.yourOwn}</span>
                        </span>
                        {server.description && (
                          <span className="line-clamp-2 text-ink-soft text-sm">
                            {server.description}
                          </span>
                        )}
                        <span className="mt-1 text-muted text-xs">
                          {count > 0 ? p.agents(count) : p.noAgent}
                        </span>
                      </span>
                    </button>
                  </li>
                );
              })}
            </ul>
          )}
        </>
      ) : (
        <>
          <p className="max-w-[65ch] text-ink-soft text-sm leading-relaxed">{p.discoverIntro}</p>
          {discover.length === 0 ? (
            <p className="text-muted text-sm">{p.nothingFound(query.trim())}</p>
          ) : (
            TOOL_KINDS.map((kind) => {
              const tools = discover.filter((tool) => tool.kind === kind);
              if (tools.length === 0) {
                return null;
              }
              return (
                <section key={kind} className="flex flex-col gap-2.5">
                  <h3 className="font-semibold text-sm">{t.toolKinds[kind]}</h3>
                  <ul
                    aria-label={t.toolKinds[kind]}
                    className="grid grid-cols-[repeat(auto-fill,minmax(17rem,1fr))] gap-3"
                  >
                    {tools.map((tool) => (
                      <li key={tool.key} className="flex">
                        <button
                          type="button"
                          className={CARD}
                          aria-label={p.connect(tool.name)}
                          onClick={() => void openClaudeAi()}
                        >
                          <ToolTile name={tool.name} />
                          <span className="flex min-w-0 flex-1 flex-col gap-0.5">
                            <span className="break-words font-semibold">{tool.name}</span>
                            <span className="line-clamp-2 text-ink-soft text-sm">
                              {t.about[tool.key]}
                            </span>
                          </span>
                          <span
                            aria-hidden
                            className="grid size-8 shrink-0 place-items-center self-start rounded-full bg-panel text-ink-soft"
                          >
                            <Plus size={16} />
                          </span>
                        </button>
                      </li>
                    ))}
                  </ul>
                </section>
              );
            })
          )}
          <Button className="self-start" onClick={() => void openClaudeAi()}>
            {p.more}
          </Button>
        </>
      )}

      {opened && <ToolDialog opened={opened} onClose={() => setOpened(null)} />}
      {adding && <AddToolDialog onClose={() => setAdding(false)} />}
    </div>
  );
}
