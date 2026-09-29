import { CircleX, RotateCw } from "lucide-react";
import { useEffect, useState } from "react";
import { useShallow } from "zustand/react/shallow";
import { useT } from "../../i18n";
import { when } from "../../lib/format";
import type { Delivery, Message, MessageId } from "../../lib/protocol.gen";
import type { AppState } from "../../store/app";
import { deadDeliveries } from "../../store/app";
import { useApi, useApp } from "../../store/context";
import { Button } from "../../ui/Button";
import { Dialog } from "../../ui/Dialog";
import { attempt } from "../../ui/toast";
import { BotAvatar } from "../bots/BotAvatar";

/**
 * Dead deliveries the owner can still act on: those to an archived bot
 * were given up on purpose (spec 9.1).
 */
export function actionableDead(state: AppState): Delivery[] {
  return deadDeliveries(state).filter((delivery) => state.bots[delivery.botId]);
}

/** Title-bar button for messages that were not delivered, with a list to retry them. */
export function FailedDeliveries() {
  const t = useT();
  const dead = useApp(useShallow(actionableDead));
  const [open, setOpen] = useState(false);
  if (dead.length === 0) {
    return null;
  }
  return (
    <>
      <button
        type="button"
        onClick={() => setOpen(true)}
        className="mr-1 flex h-7 items-center gap-1.5 px-2 font-medium text-danger text-xs hover:bg-sunken"
      >
        <CircleX aria-hidden size={13} />
        {t.messages.failed.button(dead.length)}
      </button>
      {open && <FailedDialog dead={dead} onClose={() => setOpen(false)} />}
    </>
  );
}

function FailedDialog({ dead, onClose }: { dead: Delivery[]; onClose(): void }) {
  const api = useApi();
  const t = useT();
  const text = t.messages.failed;
  const bots = useApp((state) => state.bots);
  const putDelivery = useApp((state) => state.putDelivery);
  const [bodies, setBodies] = useState<Record<MessageId, Message>>({});
  const botIds = [...new Set(dead.map((delivery) => delivery.botId))].sort().join(",");

  // The messages themselves, fetched per bot when the list opens.
  useEffect(() => {
    let alive = true;
    for (const botId of botIds.split(",")) {
      api.call("messages.list", { botId, limit: 200 }).then(
        (messages) =>
          alive &&
          setBodies((current) => ({
            ...current,
            ...Object.fromEntries(messages.map((message) => [message.id, message])),
          })),
        () => {},
      );
    }
    return () => {
      alive = false;
    };
  }, [api, botIds]);

  const retry = (delivery: Delivery) =>
    attempt(t.messages.delivery.retryFailed, async () =>
      putDelivery(await api.call("deliveries.retry", { deliveryId: delivery.id })),
    );

  return (
    <Dialog
      title={text.title}
      onClose={onClose}
      width="lg"
      footer={
        <>
          <Button onClick={onClose}>{t.common.close}</Button>
          <Button
            variant="primary"
            icon={RotateCw}
            onClick={async () => {
              for (const delivery of dead) {
                await retry(delivery);
              }
            }}
          >
            {text.retryAll}
          </Button>
        </>
      }
    >
      <p className="mb-3 text-ink-soft text-sm">{text.explanation}</p>
      <ul className="flex flex-col border border-line">
        {dead.map((delivery) => {
          const bot = bots[delivery.botId];
          const message = bodies[delivery.messageId];
          return (
            <li key={delivery.id} className="border-line border-b p-3 last:border-b-0">
              <div className="flex items-center gap-2 text-sm">
                {bot && <BotAvatar color={bot.color} size={14} />}
                <span className="font-medium">{text.to(bot?.name ?? "")}</span>
                <span className="text-muted text-xs">
                  {when(delivery.updatedAt)} · {text.tries(delivery.attempts)}
                </span>
                <Button
                  size="sm"
                  icon={RotateCw}
                  className="ml-auto"
                  onClick={() => retry(delivery)}
                >
                  {t.messages.delivery.retry}
                </Button>
              </div>
              {delivery.lastError && (
                <p className="mt-1 text-danger text-xs" data-selectable>
                  {delivery.lastError}
                </p>
              )}
              {message && (
                <p
                  className="mt-1 line-clamp-3 whitespace-pre-wrap text-ink-soft text-sm"
                  data-selectable
                >
                  {message.body}
                </p>
              )}
            </li>
          );
        })}
      </ul>
    </Dialog>
  );
}
