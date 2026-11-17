// "From Claude Code" (spec 25.8): the tools the owner set up in their own
// Claude Code, to pick and copy here. Their keys come along without ever
// reaching the app, and a program still needs the owner's "I understand".

import { useEffect, useState } from "react";
import { useT } from "../../i18n";
import { errorText } from "../../lib/api";
import type { ClaudeMcpServer, ClaudeMcpServers } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";
import { Button } from "../../ui/Button";
import { Callout } from "../../ui/Callout";
import { Details } from "../../ui/Details";
import { Dialog } from "../../ui/Dialog";

const keyOf = (server: ClaudeMcpServer) => `${server.folder ?? ""}\u0000${server.name}`;

const whatItRuns = (server: ClaudeMcpServer) =>
  server.kind === "stdio" ? [server.command ?? "", ...server.args].join(" ") : (server.url ?? "");

export function ImportClaudeDialog({ onClose }: { onClose(): void }) {
  const all = useT();
  const t = all.connections.fromClaude;
  const words = all.connections.adding;
  const api = useApi();
  const [list, setList] = useState<ClaudeMcpServers | null>(null);
  const [picked, setPicked] = useState<Set<string>>(new Set());
  const [trusted, setTrusted] = useState(false);
  const [busy, setBusy] = useState(false);
  const [failed, setFailed] = useState<string | null>(null);

  useEffect(() => {
    let alive = true;
    api.call("mcp.fromClaude").then(
      (found) => alive && setList(found),
      (error) => alive && setFailed(errorText(error)),
    );
    return () => {
      alive = false;
    };
  }, [api]);

  const servers = list?.servers ?? [];
  const chosen = servers.filter((server) => picked.has(keyOf(server)));
  const runsPrograms = chosen.some((server) => server.kind === "stdio");

  const toggle = (server: ClaudeMcpServer, on: boolean) => {
    const next = new Set(picked);
    if (on) {
      next.add(keyOf(server));
    } else {
      next.delete(keyOf(server));
    }
    setPicked(next);
    setTrusted(false);
  };

  const bring = async () => {
    setBusy(true);
    setFailed(null);
    try {
      await api.call("mcp.importClaude", {
        picks: chosen.map((server) => ({ name: server.name, folder: server.folder })),
      });
      onClose();
    } catch (error) {
      setFailed(errorText(error));
      setBusy(false);
    }
  };

  const why = (server: ClaudeMcpServer): string | null => {
    if (server.saved) {
      return t.saved;
    }
    if (server.problem?.reason === "kind") {
      return t.kind(server.problem.kind);
    }
    if (server.problem?.reason === "field") {
      return t.field(server.problem.field);
    }
    return null;
  };

  return (
    <Dialog
      title={t.title}
      onClose={onClose}
      footer={
        <>
          <Button onClick={onClose}>{all.common.cancel}</Button>
          <Button
            variant="primary"
            disabled={chosen.length === 0 || (runsPrograms && !trusted) || busy}
            onClick={bring}
          >
            {busy ? t.bringing : t.bring(chosen.length)}
          </Button>
        </>
      }
    >
      <div className="flex flex-col gap-3">
        <p className="text-ink-soft text-sm leading-relaxed">{t.intro}</p>
        {list === null && !failed && <p className="text-muted text-sm">{t.looking}</p>}
        {list && !list.found && <p className="text-muted text-sm">{t.noFile}</p>}
        {list?.found && servers.length === 0 && <p className="text-muted text-sm">{t.none}</p>}
        {servers.length > 0 && (
          <ul className="flex flex-col gap-2" aria-label={t.list}>
            {servers.map((server) => {
              const blocked = why(server);
              const secrets = [...server.envNames, ...server.headerNames];
              return (
                <li
                  key={keyOf(server)}
                  className="rounded-lg border border-line bg-sunken px-3 py-2"
                >
                  <label className="flex items-start gap-2">
                    <input
                      type="checkbox"
                      className="mt-1"
                      disabled={blocked !== null}
                      checked={picked.has(keyOf(server))}
                      onChange={(event) => toggle(server, event.target.checked)}
                    />
                    <span className="flex min-w-0 flex-col gap-0.5">
                      <span className="font-medium text-sm">{server.name}</span>
                      <span className="text-muted text-xs">
                        {server.folder ? t.oneFolder(server.folder) : t.everyFolder}
                      </span>
                      {blocked ? (
                        <span className="text-ink-soft text-xs">{blocked}</span>
                      ) : (
                        <span className="text-ink-soft text-xs leading-relaxed">
                          {server.kind === "stdio"
                            ? words.program(server.name)
                            : words.address(server.name)}
                          {secrets.length > 0 && ` ${t.secrets(secrets.join(", "))}`}
                        </span>
                      )}
                    </span>
                  </label>
                  {server.kind && <Details label={words.command}>{whatItRuns(server)}</Details>}
                </li>
              );
            })}
          </ul>
        )}
        {runsPrograms && (
          <label className="flex items-center gap-2 text-sm">
            <input
              type="checkbox"
              checked={trusted}
              onChange={(event) => setTrusted(event.target.checked)}
            />
            {words.understood}
          </label>
        )}
        {chosen.length > 0 && <p className="text-muted text-xs">{t.after}</p>}
        {failed && (
          <Callout tone="danger" title={t.failed}>
            {failed}
          </Callout>
        )}
      </div>
    </Dialog>
  );
}
