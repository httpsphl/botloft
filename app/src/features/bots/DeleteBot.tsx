import { useT } from "../../i18n";
import type { Bot } from "../../lib/protocol.gen";
import { useApi, useApp } from "../../store/context";
import { Confirm } from "../../ui/Confirm";
import { KeptFolder } from "../../ui/KeptFolder";
import { attempt } from "../../ui/toast";

/**
 * Asks before deleting a bot for good (spec 7.6), saying what goes and
 * what stays. `chiefOf` is the crew it leads, if it leads one.
 */
export function DeleteBot({
  bot,
  chiefOf,
  onClose,
  onDeleted,
}: {
  bot: Bot;
  chiefOf?: string | undefined;
  onClose(): void;
  onDeleted?(): void;
}) {
  const words = useT().bots.header;
  const api = useApi();
  const dropBot = useApp((state) => state.dropBot);
  return (
    <Confirm
      title={words.deleteConfirm.title(bot.name)}
      confirmLabel={words.deleteConfirm.confirm}
      onClose={onClose}
      onConfirm={() =>
        attempt(words.failed.delete, async () => {
          dropBot((await api.call("bots.delete", { botId: bot.id })).botId);
          onDeleted?.();
        })
      }
    >
      <p>
        {words.deleteConfirm.removed(bot.name, bot.archivedAt === null)}
        {chiefOf !== undefined && ` ${words.deleteConfirm.chief(chiefOf)}`}
      </p>
      <KeptFolder path={bot.workspace}>{words.deleteConfirm.kept(bot.name)}</KeptFolder>
    </Confirm>
  );
}
