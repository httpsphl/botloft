import {
  Archive,
  FolderInput,
  FolderOpen,
  type LucideIcon,
  Pause,
  Pencil,
  Play,
  Plus,
  Sparkles,
  Trash2,
} from "lucide-react";
import { type ReactNode, useState } from "react";
import { useShallow } from "zustand/react/shallow";
import { useT } from "../../i18n";
import type { Crew } from "../../lib/protocol.gen";
import { botsOf } from "../../store/app";
import { useApi, useApp, useHost } from "../../store/context";
import { Confirm } from "../../ui/Confirm";
import type { MenuItem } from "../../ui/Menu";
import { attempt } from "../../ui/toast";
import { BotDialog } from "../bots/BotDialog";
import { BotAgencyDialog } from "../catalog/BotAgencyDialog";
import { CrewDialog } from "./CrewDialog";
import { DeleteCrew } from "./DeleteCrew";

type Open = "bot" | "agency" | "rename" | "archive" | "delete" | { move: string } | null;

export interface CrewActions {
  /** A new bot in the crew. */
  newBot: MenuItem;
  /** The Bot agency: ready-made bots to add to the crew (spec 26.5). */
  agency: MenuItem;
  /** Pause or resume, as the crew is now. */
  pause: MenuItem & { icon: LucideIcon };
  /** Opens the work folder in the system's file manager. */
  openFolder(): void;
  /** Rename, the folder, archive and delete. */
  items: MenuItem[];
  /** The dialog an action opened, `null` while none is. */
  dialogs: ReactNode | null;
}

/**
 * What the owner can do with a crew from a menu, and the dialogs those
 * actions open. The crew page and the right-click menu of the sidebar both
 * come from here, so they stay the same.
 */
export function useCrewActions(crew: Crew): CrewActions {
  const t = useT();
  const words = t.crews.view;
  const api = useApi();
  const host = useHost();
  const putCrew = useApp((state) => state.putCrew);
  const bots = useApp(useShallow((state) => botsOf(state, crew.id)));
  const [open, setOpen] = useState<Open>(null);
  const close = () => setOpen(null);

  const openFolder = () => attempt(words.failed.openFolder, () => host.openPath(crew.workFolder));
  const pickFolder = () =>
    attempt(words.failed.changeFolder, async () => {
      const folder = await host.pickFolder(t.crews.dialog.pickTitle, crew.workFolder);
      if (folder && folder !== crew.workFolder) {
        setOpen({ move: folder });
      }
    });
  const setPaused = (paused: boolean) =>
    attempt(paused ? words.failed.pause : words.failed.resume, async () =>
      putCrew(await api.call("crews.setPaused", { crewId: crew.id, paused })),
    );

  let dialogs: ReactNode | null = null;
  if (open === "bot") {
    dialogs = <BotDialog crewId={crew.id} onClose={close} />;
  } else if (open === "agency") {
    dialogs = <BotAgencyDialog crew={crew} onClose={close} />;
  } else if (open === "rename") {
    dialogs = <CrewDialog crew={crew} onClose={close} />;
  } else if (open === "archive") {
    dialogs = (
      <Confirm
        title={words.archiveTitle(crew.name)}
        confirmLabel={words.archive}
        onClose={close}
        onConfirm={() =>
          attempt(words.failed.archive, async () =>
            putCrew(await api.call("crews.archive", { crewId: crew.id })),
          )
        }
      >
        {words.archiveBody(bots.length)}
      </Confirm>
    );
  } else if (open === "delete") {
    dialogs = <DeleteCrew crew={crew} bots={bots.length} onClose={close} />;
  } else if (open !== null) {
    const { move } = open;
    dialogs = (
      <Confirm
        title={words.moveTitle(crew.name)}
        confirmLabel={words.move}
        onClose={close}
        onConfirm={() =>
          attempt(words.failed.changeFolder, async () =>
            putCrew(await api.call("crews.setWorkFolder", { crewId: crew.id, workFolder: move })),
          )
        }
      >
        {words.moveBody(move)}
      </Confirm>
    );
  }

  return {
    newBot: { label: t.crews.newBot, icon: Plus, onSelect: () => setOpen("bot") },
    agency: { label: t.catalog.open, icon: Sparkles, onSelect: () => setOpen("agency") },
    openFolder,
    pause: crew.paused
      ? { label: words.resume, icon: Play, onSelect: () => setPaused(false) }
      : { label: words.pause, icon: Pause, onSelect: () => setPaused(true) },
    items: [
      { label: t.crews.rename, icon: Pencil, onSelect: () => setOpen("rename") },
      { label: words.openFolder, icon: FolderOpen, onSelect: openFolder },
      { label: words.changeFolder, icon: FolderInput, onSelect: pickFolder },
      { label: words.archive, icon: Archive, danger: true, onSelect: () => setOpen("archive") },
      { label: words.delete, icon: Trash2, danger: true, onSelect: () => setOpen("delete") },
    ],
    dialogs,
  };
}
