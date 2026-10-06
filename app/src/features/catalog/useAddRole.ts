import { useState } from "react";
import { useShallow } from "zustand/react/shallow";
import { useT } from "../../i18n";
import type { Bot, BotTemplate, Crew } from "../../lib/protocol.gen";
import { botsOf } from "../../store/app";
import { useApi, useApp } from "../../store/context";
import { attempt } from "../../ui/toast";
import { roleText } from "./roleText";

/** `name`, or `name 2`, `name 3`... when the crew already has a bot called so. */
export function freeName(name: string, bots: Bot[]): string {
  const taken = new Set(bots.map((bot) => bot.name.trim().toLowerCase()));
  let candidate = name;
  for (let number = 2; taken.has(candidate.toLowerCase()); number += 1) {
    candidate = `${name} ${number}`;
  }
  return candidate;
}

/**
 * Adds a role to `crew` as a bot (`catalog.add`), with the name and role in
 * the owner's language. `added` is the last bot each role made while the
 * screen was open, so its card can say so.
 */
export function useAddRole(crew: Crew) {
  const t = useT();
  const api = useApi();
  const putBot = useApp((state) => state.putBot);
  const bots = useApp(useShallow((state) => botsOf(state, crew.id)));
  const [added, setAdded] = useState<Record<string, Bot>>({});
  const [busy, setBusy] = useState<string | null>(null);

  const add = async (template: BotTemplate) => {
    const text = roleText(t, template);
    setBusy(template.id);
    await attempt(t.catalog.failed.add, async () => {
      const bot = await api.call("catalog.add", {
        crewId: crew.id,
        templateId: template.id,
        name: freeName(text.name, bots),
        role: text.role,
      });
      putBot(bot);
      setAdded((prev) => ({ ...prev, [template.id]: bot }));
    });
    setBusy(null);
  };

  return { add, added, busy };
}
