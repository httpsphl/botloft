// The command that opens the tunnel to the server from this computer (spec
// 42.6), with a button that copies it.

import { useT } from "../../i18n";
import { type ServerProfile, tunnelCommand } from "../../shell/server";
import { CopyCommand } from "./CopyCommand";

export function TunnelCommand({ server }: { server: ServerProfile }) {
  const t = useT().server;
  return <CopyCommand command={tunnelCommand(server, t.address)} />;
}
