import { ExternalLink } from "lucide-react";
import { useT } from "../../i18n";
import { useHost } from "../../store/context";
import { Button } from "../../ui/Button";
import { Details } from "../../ui/Details";
import { attempt } from "../../ui/toast";

/** Claude Code's official install page. */
export const CLAUDE_CODE_SETUP = "https://code.claude.com/docs/en/setup";

/** What to do when bots cannot start because Claude Code is missing or unusable. */
export function ClaudeCodeHelp({ error }: { error: string }) {
  const c = useT().onboarding.claudeCode;
  const host = useHost();
  return (
    <div>
      <p>{c.help}</p>
      <Button
        className="mt-2"
        size="sm"
        icon={ExternalLink}
        onClick={() => attempt(c.openFailed, () => host.openUrl(CLAUDE_CODE_SETUP))}
      >
        {c.install}
      </Button>
      <Details>{error}</Details>
    </div>
  );
}
