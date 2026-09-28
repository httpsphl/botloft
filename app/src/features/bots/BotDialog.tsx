import { Check } from "lucide-react";
import { type FormEvent, useState } from "react";
import { errorText } from "../../lib/api";
import { AVATAR_PALETTE, type Bot, type CrewId, FIELD_LIMITS } from "../../lib/protocol.gen";
import { useApi, useApp } from "../../store/context";
import { Button } from "../../ui/Button";
import { Dialog } from "../../ui/Dialog";
import { TextArea, TextField } from "../../ui/Field";
import { BotAvatar } from "./BotAvatar";

type Props = { onClose(): void } & ({ crewId: CrewId; bot?: undefined } | { bot: Bot });

/** Creates a bot in `crewId`, or edits `bot`. */
export function BotDialog(props: Props) {
  const api = useApi();
  const putBot = useApp((state) => state.putBot);
  const selectBot = useApp((state) => state.selectBot);
  const editing = props.bot;
  const [name, setName] = useState(editing?.name ?? "");
  const [role, setRole] = useState(editing?.role ?? "");
  const [instructions, setInstructions] = useState(editing?.instructions ?? "");
  const [color, setColor] = useState<string | undefined>(editing?.color);
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
      } else {
        bot = await api.call("bots.create", {
          crewId: props.crewId,
          name,
          role,
          instructions,
          ...(color !== undefined && { color }),
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
      title={editing ? `Edit ${editing.name}` : "New bot"}
      onClose={props.onClose}
      width="lg"
      footer={
        <>
          <Button onClick={props.onClose}>Cancel</Button>
          <Button variant="primary" type="submit" form={formId} disabled={busy}>
            {editing ? "Save" : "Create bot"}
          </Button>
        </>
      }
    >
      <form id={formId} onSubmit={submit} className="flex flex-col gap-4">
        <div className="grid grid-cols-[1fr_1.4fr] gap-3">
          <TextField
            label="Name"
            value={name}
            max={FIELD_LIMITS.name}
            onChange={(event) => setName(event.target.value)}
            placeholder="Reviewer"
            hint="Other bots reach it by the handle made from this name."
            autoFocus
            required
          />
          <TextField
            label="Role"
            value={role}
            max={FIELD_LIMITS.role}
            onChange={(event) => setRole(event.target.value)}
            placeholder="Reviews pull requests before they merge"
          />
        </div>
        <TextArea
          label="Instructions"
          value={instructions}
          max={FIELD_LIMITS.instructions}
          rows={8}
          onChange={(event) => setInstructions(event.target.value)}
          placeholder="How this bot works, what it may do on its own and when to ask."
          hint={
            editing
              ? "Saved to the bot's rules now; the bot reads them the next time it starts."
              : undefined
          }
        />
        <fieldset className="flex flex-col gap-1.5">
          <legend className="mb-1.5 font-medium text-ink-soft text-sm">Color</legend>
          <div className="flex items-center gap-1.5">
            {AVATAR_PALETTE.map((swatch) => (
              <button
                key={swatch}
                type="button"
                aria-label={`Color ${swatch}`}
                aria-pressed={color === swatch}
                onClick={() => setColor(swatch)}
                className={`relative rounded-[3px] p-0.5 ${color === swatch ? "ring-2 ring-accent" : ""}`}
              >
                <BotAvatar color={swatch} size={26} />
                {color === swatch && (
                  <Check aria-hidden size={12} className="absolute right-0 bottom-0 text-white" />
                )}
              </button>
            ))}
          </div>
          {!editing && color === undefined && (
            <p className="text-muted text-xs">Left unset, the bot gets the crew's next color.</p>
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
