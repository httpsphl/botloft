import { Check, Pipette } from "lucide-react";
import { type FormEvent, useState } from "react";
import { useT } from "../../i18n";
import { errorText } from "../../lib/api";
import {
  type AgentKind,
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
import { Toggle } from "../account/settingsParts";
import { BotAvatar } from "./BotAvatar";
import { ColorPicker } from "./ColorPicker";

const MODELS: BotModel[] = ["default", "fable", "opus", "sonnet", "haiku"];
/** Stable, for a selector: a new array each time would render in a loop. */
const CLAUDE_ONLY: AgentKind[] = ["claude"];

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
  const custom = color !== undefined && !(AVATAR_PALETTE as readonly string[]).includes(color);
  const [picking, setPicking] = useState(custom);
  const [model, setModel] = useState<BotModel>(editing?.model ?? "default");
  // One command prefix per line, for a bot whose agent cannot ask (spec 30).
  const [commands, setCommands] = useState((editing?.allowedCommands ?? []).join("\n"));
  // Which agent runs the bot is chosen once, when it is made (spec 30).
  const agents = useApp((state) => state.system?.enabledAgents ?? CLAUDE_ONLY);
  const preferred = useApp((state) => state.settings?.defaultAgent ?? "claude");
  const [agent, setAgent] = useState<AgentKind>(agents.includes(preferred) ? preferred : "claude");
  const runsOn: AgentKind = editing?.agent ?? agent;
  const fullAccess = commands
    .split("\n")
    .map((line) => line.trim())
    .includes("*");
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
          ...(editing.agent !== "claude" &&
            commands !== editing.allowedCommands.join("\n") && {
              allowedCommands: commands.split("\n"),
            }),
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
          ...(runsOn === "claude" && model !== "default" && { model }),
          ...(runsOn !== "claude" && { agent: runsOn }),
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
        {!editing && agents.length > 1 && (
          <div className="flex flex-col gap-2">
            <SelectField
              label={t.bots.dialog.agent.title}
              value={agent}
              onChange={(event) => setAgent(event.target.value as AgentKind)}
              options={agents.map((value) => ({ value, label: t.bots.dialog.agent.names[value] }))}
            />
            {agent !== "claude" && (
              <p role="note" className="rounded-lg bg-sunken p-2.5 text-ink-soft text-xs">
                {agent === "codex"
                  ? t.bots.dialog.agent.experimentalCodex
                  : t.bots.dialog.agent.experimental}
              </p>
            )}
          </div>
        )}
        {editing && editing.agent === "agy" && (
          <>
            <Toggle
              label={t.bots.dialog.fullAccess.title}
              hint={fullAccess ? t.bots.dialog.fullAccess.on : t.bots.dialog.fullAccess.off}
              checked={fullAccess}
              onChange={(on) => setCommands(on ? "*" : "")}
            />
            {!fullAccess && (
              <TextArea
                label={t.bots.dialog.commands.title}
                value={commands}
                rows={4}
                onChange={(event) => setCommands(event.target.value)}
                placeholder={t.bots.dialog.commands.placeholder}
                hint={t.bots.dialog.commands.hint}
              />
            )}
          </>
        )}
        {runsOn === "claude" && (
          <SelectField
            label={t.chat.model.title}
            value={model}
            onChange={(event) => setModel(event.target.value as BotModel)}
            options={MODELS.map((value) => ({ value, label: t.chat.model.names[value] }))}
            hint={t.chat.model.cost}
          />
        )}
        <fieldset className="flex flex-col gap-1.5">
          <legend className="mb-1.5 font-medium text-ink-soft text-sm">
            {t.bots.dialog.color}
          </legend>
          <div className="flex flex-wrap items-center gap-1.5">
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
            <button
              type="button"
              aria-label={t.bots.dialog.custom}
              title={t.bots.dialog.custom}
              aria-expanded={picking}
              aria-pressed={custom}
              onClick={() => setPicking(!picking)}
              className={`relative flex size-[30px] items-center justify-center rounded-lg ${
                custom
                  ? "ring-2 ring-accent"
                  : "border border-line-strong border-dashed text-muted hover:text-ink"
              }`}
            >
              {custom && color ? (
                <BotAvatar color={color} size={26} />
              ) : (
                <Pipette aria-hidden size={14} />
              )}
            </button>
          </div>
          {picking && <ColorPicker value={color ?? AVATAR_PALETTE[0]} onChange={setColor} />}
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
