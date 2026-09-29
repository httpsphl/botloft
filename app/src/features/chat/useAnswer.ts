import { useState } from "react";
import type { ApprovalItem } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";
import { attempt } from "../../ui/toast";

/**
 * Answering a request; the note goes to the bot on a denial. `input` is a
 * bot suggestion as the owner changed it (spec 10.2).
 */
export function useAnswer(approval: ApprovalItem, failed: { allow: string; deny: string }) {
  const api = useApi();
  const [note, setNote] = useState("");
  const [busy, setBusy] = useState(false);
  const answer = async (allow: boolean, input?: string) => {
    setBusy(true);
    const trimmed = note.trim();
    await attempt(allow ? failed.allow : failed.deny, () =>
      api.call("approvals.answer", {
        approvalId: approval.approvalId,
        allow,
        ...(allow || !trimmed ? {} : { note: trimmed }),
        ...(allow && input !== undefined ? { input } : {}),
      }),
    );
    setBusy(false);
  };
  return { note, setNote, busy, answer };
}
