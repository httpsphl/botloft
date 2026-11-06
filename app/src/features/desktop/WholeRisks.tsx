// The risks of giving a bot the owner's whole desktop (spec 24.2, 24.10),
// in the dialog of the Permissions page: the confirm button lights only
// once the owner checks that they understand them. Never asked for in the
// chat.

import { useT } from "../../i18n";

/** The risks of the whole desktop, and the box the owner checks first. */
export function WholeRisks({
  bot,
  understood,
  onUnderstood,
}: {
  bot: string;
  understood: boolean;
  onUnderstood(checked: boolean): void;
}) {
  const words = useT().desktop.grants;
  return (
    <>
      <ul className="mt-4 flex list-disc flex-col gap-1.5 pl-5 text-sm">
        {words.wholeRisks(bot).map((risk) => (
          <li key={risk}>{risk}</li>
        ))}
      </ul>
      <label className="mt-4 flex items-center gap-2 font-medium text-sm">
        <input
          type="checkbox"
          checked={understood}
          onChange={(event) => onUnderstood(event.target.checked)}
        />
        {words.awayUnderstood}
      </label>
    </>
  );
}
