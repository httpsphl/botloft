import { type FormEvent, useState } from "react";
import { errorText } from "../../lib/api";
import { type Crew, FIELD_LIMITS } from "../../lib/protocol.gen";
import { useApi, useApp } from "../../store/context";
import { Button } from "../../ui/Button";
import { Dialog } from "../../ui/Dialog";
import { TextField } from "../../ui/Field";

/** Creates a crew, or renames `crew`. */
export function CrewDialog({ crew, onClose }: { crew?: Crew; onClose(): void }) {
  const api = useApi();
  const putCrew = useApp((state) => state.putCrew);
  const selectCrew = useApp((state) => state.selectCrew);
  const [name, setName] = useState(crew?.name ?? "");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    setBusy(true);
    setError(null);
    try {
      const saved = crew
        ? await api.call("crews.rename", { crewId: crew.id, name })
        : await api.call("crews.create", { name });
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
      title={crew ? "Rename crew" : "New crew"}
      onClose={onClose}
      footer={
        <>
          <Button onClick={onClose}>Cancel</Button>
          <Button variant="primary" type="submit" form={formId} disabled={busy}>
            {crew ? "Rename" : "Create crew"}
          </Button>
        </>
      }
    >
      <form id={formId} onSubmit={submit} className="flex flex-col gap-3">
        <TextField
          label="Name"
          value={name}
          max={FIELD_LIMITS.name}
          onChange={(event) => setName(event.target.value)}
          placeholder="Research"
          hint={
            crew
              ? `The folder keeps its name (${crew.slug}).`
              : "Bots in a crew can message each other and share a folder."
          }
          autoFocus
          required
        />
        {error && (
          <p role="alert" className="text-danger text-sm">
            {error}
          </p>
        )}
      </form>
    </Dialog>
  );
}
