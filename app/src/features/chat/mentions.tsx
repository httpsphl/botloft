// A bot named by its handle ("@writer") stands out in what bots write, so
// it is not lost in the middle of a sentence (spec 15.3). An e-mail address
// or a path with an @ inside is not a mention.

import type { ReactNode } from "react";

const MENTION = /(^|[^\w@./\\-])(@[a-z0-9][\w-]*)/gi;

/** `text` cut into plain parts and mentions. */
export function splitMentions(text: string): { text: string; mention: boolean }[] {
  const parts: { text: string; mention: boolean }[] = [];
  let from = 0;
  for (const match of text.matchAll(MENTION)) {
    const start = match.index + (match[1] as string).length;
    if (start > from) {
      parts.push({ text: text.slice(from, start), mention: false });
    }
    const mention = match[2] as string;
    parts.push({ text: mention, mention: true });
    from = start + mention.length;
  }
  if (from < text.length) {
    parts.push({ text: text.slice(from), mention: false });
  }
  return parts;
}

/** Plain text with its mentions marked. */
export function Mentions({ text }: { text: string }): ReactNode {
  return splitMentions(text).map((part, index) =>
    part.mention ? (
      // biome-ignore lint/suspicious/noArrayIndexKey: the parts of one text never move
      <span key={index} className="chat-mention">
        {part.text}
      </span>
    ) : (
      part.text
    ),
  );
}

interface Node {
  type: string;
  value?: string;
  children?: Node[];
  data?: { hName: string; hProperties: { className: string[] } };
}

function mark(node: Node) {
  // A link already stands out, and code is shown as it was written.
  if (!node.children || node.type === "link" || node.type === "linkReference") {
    return;
  }
  node.children = node.children.flatMap((child): Node[] => {
    if (child.type !== "text" || child.value === undefined) {
      mark(child);
      return [child];
    }
    return splitMentions(child.value).map((part) =>
      part.mention
        ? {
            type: "mention",
            data: { hName: "span", hProperties: { className: ["chat-mention"] } },
            children: [{ type: "text", value: part.text }],
          }
        : { type: "text", value: part.text },
    );
  });
}

/** A remark plugin that marks the mentions of a markdown reply. */
export function remarkMentions() {
  return (tree: Node) => mark(tree);
}
