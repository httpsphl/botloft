import { SendHorizontal } from "lucide-react";
import { type FormEvent, type KeyboardEvent, useId, useState } from "react";
import { useShallow } from "zustand/react/shallow";
import { useT } from "../../i18n";
import { errorText } from "../../lib/api";
import { type BotId, type CrewId, FIELD_LIMITS, type Message } from "../../lib/protocol.gen";
import { botsOf } from "../../store/app";
import { useApi, useApp } from "../../store/context";
import { Button } from "../../ui/Button";

/**
 * The owner writes to a bot of the crew, or to `botId` when given. The bot
 * reads it as a prompt from the owner (spec 9.3).
 */
export function Composer({
  crewId,
  botId,
  onSent,
}: {
  crewId: CrewId;
  botId?: BotId;
  onSent(message: Message): void;
}) {
  const api = useApi();
  const t = useT();
  const bots = useApp(useShallow((state) => botsOf(state, crewId)));
  const [chosen, setChosen] = useState<BotId | null>(null);
  const [body, setBody] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const fieldId = useId();
  const to = botId ?? (bots.some((bot) => bot.id === chosen) ? chosen : (bots[0]?.id ?? null));
  const recipient = bots.find((bot) => bot.id === to);
  const tooLong = body.length > FIELD_LIMITS.message;

  const send = async (event?: FormEvent) => {
    event?.preventDefault();
    if (!to || !body.trim() || tooLong || busy) {
      return;
    }
    setBusy(true);
    setError(null);
    try {
      onSent(await api.call("messages.send", { botId: to, body }));
      setBody("");
    } catch (failure) {
      setError(errorText(failure));
    } finally {
      setBusy(false);
    }
  };

  const onKeyDown = (event: KeyboardEvent<HTMLTextAreaElement>) => {
    if (event.key === "Enter" && (event.ctrlKey || event.metaKey)) {
      event.preventDefault();
      void send();
    }
  };

  if (bots.length === 0) {
    return null;
  }
  return (
    <form onSubmit={send} className="shrink-0 border-line border-t bg-panel px-5 py-3">
      <div className="mb-1.5 flex items-center gap-2 text-sm">
        <label htmlFor={fieldId} className="font-medium text-ink-soft">
          {t.messages.composer.messageTo}
        </label>
        {botId ? (
          <span className="font-medium">{recipient?.name}</span>
        ) : (
          <select
            aria-label={t.messages.composer.recipient}
            value={to ?? ""}
            onChange={(event) => setChosen(event.target.value)}
            className="h-7 rounded-[3px] border border-line-strong bg-canvas px-1.5 text-sm"
          >
            {bots.map((bot) => (
              <option key={bot.id} value={bot.id}>
                {bot.name} (@{bot.handle})
              </option>
            ))}
          </select>
        )}
        <span className={`ml-auto font-mono text-xs ${tooLong ? "text-danger" : "text-muted"}`}>
          {body.length > 0 && `${body.length}/${FIELD_LIMITS.message}`}
        </span>
      </div>
      <div className="flex items-end gap-2">
        <textarea
          id={fieldId}
          value={body}
          rows={3}
          onChange={(event) => setBody(event.target.value)}
          onKeyDown={onKeyDown}
          placeholder={t.messages.composer.placeholder(recipient?.handle)}
          className="min-h-16 flex-1 resize-y rounded-[3px] border border-line-strong bg-canvas px-2.5 py-2 text-sm leading-relaxed outline-none placeholder:text-muted focus:border-accent"
        />
        <Button
          variant="primary"
          type="submit"
          icon={SendHorizontal}
          disabled={busy || !body.trim() || tooLong}
        >
          {t.messages.composer.send}
        </Button>
      </div>
      {error && (
        <p role="alert" className="mt-1.5 text-danger text-sm">
          {error}
        </p>
      )}
    </form>
  );
}
