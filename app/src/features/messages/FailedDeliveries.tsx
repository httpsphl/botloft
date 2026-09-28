import { CircleX, RotateCw } from "lucide-react";
import { useEffect, useState } from "react";
import { useShallow } from "zustand/react/shallow";
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
        {dead.length} not delivered
      </button>
      {open && <FailedDialog dead={dead} onClose={() => setOpen(false)} />}
    </>
  );
}

function FailedDialog({ dead, onClose }: { dead: Delivery[]; onClose(): void }) {
  const api = useApi();
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
    attempt("Could not retry the delivery", async () =>
      putDelivery(await api.call("deliveries.retry", { deliveryId: delivery.id })),
    );

  return (
    <Dialog
      title="Messages not delivered"
      onClose={onClose}
      width="lg"
      footer={
        <>
          <Button onClick={onClose}>Close</Button>
          <Button
            variant="primary"
            icon={RotateCw}
            onClick={async () => {
              for (const delivery of dead) {
                await retry(delivery);
              }
            }}
          >
            Retry all
          </Button>
        </>
      }
    >
      <p className="mb-3 text-ink-soft text-sm">
        The daemon stopped trying after several attempts. Retrying puts a message back in line; it
        goes out when the bot is ready.
      </p>
      <ul className="flex flex-col border border-line">
        {dead.map((delivery) => {
          const bot = bots[delivery.botId];
          const message = bodies[delivery.messageId];
          return (
            <li key={delivery.id} className="border-line border-b p-3 last:border-b-0">
              <div className="flex items-center gap-2 text-sm">
                {bot && <BotAvatar color={bot.color} size={14} />}
                <span className="font-medium">To {bot?.name}</span>
                <span className="text-muted text-xs">
                  {when(delivery.updatedAt)} · {delivery.attempts}{" "}
                  {delivery.attempts === 1 ? "try" : "tries"}
                </span>
                <Button
                  size="sm"
                  icon={RotateCw}
                  className="ml-auto"
                  onClick={() => retry(delivery)}
                >
                  Retry
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
