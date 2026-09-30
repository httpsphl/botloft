import { useT } from "../../i18n";
import type { Crew } from "../../lib/protocol.gen";
import { useApi, useApp } from "../../store/context";
import { Confirm } from "../../ui/Confirm";
import { KeptFolder } from "../../ui/KeptFolder";
import { attempt } from "../../ui/toast";

/**
 * Asks before deleting a crew for good (spec 7.6), saying what goes and
 * what stays. `bots` is how many bots go with it.
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
  return (
    <Confirm
      title={words.deleteTitle(crew.name)}
      confirmLabel={words.delete}
      onClose={onClose}
      onConfirm={() =>
        attempt(words.failed.delete, async () => {
          dropCrew((await api.call("crews.delete", { crewId: crew.id })).crewId);
          onDeleted?.();
        })
      }
    >
      <p>{words.deleteBody(crew.name, bots, crew.archivedAt === null)}</p>
      <KeptFolder path={crew.workFolder}>{words.deleteKept}</KeptFolder>
    </Confirm>
  );
}
