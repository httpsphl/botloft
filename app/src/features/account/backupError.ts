import { errorText } from "../../lib/api";
import { RpcError } from "../../lib/rpc";

/** A failure as the owner reads it: the daemon's reason, worded, or the error. */
export function backupError(failure: unknown, reasons: Record<string, string>): string {
  const reason = failure instanceof RpcError ? failure.reason : undefined;
  return (reason && reasons[reason]) || errorText(failure);
}
