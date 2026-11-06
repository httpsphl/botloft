// The owner's skills and who uses each (spec 38), read when shown and kept
// up to date by what `skills.share` answers.

import { useCallback, useEffect, useState } from "react";
import type { BotId, Skill } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";
import { attempt } from "../../ui/toast";

export function useSkills(failed: string): {
  skills: Skill[] | null;
  share(name: string, botIds: BotId[]): Promise<unknown>;
} {
  const api = useApi();
  const [skills, setSkills] = useState<Skill[] | null>(null);
  useEffect(() => {
    let alive = true;
    api.call("skills.list").then(
      (listed) => alive && setSkills(listed.skills),
      // An older daemon has no skills: nothing to show.
      () => alive && setSkills([]),
    );
    return () => {
      alive = false;
    };
  }, [api]);
  const share = useCallback(
    (name: string, botIds: BotId[]) =>
      attempt(failed, async () => {
        const skill = await api.call("skills.share", { name, botIds });
        setSkills((all) => all?.map((have) => (have.name === skill.name ? skill : have)) ?? null);
      }),
    [api, failed],
  );
  return { skills, share };
}
