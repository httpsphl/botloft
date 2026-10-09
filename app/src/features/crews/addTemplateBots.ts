import type { Messages } from "../../i18n";
import type { BotloftApi } from "../../lib/api";
import type { Bot, BotTemplate, CrewId } from "../../lib/protocol.gen";
import { roleText } from "../catalog/roleText";
import { freeName } from "../catalog/useAddRole";
import type { CrewTemplate } from "./crewTemplates";

/**
 * Adds the bots of `template` to a crew just created, one after the other,
 * each with its name and role in the owner's language (`catalog.add`, as the
 * Bot agency does). A bot that fails does not stop the others: the crew is
 * already there, and the owner can add what is missing from the Bot agency.
 */
export async function addTemplateBots(
  api: BotloftApi,
  t: Messages,
  crewId: CrewId,
  template: CrewTemplate,
  roles: readonly BotTemplate[],
): Promise<{ added: Bot[]; failed: number; error: unknown }> {
  const added: Bot[] = [];
  let failed = 0;
  let error: unknown = null;
  for (const id of template.roles) {
    const role = roles.find((candidate) => candidate.id === id);
    if (!role) {
      failed += 1;
      continue;
    }
    const text = roleText(t, role);
    try {
      added.push(
        await api.call("catalog.add", {
          crewId,
          templateId: role.id,
          name: freeName(text.name, added),
          role: text.role,
        }),
      );
    } catch (caught) {
      failed += 1;
      error ??= caught;
    }
  }
  return { added, failed, error };
}
