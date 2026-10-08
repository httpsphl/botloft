// Settings, "Backup", the phone block (spec 28.7): connect a phone by showing
// a code for it to scan, compare the two codes, and cut a phone off.

import { Smartphone } from "lucide-react";
import { useT } from "../../i18n";
import { MobileSignedIn } from "./MobileSignedIn";
import { Section } from "./settingsParts";
import { useCloud } from "./useCloud";
import { useMobile } from "./useMobile";

export function MobilePhones() {
  const m = useT().account.mobile;
  const { status: cloud } = useCloud();
  const { status, refresh } = useMobile();
  // No cloud set up, nothing to connect a phone to.
  if (!cloud?.url) {
    return null;
  }
  return (
    <Section title={m.title}>
      <p className="flex items-start gap-2 text-ink-soft text-sm leading-relaxed">
        <Smartphone size={16} aria-hidden className="mt-0.5 shrink-0 text-muted" />
        {m.intro}
      </p>
      {!cloud.signedIn ? (
        <p className="text-muted text-sm">{m.needAccount}</p>
      ) : (
        status && <MobileSignedIn status={status} refresh={refresh} />
      )}
    </Section>
  );
}
