// The owner's Claude Code plugins and who uses each (spec 41), read when
// shown and kept up to date by what `plugins.share` and `plugins.everyone`
// answer.

import { useCallback, useEffect, useState } from "react";
import type { BotId, Plugin, PluginList } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";
import { attempt } from "../../ui/toast";

export function usePlugins(
  failed: string,
  /** When every plugin could not go to every agent, or come back. */
  allFailed: string,
): {
  plugins: Plugin[] | null;
  /** Every plugin for every agent on Claude Code. */
  everyone: boolean;
  share(name: string, botIds: BotId[]): Promise<unknown>;
  /** One plugin (with its name) or all of them for every agent, or not. */
  forEveryone(name: string | undefined, on: boolean): Promise<unknown>;
} {
  const api = useApi();
  const [list, setList] = useState<PluginList | null>(null);
  useEffect(() => {
    let alive = true;
    api.call("plugins.list").then(
      (listed) => alive && setList(listed),
      // An older daemon has no plugins: nothing to show.
      () => alive && setList({ plugins: [], everyone: false }),
    );
    return () => {
      alive = false;
    };
  }, [api]);
  const share = useCallback(
    (name: string, botIds: BotId[]) =>
      attempt(failed, async () => {
        const plugin = await api.call("plugins.share", { name, botIds });
        setList(
          (all) =>
            all && {
              ...all,
              plugins: all.plugins.map((have) => (have.name === plugin.name ? plugin : have)),
            },
        );
      }),
    [api, failed],
  );
  const forEveryone = useCallback(
    (name: string | undefined, on: boolean) =>
      attempt(name === undefined ? allFailed : failed, async () => {
        setList(await api.call("plugins.everyone", name === undefined ? { on } : { name, on }));
      }),
    [api, failed, allFailed],
  );
  return {
    plugins: list?.plugins ?? null,
    everyone: list?.everyone === true,
    share,
    forEveryone,
  };
}
