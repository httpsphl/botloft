// Where a new crew works (spec 5): a folder the owner picks with Windows'
// folder picker, or a new one inside Botloft. With Botloft on a server, a
// folder of the server's, written as its path there (spec 42.7).

import { Folder, X } from "lucide-react";
import { useT } from "../../i18n";
import { useHost } from "../../store/context";
import { Button } from "../../ui/Button";
import { TextField } from "../../ui/Field";
import { attempt } from "../../ui/toast";
import { useFoldersHere } from "../files/fileActions";

export function WorkFolderField({
  value,
  onChange,
}: {
  value: string | null;
  onChange(folder: string | null): void;
}) {
  const t = useT();
  const words = t.crews.dialog;
  const failed = t.crews.view.failed.changeFolder;
  const host = useHost();
  const here = useFoldersHere();

  const pick = () =>
    attempt(failed, async () => {
      const folder = await host.pickFolder(words.pickTitle, value ?? undefined);
      if (folder) {
        onChange(folder);
      }
    });

  if (!here) {
    return (
      <TextField
        label={t.server.folder}
        hint={t.server.folderHint}
        placeholder={words.folderDefault}
        value={value ?? ""}
        onChange={(event) => onChange(event.target.value.trim() === "" ? null : event.target.value)}
      />
    );
  }

  return (
    <fieldset className="flex flex-col gap-1">
      <legend className="mb-1 font-medium text-ink-soft text-sm">{words.folder}</legend>
      <div className="flex items-center gap-2">
        <p
          className="flex h-8 min-w-0 flex-1 items-center gap-2 rounded-lg border border-line-strong bg-canvas px-2.5 text-sm"
          title={value ?? undefined}
        >
          <Folder aria-hidden size={15} className="shrink-0 text-muted" />
          <span className={`truncate ${value ? "text-ink" : "text-muted"}`}>
            {value ?? words.folderDefault}
          </span>
        </p>
        {value && (
          <button
            type="button"
            aria-label={words.useDefault}
            title={words.useDefault}
            onClick={() => onChange(null)}
            className="grid h-8 w-8 shrink-0 place-items-center rounded-lg text-muted hover:bg-sunken hover:text-ink"
          >
            <X aria-hidden size={15} />
          </button>
        )}
        <Button onClick={pick}>{words.chooseFolder}</Button>
      </div>
      <p className="text-muted text-xs">{words.folderHint}</p>
    </fieldset>
  );
}
