// Settings, "Account and phone": the account that keeps the copies and
// connects the phone (spec 27.6), and the phones connected to it (spec 28.7).

import { useT } from "../../i18n";
import { CloudAccount } from "./CloudAccount";
import { MobilePhones } from "./MobilePhones";
import { Section } from "./settingsParts";

export function AccountSettings() {
  const cloud = useT().account.cloud;
  return (
    <>
      <Section title={cloud.title}>
        <CloudAccount />
      </Section>
      <MobilePhones />
    </>
  );
}
