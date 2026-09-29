import { type FormEvent, useState } from "react";
import { useT } from "../../i18n";
import { errorText } from "../../lib/api";
import { type BotModel, type Crew, FIELD_LIMITS } from "../../lib/protocol.gen";
import { useApi, useApp } from "../../store/context";
import { Button } from "../../ui/Button";
import { Dialog } from "../../ui/Dialog";
import { SelectField, TextArea, TextField } from "../../ui/Field";
import { WorkFolderField } from "./WorkFolderField";

const MODELS: BotModel[] = ["default", "fable", "opus", "sonnet", "haiku"];

/** Creates a crew with its chief, or renames `crew`. */
export function CrewDialog({ crew, onClose }: { crew?: Crew; onClose(): void }) {
  const t = useT();
  const api = useApi();
  const putCrew = useApp((state) => state.putCrew);
  const selectCrew = useApp((state) => state.selectCrew);
  const [name, setName] = useState(crew?.name ?? "");
  const [folder, setFolder] = useState<string | null>(null);
  const [goal, setGoal] = useState("");
  const [chiefModel, setChiefModel] = useState<BotModel>("default");
  const selectBot = useApp((state) => state.selectBot);
  const putBot = useApp((state) => state.putBot);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    setBusy(true);
    setError(null);
    try {
      const d = t.crews.dialog;
      const saved = crew
        ? await api.call("crews.rename", { crewId: crew.id, name })
        : await api.call("crews.create", {
            name,
            ...(folder && { workFolder: folder }),
            // Every new crew starts with its chief (spec 10.2).
            lead: {
              name: d.chiefName,
              role: d.chiefRole,
              instructions: goal.trim(),
              ...(chiefModel !== "default" && { model: chiefModel }),
            },
          });
      putCrew(saved);
      if (!crew) {
        selectCrew(saved.id);
        // Straight to the chief's chat, to say what the crew should do.
        const bots = await api.call("bots.list", { crewId: saved.id });
        for (const bot of bots) {
          putBot(bot);
        }
        if (saved.leadBotId) {
          selectBot(saved.leadBotId);
        }
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
        {!crew && (
          <>
            <TextArea
              label={t.crews.dialog.goal}
              value={goal}
              rows={3}
              max={FIELD_LIMITS.instructions}
              onChange={(event) => setGoal(event.target.value)}
              placeholder={t.crews.dialog.goalPlaceholder}
              hint={t.crews.dialog.goalHint}
            />
            <WorkFolderField value={folder} onChange={setFolder} />
            <SelectField
              label={t.crews.dialog.chiefModel}
              value={chiefModel}
              onChange={(event) => setChiefModel(event.target.value as BotModel)}
              options={MODELS.map((value) => ({ value, label: t.chat.model.names[value] }))}
            />
          </>
        )}
        {error && (
          <p role="alert" className="text-danger text-sm">
            {error}
          </p>
        )}
      </form>
    </Dialog>
  );
}
