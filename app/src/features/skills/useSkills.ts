// The owner's skills and who uses each (spec 38), read when shown and kept
// up to date by what `skills.share` and `skills.everyone` answer.

import { useCallback, useEffect, useState } from "react";
import type { BotId, Skill, SkillList } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";
import { attempt } from "../../ui/toast";

export function useSkills(
  failed: string,
  /** When every skill could not go to every agent, or come back. */
  allFailed = failed,
): {
  skills: Skill[] | null;
  /** Every skill for every agent on Claude Code. */
  everyone: boolean;
  share(name: string, botIds: BotId[]): Promise<unknown>;
  /** One skill (with its name) or all of them for every agent, or not. */
  forEveryone(name: string | undefined, on: boolean): Promise<unknown>;
} {
  const api = useApi();
  const [list, setList] = useState<SkillList | null>(null);
  useEffect(() => {
    let alive = true;
    api.call("skills.list").then(
      (listed) => alive && setList(listed),
      // An older daemon has no skills: nothing to show.
      () => alive && setList({ skills: [], everyone: false }),
    );
    return () => {
      alive = false;
    };
  }, [api]);
  const share = useCallback(
    (name: string, botIds: BotId[]) =>
      attempt(failed, async () => {
        const skill = await api.call("skills.share", { name, botIds });
        setList(
          (all) =>
            all && {
              ...all,
              skills: all.skills.map((have) => (have.name === skill.name ? skill : have)),
            },
        );
      }),
    [api, failed],
  );
  const forEveryone = useCallback(
    (name: string | undefined, on: boolean) =>
      attempt(name === undefined ? allFailed : failed, async () => {
        setList(await api.call("skills.everyone", name === undefined ? { on } : { name, on }));
      }),
    [api, failed, allFailed],
  );
  return {
    skills: list?.skills ?? null,
    // An older daemon does not say.
    everyone: list?.everyone === true,
    share,
    forEveryone,
  };
}
