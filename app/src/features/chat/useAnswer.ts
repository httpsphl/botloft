import { useState } from "react";
import type { ApprovalItem } from "../../lib/protocol.gen";
import { useApi } from "../../store/context";
import { attempt } from "../../ui/toast";

/** Answering a permission request; the note goes to the bot on a denial. */
export function useAnswer(approval: ApprovalItem, failed: { allow: string; deny: string }) {
  const api = useApi();
  const [note, setNote] = useState("");
  const [busy, setBusy] = useState(false);
  const answer = async (allow: boolean) => {
    setBusy(true);
    const trimmed = note.trim();
    await attempt(allow ? failed.allow : failed.deny, () =>
      api.call("approvals.answer", {
        approvalId: approval.approvalId,
        allow,
        ...(allow || !trimmed ? {} : { note: trimmed }),
      }),
    );
    setBusy(false);
  };
  return { note, setNote, busy, answer };
}
