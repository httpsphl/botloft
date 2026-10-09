import { type FormEvent, useState } from "react";
import { useT } from "../../i18n";
import { errorText } from "../../lib/api";
import { type BotModel, type Crew, FIELD_LIMITS } from "../../lib/protocol.gen";
import { useApi, useApp } from "../../store/context";
import { Button } from "../../ui/Button";
import { Dialog } from "../../ui/Dialog";
import { SelectField, TextArea, TextField } from "../../ui/Field";
import { notifyError } from "../../ui/toast";
import { useCatalog } from "../catalog/useCatalog";
import { addTemplateBots } from "./addTemplateBots";
import { CrewColorField } from "./CrewColorField";
import type { CrewTemplate } from "./crewTemplates";
import { PickedTemplate, TemplatePicker } from "./TemplatePicker";
import { WorkFolderField } from "./WorkFolderField";

const MODELS: BotModel[] = ["default", "fable", "opus", "sonnet", "haiku"];

/**
 * Creates a crew with its chief, or renames `crew`. `start` opens a new crew
 * already on that template, as when the owner taps an idea on the welcome
 * screen.
 */
export function CrewDialog({
  crew,
  start,
  onClose,
}: {
  crew?: Crew;
  start?: CrewTemplate | undefined;
  onClose(): void;
}) {
  const t = useT();
  const api = useApi();
  const putCrew = useApp((state) => state.putCrew);
  const selectCrew = useApp((state) => state.selectCrew);
  const [name, setName] = useState(
    crew?.name ?? (start ? t.crewTemplates.items[start.id].name : ""),
  );
  const [color, setColor] = useState<string | null>(crew?.color ?? null);
  const [folder, setFolder] = useState<string | null>(null);
  const [goal, setGoal] = useState(start ? t.crewTemplates.items[start.id].goal : "");
  const [chiefModel, setChiefModel] = useState<BotModel>("default");
  const selectBot = useApp((state) => state.selectBot);
  const putBot = useApp((state) => state.putBot);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  // A new crew starts from a template (spec 29.1) when the catalog loaded:
  // `undefined` is the list of templates, `null` an empty crew.
  const { roles, failed: catalogFailed } = useCatalog(!crew);
  const [template, setTemplate] = useState<CrewTemplate | null | undefined>(start);
  const loading = !crew && roles === null && !catalogFailed;
  const choosing = !crew && template === undefined && roles !== null;

  const pick = (picked: CrewTemplate) => {
    const item = t.crewTemplates.items[picked.id];
    setTemplate(picked);
    setName((current) => current || item.name);
    setGoal(item.goal);
  };

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    setBusy(true);
    setError(null);
    try {
      const d = t.crews.dialog;
      let saved = crew
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
      if (crew && color !== crew.color) {
        saved = await api.call("crews.setColor", { crewId: crew.id, color });
      }
      putCrew(saved);
      if (!crew) {
        selectCrew(saved.id);
        // Straight to the chief's chat, to say what the crew should do.
        const bots = await api.call("bots.list", { crewId: saved.id });
        for (const bot of bots) {
          putBot(bot);
        }
        if (template && roles) {
          const made = await addTemplateBots(api, t, saved.id, template, roles);
          for (const bot of made.added) {
            putBot(bot);
          }
          if (made.failed > 0) {
            notifyError(t.crewTemplates.partial(made.failed), made.error);
          }
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
      title={crew ? t.crews.dialog.renameTitle : choosing ? t.crewTemplates.title : t.crews.newCrew}
      onClose={onClose}
      footer={
        <>
          <Button onClick={onClose}>{t.common.cancel}</Button>
          {!loading && !choosing && (
            <Button variant="primary" type="submit" form={formId} disabled={busy}>
              {crew ? t.crews.rename : t.crews.dialog.create}
            </Button>
          )}
        </>
      }
    >
      {loading && <p className="text-muted text-sm">{t.catalog.loading}</p>}
      {choosing && roles && <TemplatePicker onPick={pick} onScratch={() => setTemplate(null)} />}
      <form
        id={formId}
        onSubmit={submit}
        hidden={loading || choosing}
        className="flex flex-col gap-3"
      >
        {template && roles && (
          <PickedTemplate
            template={template}
            roles={roles}
            onChange={() => {
              setTemplate(undefined);
              setGoal("");
            }}
          />
        )}
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
        {crew && <CrewColorField value={color} onChange={setColor} />}
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
