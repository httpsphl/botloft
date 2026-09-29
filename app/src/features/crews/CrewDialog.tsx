import { type FormEvent, useState } from "react";
import { useT } from "../../i18n";
import { errorText } from "../../lib/api";
import { type Crew, FIELD_LIMITS } from "../../lib/protocol.gen";
import { useApi, useApp } from "../../store/context";
import { Button } from "../../ui/Button";
import { Dialog } from "../../ui/Dialog";
import { TextField } from "../../ui/Field";
import { WorkFolderField } from "./WorkFolderField";

/** Creates a crew, or renames `crew`. */
export function CrewDialog({ crew, onClose }: { crew?: Crew; onClose(): void }) {
  const t = useT();
  const api = useApi();
  const putCrew = useApp((state) => state.putCrew);
  const selectCrew = useApp((state) => state.selectCrew);
  const [name, setName] = useState(crew?.name ?? "");
  const [folder, setFolder] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    setBusy(true);
    setError(null);
    try {
      const saved = crew
        ? await api.call("crews.rename", { crewId: crew.id, name })
        : await api.call("crews.create", { name, ...(folder && { workFolder: folder }) });
      putCrew(saved);
      if (!crew) {
        selectCrew(saved.id);
      }
      onClose();
    } catch (failure) {
      setError(errorText(failure));
      setBusy(false);
    }
  };

  const formId = "crew-form";
  return (
    <Dialog
      title={crew ? t.crews.dialog.renameTitle : t.crews.newCrew}
      onClose={onClose}
      footer={
        <>
          <Button onClick={onClose}>{t.common.cancel}</Button>
          <Button variant="primary" type="submit" form={formId} disabled={busy}>
            {crew ? t.crews.rename : t.crews.dialog.create}
          </Button>
        </>
      }
    >
      <form id={formId} onSubmit={submit} className="flex flex-col gap-3">
        <TextField
          label={t.crews.dialog.name}
          value={name}
          max={FIELD_LIMITS.name}
          onChange={(event) => setName(event.target.value)}
          placeholder={t.crews.dialog.namePlaceholder}
          hint={crew ? t.crews.dialog.renameHint(crew.slug) : t.crews.dialog.createHint}
          autoFocus
          required
        />
        {!crew && <WorkFolderField value={folder} onChange={setFolder} />}
        {error && (
          <p role="alert" className="text-danger text-sm">
            {error}
          </p>
        )}
      </form>
    </Dialog>
  );
}
