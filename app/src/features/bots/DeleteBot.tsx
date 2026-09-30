import { useState } from "react";
import { useT } from "../../i18n";
import type { Bot } from "../../lib/protocol.gen";
import { useApi, useApp } from "../../store/context";
import { Confirm } from "../../ui/Confirm";
import { KeptFolder } from "../../ui/KeptFolder";
import { attempt } from "../../ui/toast";

/**
 * Asks before deleting a bot for good (spec 7.6), saying what goes and
 * what stays, with the choice to send its folder to the Recycle Bin.
 * `chiefOf` is the crew it leads, if it leads one.
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
  // The folder stays unless the owner says otherwise.
  const [recycle, setRecycle] = useState(false);
  return (
    <Confirm
      title={words.deleteConfirm.title(bot.name)}
      confirmLabel={words.deleteConfirm.confirm}
      onClose={onClose}
      onConfirm={() =>
        attempt(words.failed.delete, async () => {
          const asked = recycle ? { botId: bot.id, recycleFolder: true } : { botId: bot.id };
          dropBot((await api.call("bots.delete", asked)).botId);
          onDeleted?.();
        })
      }
    >
      <p>
        {words.deleteConfirm.removed(bot.name, bot.archivedAt === null)}
        {chiefOf !== undefined && ` ${words.deleteConfirm.chief(chiefOf)}`}
      </p>
      <KeptFolder
        path={bot.workspace}
        recycle={{ label: words.deleteConfirm.recycle, checked: recycle, onChange: setRecycle }}
      >
        {recycle ? words.deleteConfirm.recycled(bot.name) : words.deleteConfirm.kept(bot.name)}
      </KeptFolder>
    </Confirm>
  );
}
