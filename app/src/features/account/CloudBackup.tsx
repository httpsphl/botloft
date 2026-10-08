// Settings, "Backup": the copies in the cloud (spec 27): send one, keep them
// up to date by themselves, bring one back. They need the account, which
// is in "Account and phone".

import { useState } from "react";
import { useT } from "../../i18n";
import { Button } from "../../ui/Button";
import { CloudAuto } from "./CloudAuto";
import { CloudCopies } from "./CloudCopies";
import { CloudUpload } from "./CloudUpload";
import { Section } from "./settingsParts";
import { useCloud } from "./useCloud";

export function CloudBackup({ goToAccount }: { goToAccount(): void }) {
  const s = useT().account.settings;
  const { status, refresh } = useCloud();
  const [version, setVersion] = useState(0);
  // No cloud set up in this Botloft: the Account page says so.
  if (!status?.url) {
    return null;
  }
  return (
    <Section title={s.copiesTitle}>
      {status.signedIn ? (
        <>
          <CloudUpload
            onSent={() => {
              setVersion((current) => current + 1);
              void refresh();
            }}
          />
          <CloudAuto />
          <CloudCopies version={version} onChanged={refresh} />
        </>
      ) : (
        <>
          <p className="text-ink-soft text-sm">{s.backupNeedsAccount}</p>
          <Button className="self-start" onClick={goToAccount}>
            {s.goToAccount}
          </Button>
        </>
      )}
    </Section>
  );
}
