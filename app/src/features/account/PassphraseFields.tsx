// The passphrase typed twice (spec 14.2, 27.6): for a copy saved on this
// computer and for one sent to the cloud.

import { useState } from "react";
import { useT } from "../../i18n";
import { TextField } from "../../ui/Field";

export function usePassphrase() {
  const [passphrase, setPassphrase] = useState("");
  const [repeat, setRepeat] = useState("");
  return {
    passphrase,
    repeat,
    setPassphrase,
    setRepeat,
    ready: [...passphrase].length >= 8 && repeat === passphrase,
    clear: () => {
      setPassphrase("");
      setRepeat("");
    },
  };
}

export function PassphraseFields({ value }: { value: ReturnType<typeof usePassphrase> }) {
  const b = useT().account.backup;
  const mismatch = value.repeat.length > 0 && value.repeat !== value.passphrase;
  return (
    <>
      <div className="grid grid-cols-2 gap-3">
        <TextField
          type="password"
          autoComplete="new-password"
          label={b.passphrase}
          value={value.passphrase}
          onChange={(event) => value.setPassphrase(event.target.value)}
        />
        <TextField
          type="password"
          autoComplete="new-password"
          label={b.repeat}
          value={value.repeat}
          onChange={(event) => value.setRepeat(event.target.value)}
        />
      </div>
      <p className={`text-xs ${mismatch ? "text-danger" : "text-muted"}`}>
        {mismatch ? b.mismatch : b.passphraseHint}
      </p>
    </>
  );
}
