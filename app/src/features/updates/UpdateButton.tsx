// Updating Botloft from inside the app (spec 15.5): a quiet title-bar
// button when a newer version is out, and one click to install it.

import { ArrowDownToLine, RefreshCw } from "lucide-react";
import { useEffect, useState } from "react";
import { errorText } from "../../lib/api";
import type { AppUpdate } from "../../lib/host";
import { useHost } from "../../store/context";
import { Button } from "../../ui/Button";
import { Callout } from "../../ui/Callout";
import { Details } from "../../ui/Details";
import { Dialog } from "../../ui/Dialog";

/** How often a running app looks for a new version. */
const CHECK_EVERY_MS = 6 * 60 * 60 * 1000;

/** The newest version the release feed offers, checked now and every few hours. */
function useAppUpdate(): AppUpdate | null {
  const host = useHost();
  const [update, setUpdate] = useState<AppUpdate | null>(null);
  useEffect(() => {
    let alive = true;
    const look = () => {
      host.checkForUpdate().then(
        (found) => alive && setUpdate(found),
        // Offline or no release yet: try again at the next check.
        () => {},
      );
    };
    look();
    const timer = setInterval(look, CHECK_EVERY_MS);
    return () => {
      alive = false;
      clearInterval(timer);
    };
  }, [host]);
  return update;
}

export function UpdateButton() {
  const update = useAppUpdate();
  const [open, setOpen] = useState(false);
  if (!update) {
    return null;
  }
  return (
    <>
      <button
        type="button"
        onClick={() => setOpen(true)}
        className="mr-1 flex h-7 items-center gap-1.5 px-2 font-medium text-work text-xs hover:bg-sunken"
      >
        <ArrowDownToLine aria-hidden size={13} />
        Update available
      </button>
      {open && <UpdateDialog update={update} onClose={() => setOpen(false)} />}
    </>
  );
}

type Progress =
  | { stage: "idle" }
  | { stage: "downloading"; fraction: number | null }
  | { stage: "failed"; error: string };

function UpdateDialog({ update, onClose }: { update: AppUpdate; onClose(): void }) {
  const [progress, setProgress] = useState<Progress>({ stage: "idle" });
  const busy = progress.stage === "downloading";

  const install = async () => {
    setProgress({ stage: "downloading", fraction: null });
    try {
      // Botloft closes when the installer starts; returning means it failed.
      await update.install((fraction) => setProgress({ stage: "downloading", fraction }));
    } catch (error) {
      setProgress({ stage: "failed", error: errorText(error) });
    }
  };

  return (
    <Dialog
      title="Update Botloft"
      onClose={busy ? () => {} : onClose}
      footer={
        <>
          <Button onClick={onClose} disabled={busy}>
            Later
          </Button>
          <Button
            variant="primary"
            icon={progress.stage === "failed" ? RefreshCw : ArrowDownToLine}
            onClick={install}
            disabled={busy}
          >
            {progress.stage === "failed" ? "Try again" : "Update now"}
          </Button>
        </>
      }
    >
      <div className="flex flex-col gap-3 text-sm leading-relaxed">
        <p>
          Botloft {update.version} is ready. Botloft closes, installs the update and opens again.
          Your bots pause for a moment and pick up where they left off.
        </p>
        {update.notes && (
          <details>
            <summary className="cursor-pointer text-ink-soft hover:text-ink">What's new</summary>
            <p className="mt-1 whitespace-pre-wrap text-ink-soft">{update.notes}</p>
          </details>
        )}
        {progress.stage === "downloading" && (
          <p role="status" className="text-ink-soft">
            {progress.fraction === null
              ? "Downloading…"
              : progress.fraction >= 1
                ? "Installing…"
                : `Downloading… ${Math.round(progress.fraction * 100)}%`}
          </p>
        )}
        {progress.stage === "failed" && (
          <Callout tone="danger" title="The update didn't install">
            Botloft keeps working on the current version.
            <Details>{progress.error}</Details>
          </Callout>
        )}
      </div>
    </Dialog>
  );
}
