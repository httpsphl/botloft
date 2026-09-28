import { type ReactNode, useState } from "react";
import { Button } from "./Button";
import { Dialog } from "./Dialog";

/** Asks before something that cannot be undone from the app. */
export function Confirm({
  title,
  children,
  confirmLabel,
  onConfirm,
  onClose,
}: {
  title: string;
  children: ReactNode;
  confirmLabel: string;
  onConfirm(): Promise<unknown>;
  onClose(): void;
}) {
  const [busy, setBusy] = useState(false);
  const confirm = async () => {
    setBusy(true);
    try {
      await onConfirm();
    } finally {
      setBusy(false);
    }
    onClose();
  };
  return (
    <Dialog
      title={title}
      onClose={onClose}
      footer={
        <>
          <Button onClick={onClose}>Cancel</Button>
          <Button variant="danger" disabled={busy} onClick={confirm}>
            {confirmLabel}
          </Button>
        </>
      }
    >
      <div className="text-ink-soft text-sm leading-relaxed">{children}</div>
    </Dialog>
  );
}
