import { CircleCheck, CircleX, Clock, LoaderCircle, RotateCw } from "lucide-react";
import { fromNow } from "../../lib/format";
import type { Delivery } from "../../lib/protocol.gen";
import { useApi, useApp } from "../../store/context";
import { Button } from "../../ui/Button";
import { attempt } from "../../ui/toast";

/** Where a message is on its way into the bot's inbox (spec 9.1). */
export function DeliveryStatus({ delivery }: { delivery: Delivery }) {
  const api = useApi();
  const putDelivery = useApp((state) => state.putDelivery);
  const line = "flex items-center gap-1.5 text-xs";

  switch (delivery.state) {
    case "sent":
      return (
        <p className={`${line} text-muted`}>
          <CircleCheck aria-hidden size={12} />
          Delivered
        </p>
      );
    case "sending":
      return (
        <p className={`${line} text-work`}>
          <LoaderCircle aria-hidden size={12} className="animate-spin" />
          Delivering
        </p>
      );
    case "pending":
      return (
        <p className={`${line} ${delivery.attempts > 0 ? "text-warn" : "text-muted"}`}>
          <Clock aria-hidden size={12} />
          {delivery.attempts > 0
            ? `Retrying ${fromNow(delivery.nextAttemptAt)}, after ${delivery.attempts} failed ${delivery.attempts === 1 ? "try" : "tries"}`
            : "Waiting for the bot"}
          {delivery.lastError && <span className="text-muted">· {delivery.lastError}</span>}
        </p>
      );
    case "dead":
      return (
        <div className={`${line} flex-wrap text-danger`}>
          <CircleX aria-hidden size={12} />
          <span className="font-medium">Not delivered</span>
          {delivery.lastError && <span className="text-ink-soft">· {delivery.lastError}</span>}
          <Button
            size="sm"
            variant="ghost"
            icon={RotateCw}
            className="ml-1"
            onClick={() =>
              attempt("Could not retry the delivery", async () =>
                putDelivery(await api.call("deliveries.retry", { deliveryId: delivery.id })),
              )
            }
          >
            Retry
          </Button>
        </div>
      );
  }
}
