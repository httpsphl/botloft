import { CheckCheck, CircleCheck, CircleX, Clock, LoaderCircle, RotateCw } from "lucide-react";
import { useT } from "../../i18n";
import { fromNow } from "../../lib/format";
import type { Delivery } from "../../lib/protocol.gen";
import { useApi, useApp } from "../../store/context";
import { Button } from "../../ui/Button";
import { attempt } from "../../ui/toast";

/** Where a message is on its way to the bot (spec 9.1). */
export function DeliveryStatus({ delivery }: { delivery: Delivery }) {
  const api = useApi();
  const t = useT();
  const text = t.messages.delivery;
  const putDelivery = useApp((state) => state.putDelivery);
  const line = "flex items-center gap-1.5 text-xs";

  switch (delivery.state) {
    case "sent":
      return delivery.readAt === null ? (
        <p className={`${line} text-muted`}>
          <CircleCheck aria-hidden size={12} />
          {text.delivered}
        </p>
      ) : (
        // The two marks turn blue, as in chat apps; the word says it too.
        <p className={`${line} text-muted`} title={text.readTitle}>
          <CheckCheck aria-hidden size={12} className="text-read" />
          {text.read}
        </p>
      );
    case "sending":
      return (
        <p className={`${line} text-work`}>
          <LoaderCircle aria-hidden size={12} className="animate-spin" />
          {text.delivering}
        </p>
      );
    case "pending":
      return (
        <p className={`${line} ${delivery.attempts > 0 ? "text-warn" : "text-muted"}`}>
          <Clock aria-hidden size={12} />
          {delivery.attempts > 0
            ? text.retrying(fromNow(delivery.nextAttemptAt), delivery.attempts)
            : text.waiting}
          {delivery.lastError && <span className="text-muted">· {delivery.lastError}</span>}
        </p>
      );
    case "dead":
      return (
        <div className={`${line} flex-wrap text-danger`}>
          <CircleX aria-hidden size={12} />
          <span className="font-medium">{text.notDelivered}</span>
          {delivery.lastError && <span className="text-ink-soft">· {delivery.lastError}</span>}
          <Button
            size="sm"
            variant="ghost"
            icon={RotateCw}
            className="ml-1"
            onClick={() =>
              attempt(text.retryFailed, async () =>
                putDelivery(await api.call("deliveries.retry", { deliveryId: delivery.id })),
              )
            }
          >
            {text.retry}
          </Button>
        </div>
      );
  }
}
