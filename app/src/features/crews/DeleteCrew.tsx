import { useState } from "react";
import { useT } from "../../i18n";
import type { Crew } from "../../lib/protocol.gen";
import { useApi, useApp } from "../../store/context";
import { Confirm } from "../../ui/Confirm";
import { KeptFolder } from "../../ui/KeptFolder";
import { attempt } from "../../ui/toast";

/**
 * Asks before deleting a crew for good (spec 7.6), saying what goes and
 * what stays, with the choice to send its folders to the Recycle Bin.
 * `bots` is how many bots go with it.
 */
export function DeleteCrew({
  crew,
  bots,
  onClose,
  onDeleted,
}: {
  crew: Crew;
  bots: number;
  onClose(): void;
  onDeleted?(): void;
}) {
  const words = useT().crews.view;
  const api = useApi();
  const dropCrew = useApp((state) => state.dropCrew);
  // The folders stay unless the owner says otherwise.
  const [recycle, setRecycle] = useState(false);
  // What goes to the bin is the crew's own folder, where `shared` is
  // (spec 5); a work folder the owner chose is somewhere else and stays.
  const shown =
    recycle && !crew.workFolderChosen
      ? crew.workFolder.replace(/[\\/]shared$/, "")
      : crew.workFolder;
  const said = !recycle
    ? words.deleteKept
    : crew.workFolderChosen
      ? words.deleteRecycledChosen
      : words.deleteRecycled;
  return (
    <Confirm
      title={words.deleteTitle(crew.name)}
      confirmLabel={words.delete}
      onClose={onClose}
      onConfirm={() =>
        attempt(words.failed.delete, async () => {
          const asked = recycle ? { crewId: crew.id, recycleFolder: true } : { crewId: crew.id };
          dropCrew((await api.call("crews.delete", asked)).crewId);
          onDeleted?.();
        })
      }
    >
      <p>{words.deleteBody(crew.name, bots, crew.archivedAt === null)}</p>
      <KeptFolder
        path={shown}
        recycle={{ label: words.deleteRecycle, checked: recycle, onChange: setRecycle }}
      >
        {said}
      </KeptFolder>
    </Confirm>
  );
}
