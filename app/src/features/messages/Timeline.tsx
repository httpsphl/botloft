import { LoaderCircle } from "lucide-react";
import { type ReactNode, useLayoutEffect, useRef } from "react";
import { useT } from "../../i18n";
import type { Message } from "../../lib/protocol.gen";
import { Button } from "../../ui/Button";
import { Callout } from "../../ui/Callout";
import { MessageRow } from "./MessageRow";
import { type MessageFilter, useMessages } from "./useMessages";

/** Distance from the bottom that still counts as "reading the latest". */
const STICKY_PX = 80;

/** Messages oldest first, following new ones while the owner is at the end. */
export function Timeline({
  filter,
  empty,
  composer,
}: {
  filter: MessageFilter;
  empty: string;
  /** The composer, given what to call with a message it sent. */
  composer: (onSent: (message: Message) => void) => ReactNode;
}) {
  const t = useT();
  const text = t.messages.timeline;
  const { messages, complete, loading, error, loadOlder, add } = useMessages(filter);
  const scroller = useRef<HTMLDivElement>(null);
  const atEnd = useRef(true);
  const count = messages.length;
  const newest = messages.at(-1)?.id;

  // Only a new last message scrolls; loading older ones keeps the view.
  useLayoutEffect(() => {
    const element = scroller.current;
    if (newest && element && atEnd.current) {
      element.scrollTop = element.scrollHeight;
    }
  }, [newest]);

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <div
        ref={scroller}
        className="min-h-0 flex-1 overflow-y-auto"
        onScroll={(event) => {
          const element = event.currentTarget;
          atEnd.current =
            element.scrollHeight - element.scrollTop - element.clientHeight < STICKY_PX;
        }}
      >
        {error && (
          <div className="p-4">
            <Callout tone="danger" title={text.loadFailed}>
              {error}
            </Callout>
          </div>
        )}
        {count > 0 && !complete && (
          <div className="flex justify-center py-3">
            <Button size="sm" disabled={loading} onClick={loadOlder}>
              {text.loadOlder}
            </Button>
          </div>
        )}
        {count === 0 && loading && (
          <p role="status" className="flex items-center gap-2 p-5 text-muted text-sm">
            <LoaderCircle aria-hidden size={14} className="animate-spin" />
            {text.loading}
          </p>
        )}
        {count === 0 && !loading && !error && <p className="p-5 text-muted text-sm">{empty}</p>}
        <ol aria-label={text.list} className="offscreen-rows">
          {messages.map((message) => (
            <MessageRow key={message.id} message={message} />
          ))}
        </ol>
      </div>
      {composer((message) => {
        atEnd.current = true;
        add(message);
      })}
    </div>
  );
}
