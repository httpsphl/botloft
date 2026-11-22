// A command for a terminal, with a button that copies it (spec 42.6, 42.9).

import { Check, Copy } from "lucide-react";
import { useEffect, useState } from "react";
import { useT } from "../../i18n";
import { Button } from "../../ui/Button";

/** How long "Copied" shows. */
const COPIED_MS = 2000;

export function CopyCommand({ command }: { command: string }) {
  const t = useT().server;
  const [copied, setCopied] = useState(false);
  useEffect(() => {
    if (!copied) {
      return;
    }
    const timer = setTimeout(() => setCopied(false), COPIED_MS);
    return () => clearTimeout(timer);
  }, [copied]);

  const copy = async () => {
    try {
      await navigator.clipboard.writeText(command);
      setCopied(true);
    } catch {
      // No clipboard here: the command stays on screen, selectable.
    }
  };

  return (
    <div className="flex items-start gap-2">
      <code className="min-w-0 flex-1 select-all break-all rounded-lg border border-line bg-sunken px-3 py-2 font-mono text-xs leading-relaxed">
        {command}
      </code>
      <Button icon={copied ? Check : Copy} onClick={copy}>
        {copied ? t.copied : t.copy}
      </Button>
    </div>
  );
}
