import { ExternalLink } from "lucide-react";
import { useHost } from "../../store/context";
import { Button } from "../../ui/Button";
import { Details } from "../../ui/Details";
import { attempt } from "../../ui/toast";

/** Claude Code's official install page. */
export const CLAUDE_CODE_SETUP = "https://code.claude.com/docs/en/setup";

/** What to do when bots cannot start because Claude Code is missing or unusable. */
export function ClaudeCodeHelp({ error }: { error: string }) {
  const host = useHost();
  return (
    <div>
      <p>
        Botloft runs your bots with Claude Code. Install it or update it, open it once to sign in,
        and Botloft picks it up within 30 seconds.
      </p>
      <Button
        className="mt-2"
        size="sm"
        icon={ExternalLink}
        onClick={() => attempt("Could not open the link", () => host.openUrl(CLAUDE_CODE_SETUP))}
      >
        How to install Claude Code
      </Button>
      <Details>{error}</Details>
    </div>
  );
}
