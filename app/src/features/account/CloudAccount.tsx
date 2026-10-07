// Settings, "Backup", the account block (spec 27.6): sign in with an e-mail
// link, then send light copies to the cloud and bring them back.

import { useT } from "../../i18n";
import { CloudSignedIn } from "./CloudSignedIn";
import { CloudSignIn } from "./CloudSignIn";
import { useCloud } from "./useCloud";

export function CloudAccount() {
  const t = useT().account.cloud;
  const { status, refresh, expired, forgetExpired } = useCloud();
  if (!status) {
    return null;
  }
  if (!status.url) {
    return <p className="text-muted text-sm">{t.noCloud}</p>;
  }
  if (status.signedIn) {
    return <CloudSignedIn status={status} refresh={refresh} />;
  }
  return (
    <CloudSignIn
      status={status}
      refresh={refresh}
      expired={expired}
      forgetExpired={forgetExpired}
    />
  );
}
