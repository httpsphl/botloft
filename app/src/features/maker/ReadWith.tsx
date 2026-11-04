// Which model and effort the Maker reads the summary with, chosen right
// before sending it (spec 36.7): the same pickers as below the Maker's
// chat, so a big history can go to a stronger model without looking for
// them.

import { useT } from "../../i18n";
import { makerBot } from "../../store/app";
import { useApp } from "../../store/context";
import { notify } from "../../ui/toast";
import { EffortPicker } from "../chat/EffortPicker";
import { ModelPicker } from "../chat/ModelPicker";

export function ReadWith() {
  const h = useT().maker.history;
  const maker = useApp(makerBot);
  if (!maker) return null;
  return (
    <div className="@container flex flex-wrap items-center justify-between gap-2 rounded-xl border border-line px-4 py-2.5">
      <div className="min-w-0">
        <p className="font-medium text-sm">{h.readWith}</p>
        <p className="text-muted text-xs">{h.readWithHint}</p>
      </div>
      <div className="flex items-center gap-1">
        <EffortPicker bot={maker} onLater={notify} />
        <ModelPicker bot={maker} onLater={notify} />
      </div>
    </div>
  );
}
