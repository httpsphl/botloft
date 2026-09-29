// A bot's reply as markdown (GitHub flavor). Raw HTML is never rendered.
// Links open in the browser through the host, since following one inside
// the app would replace it. Images are not fetched: a reply could use one
// to call home, so they show as links.

import { ExternalLink, ImageOff } from "lucide-react";
import { memo, type ReactNode } from "react";
import ReactMarkdown, { type Components } from "react-markdown";
import remarkGfm from "remark-gfm";
import { useT } from "../../i18n";
import { useHost } from "../../store/context";
import { attempt } from "../../ui/toast";

function isWeb(href: string | undefined): href is string {
  return href !== undefined && /^https?:\/\/\S+$/i.test(href);
}

function Link({ href, children }: { href: string | undefined; children?: ReactNode }) {
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

const components: Components = {
  a: ({ href, children }) => <Link href={href}>{children}</Link>,
  img: ({ src, alt }) => <Picture src={src} alt={alt} />,
};

export const Markdown = memo(function Markdown({ text }: { text: string }) {
  return (
    <div className="chat-md" data-selectable>
      <ReactMarkdown remarkPlugins={[remarkGfm]} components={components}>
        {text}
      </ReactMarkdown>
    </div>
  );
});
