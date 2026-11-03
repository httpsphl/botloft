// Links in the chat (spec 15.3): in a bot's markdown reply and in the plain
// text of the owner's and other bots' messages, a web address is a link in
// the blue of work, underlined. It opens in the browser through the host,
// since following one inside the app would replace it.

import { Fragment, type ReactNode } from "react";
import { useT } from "../../i18n";
import { useHost } from "../../store/context";
import { attempt } from "../../ui/toast";
import { Mentions } from "./mentions";

export function isWeb(href: string | undefined): href is string {
  return href !== undefined && /^https?:\/\/\S+$/i.test(href);
}

export function ChatLink({ href, children }: { href: string | undefined; children?: ReactNode }) {
  const host = useHost();
  const t = useT();
  if (!isWeb(href)) {
    return <span className="underline decoration-dotted">{children}</span>;
  }
  return (
    <a
      href={href}
      title={href}
      onClick={(event) => {
        event.preventDefault();
        void attempt(t.chat.markdown.openLinkFailed, () => host.openUrl(href));
      }}
      className="text-work underline decoration-1 underline-offset-2 hover:decoration-2"
    >
      {children}
    </a>
  );
}

const ADDRESS = /\bhttps?:\/\/[^\s<>"'`]+/gi;
const ENDS = ".,;:!?…'\"";
const OPENS: Record<string, string> = { ")": "(", "]": "[", "}": "{" };

const count = (text: string, char: string) => text.split(char).length - 1;

/**
 * An address without what ends the sentence around it: a period or comma,
 * and a closing bracket it did not open ("(see https://a.com/x)").
 */
function trim(address: string): string {
  let end = address;
  for (;;) {
    const last = end.at(-1) ?? "/";
    const open = OPENS[last];
    if (ENDS.includes(last) || (open !== undefined && count(end, open) < count(end, last))) {
      end = end.slice(0, -1);
    } else {
      return end;
    }
  }
}

/** `text` cut into plain parts and web addresses. */
export function splitLinks(text: string): { text: string; link: boolean }[] {
  const parts: { text: string; link: boolean }[] = [];
  let from = 0;
  for (const match of text.matchAll(ADDRESS)) {
    const address = trim(match[0]);
    if (!isWeb(address)) {
      continue;
    }
    if (match.index > from) {
      parts.push({ text: text.slice(from, match.index), link: false });
    }
    parts.push({ text: address, link: true });
    from = match.index + address.length;
  }
  if (from < text.length) {
    parts.push({ text: text.slice(from), link: false });
  }
  return parts;
}

/** Plain text with its web addresses as links and its mentions marked. */
export function ChatText({ text }: { text: string }): ReactNode {
  return splitLinks(text).map((part, index) => (
    // biome-ignore lint/suspicious/noArrayIndexKey: the parts of one text never move
    <Fragment key={index}>
      {part.link ? (
        <ChatLink href={part.text}>{part.text}</ChatLink>
      ) : (
        <Mentions text={part.text} />
      )}
    </Fragment>
  ));
}
