// "Connect to a server" under a problem on the opening screen (spec 42.6):
// a button that opens the key form in place.

import { Server } from "lucide-react";
import { useState } from "react";
import { useT } from "../../i18n";
import { Button } from "../../ui/Button";
import { ServerKeyForm } from "./ServerKeyForm";

export function ServerOffer() {
  const t = useT().server;
  const [open, setOpen] = useState(false);
  if (!open) {
    return (
      <Button variant="ghost" icon={Server} onClick={() => setOpen(true)}>
        {t.form.title}
      </Button>
    );
  }
  return (
    <div className="flex w-full flex-col gap-3 rounded-2xl bg-bubble p-4">
      <h2 className="font-semibold text-sm">{t.form.title}</h2>
      <ServerKeyForm onCancel={() => setOpen(false)} />
    </div>
  );
}
