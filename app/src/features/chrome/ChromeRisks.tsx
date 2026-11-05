// The risks of giving the owner's own Chrome to an agent (spec 37.2), and
// the box the owner checks before the confirm button lights.

import { useT } from "../../i18n";

export function ChromeRisks({
  bot,
  understood,
  onUnderstood,
}: {
  bot: string;
  understood: boolean;
  onUnderstood(checked: boolean): void;
}) {
  const words = useT().chrome.dialog;
  return (
    <>
      <ul className="mt-3 flex list-disc flex-col gap-1.5 pl-5 text-sm">
        {words.risks(bot).map((risk) => (
          <li key={risk}>{risk}</li>
        ))}
      </ul>
      <label className="mt-3 flex items-center gap-2 font-medium text-sm">
        <input
          type="checkbox"
          checked={understood}
          onChange={(event) => onUnderstood(event.target.checked)}
        />
        {words.understood}
      </label>
    </>
  );
}
