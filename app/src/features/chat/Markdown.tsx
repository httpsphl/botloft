// A bot's reply as markdown (GitHub flavor). Raw HTML is never rendered.
// Links open in the browser (ChatText.tsx). Images are not fetched: a reply
// could use one to call home, so they show as links. How it reads is in
// chat-md.css.

import { ExternalLink, ImageOff } from "lucide-react";
import { memo } from "react";
import ReactMarkdown, { type Components, type ExtraProps } from "react-markdown";
import remarkGfm from "remark-gfm";
import { useT } from "../../i18n";
import { isWeb, ChatLink as Link } from "./ChatText";
import { remarkMentions } from "./mentions";

function Picture({ src, alt }: { src: string | Blob | undefined; alt: string | undefined }) {
  const t = useT();
  const label = alt || t.chat.markdown.image;
  const href = typeof src === "string" ? src : undefined;
  return (
    <span className="inline-flex items-center gap-1 text-muted">
      <ImageOff aria-hidden size={13} />
      {isWeb(href) ? (
        <Link href={href}>
          {label}
          <ExternalLink aria-hidden size={11} className="ml-0.5 inline" />
        </Link>
      ) : (
        label
      )}
    </span>
  );
}

/**
 * A paragraph of bold words only ("**What I checked**"), which bots write
 * as a title: it opens a section like one.
 */
function isTitle(node: ExtraProps["node"]): boolean {
  const parts = node?.children.filter((child) => child.type !== "text" || child.value.trim());
  const only = parts?.length === 1 ? parts[0] : undefined;
  return only?.type === "element" && only.tagName === "strong";
}

const plugins = [remarkGfm, remarkMentions];

const components: Components = {
  a: ({ href, children }) => <Link href={href}>{children}</Link>,
  img: ({ src, alt }) => <Picture src={src} alt={alt} />,
  p: ({ node, children }) => <p data-title={isTitle(node) || undefined}>{children}</p>,
};

/** `streaming` shows a caret after the text, while the bot still writes it. */
export const Markdown = memo(function Markdown({
  text,
  streaming = false,
}: {
  text: string;
  streaming?: boolean;
}) {
  return (
    <div className={`chat-md ${streaming ? "is-streaming" : ""}`} data-selectable>
      <ReactMarkdown remarkPlugins={plugins} components={components}>
        {text}
      </ReactMarkdown>
    </div>
  );
});
