import { Check } from "lucide-react";
import { type FormEvent, useState } from "react";
import { useT } from "../../i18n";
import { errorText } from "../../lib/api";
import {
  AVATAR_PALETTE,
  type Bot,
  type BotModel,
  type CrewId,
  FIELD_LIMITS,
} from "../../lib/protocol.gen";
import { useApi, useApp } from "../../store/context";
import { Button } from "../../ui/Button";
import { Dialog } from "../../ui/Dialog";
import { SelectField, TextArea, TextField } from "../../ui/Field";
import { BotAvatar } from "./BotAvatar";

const MODELS: BotModel[] = ["default", "fable", "opus", "sonnet", "haiku"];

type Props = { onClose(): void } & ({ crewId: CrewId; bot?: undefined } | { bot: Bot });

/** Creates a bot in `crewId`, or edits `bot`. */
export function BotDialog(props: Props) {
  const t = useT();
  const api = useApi();
  const putBot = useApp((state) => state.putBot);
  const selectBot = useApp((state) => state.selectBot);
  const editing = props.bot;
  const [name, setName] = useState(editing?.name ?? "");
  const [role, setRole] = useState(editing?.role ?? "");
  const [instructions, setInstructions] = useState(editing?.instructions ?? "");
  const [color, setColor] = useState<string | undefined>(editing?.color);
  const [model, setModel] = useState<BotModel>(editing?.model ?? "default");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    setBusy(true);
    setError(null);
    try {
      let bot: Bot;
      if (editing) {
        bot = await api.call("bots.update", {
          botId: editing.id,
          ...(name !== editing.name && { name }),
          ...(role !== editing.role && { role }),
          ...(instructions !== editing.instructions && { instructions }),
          ...(color !== undefined && color !== editing.color && { color }),
        });
        if (model !== editing.model) {
          // The bot restarts on it once nothing is in progress.
          bot = await api.call("bots.setModel", { botId: editing.id, model });
        }
      } else {
        bot = await api.call("bots.create", {
          crewId: props.crewId,
          name,
          role,
          instructions,
          ...(color !== undefined && { color }),
          ...(model !== "default" && { model }),
        });
      }
      putBot(bot);
      selectBot(bot.id);
      props.onClose();
    } catch (failure) {
      setError(errorText(failure));
      setBusy(false);
    }
  };

  const formId = "bot-form";
  return (
    <Dialog
      title={editing ? t.bots.dialog.editTitle(editing.name) : t.bots.dialog.newTitle}
      onClose={props.onClose}
      width="lg"
      footer={
        <>
          <Button onClick={props.onClose}>{t.common.cancel}</Button>
          <Button variant="primary" type="submit" form={formId} disabled={busy}>
            {editing ? t.bots.dialog.save : t.bots.dialog.create}
          </Button>
        </>
      }
    >
      <form id={formId} onSubmit={submit} className="flex flex-col gap-4">
        <div className="grid grid-cols-[1fr_1.4fr] gap-3">
          <TextField
            label={t.bots.dialog.name}
            value={name}
            max={FIELD_LIMITS.name}
            onChange={(event) => setName(event.target.value)}
            placeholder={t.bots.dialog.namePlaceholder}
            hint={t.bots.dialog.nameHint}
            autoFocus
            required
          />
          <TextField
            label={t.bots.dialog.role}
            value={role}
            max={FIELD_LIMITS.role}
            onChange={(event) => setRole(event.target.value)}
            placeholder={t.bots.dialog.rolePlaceholder}
          />
        </div>
        <TextArea
          label={t.bots.dialog.instructions}
          value={instructions}
          max={FIELD_LIMITS.instructions}
          rows={8}
          onChange={(event) => setInstructions(event.target.value)}
          placeholder={t.bots.dialog.instructionsPlaceholder}
          hint={editing ? t.bots.dialog.instructionsHint : undefined}
        />
        <SelectField
          label={t.chat.model.title}
          value={model}
          onChange={(event) => setModel(event.target.value as BotModel)}
          options={MODELS.map((value) => ({ value, label: t.chat.model.names[value] }))}
          hint={t.chat.model.cost}
        />
        <fieldset className="flex flex-col gap-1.5">
          <legend className="mb-1.5 font-medium text-ink-soft text-sm">
            {t.bots.dialog.color}
          </legend>
          <div className="flex items-center gap-1.5">
            {AVATAR_PALETTE.map((swatch) => (
              <button
                key={swatch}
                type="button"
                aria-label={t.bots.dialog.swatch(swatch)}
                aria-pressed={color === swatch}
                onClick={() => setColor(swatch)}
                className={`relative rounded-lg p-0.5 ${color === swatch ? "ring-2 ring-accent" : ""}`}
              >
                <BotAvatar color={swatch} size={26} />
                {color === swatch && (
                  <Check aria-hidden size={12} className="absolute right-0 bottom-0 text-white" />
                )}
              </button>
            ))}
          </div>
          {!editing && color === undefined && (
            <p className="text-muted text-xs">{t.bots.dialog.colorUnset}</p>
          )}
        </fieldset>
        {error && (
          <p role="alert" className="text-danger text-sm">
            {error}
          </p>
        )}
      </form>
    </Dialog>
  );
}
