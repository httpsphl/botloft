// When the app's own tunnel to the server does not open (spec 42.6): what
// went wrong in plain words, what `ssh` said under "Details", and, when the
// server does not answer, its address to change.

import { RefreshCw } from "lucide-react";
import { type ReactNode, useState } from "react";
import { useT } from "../../i18n";
import { isHost, type ServerProfile, server } from "../../shell/server";
import type { TunnelProblem } from "../../shell/tunnel";
import { Button } from "../../ui/Button";
import { Callout } from "../../ui/Callout";
import { Details } from "../../ui/Details";
import { TextField } from "../../ui/Field";

export function TunnelTrouble({
  chosen,
  problem,
  onRetry,
  children,
}: {
  chosen: ServerProfile;
  problem: TunnelProblem;
  onRetry: () => void;
  /** More ways out, beside "Try again". */
  children?: ReactNode;
}) {
  const t = useT();
  const own = t.server.own;
  const words = own.trouble[problem.reason];
  const ssh = chosen.ssh;
  const [address, setAddress] = useState(ssh?.host ?? "");
  const typed = address.trim();
  const changed = ssh !== undefined && typed !== ssh.host && isHost(typed);

  return (
    <div className="flex w-full flex-col items-start gap-4">
      <div className="w-full">
        <Callout tone="warn" title={words.title(chosen.name)}>
          {words.body(ssh?.host ?? "")}
          {problem.detail !== "" && <Details>{problem.detail}</Details>}
        </Callout>
      </div>
      {problem.reason === "unreachable" && ssh && (
        <form
          className="flex w-full items-end gap-2"
          onSubmit={(event) => {
            event.preventDefault();
            if (changed) {
              // A new address starts the link over (App.tsx).
              server.set({ ...chosen, ssh: { ...ssh, host: typed } });
            }
          }}
        >
          <div className="flex-1">
            <TextField
              label={own.address}
              value={address}
              onChange={(event) => setAddress(event.target.value)}
            />
          </div>
          <Button type="submit" disabled={!changed}>
            {own.use}
          </Button>
        </form>
      )}
      <div className="flex gap-2">
        <Button variant="primary" icon={RefreshCw} onClick={onRetry}>
          {t.common.tryAgain}
        </Button>
        {children}
      </div>
    </div>
  );
}
