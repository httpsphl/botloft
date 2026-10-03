import { CircleAlert, Info, TriangleAlert } from "lucide-react";
import type { ReactNode } from "react";
import { TONES } from "./tone";

type Tone = "info" | "warn" | "danger";

const tones: Record<Tone, { icon: typeof Info; frame: string; mark: string }> = {
  info: { icon: Info, frame: TONES.quiet.frame, mark: TONES.work.text },
  warn: { icon: TriangleAlert, frame: TONES.warn.frame, mark: TONES.warn.text },
  danger: { icon: CircleAlert, frame: TONES.danger.frame, mark: TONES.danger.text },
};

export function Callout({
  tone = "info",
  title,
  children,
  action,
}: {
  tone?: Tone;
  title: string;
  children?: ReactNode;
  action?: ReactNode;
}) {
  const { icon: Icon, frame, mark } = tones[tone];
  return (
    <div
      role={tone === "info" ? "status" : "alert"}
      className={`flex items-start gap-3 rounded-xl border px-3.5 py-3 ${frame}`}
    >
      <Icon aria-hidden size={16} className={`mt-0.5 shrink-0 ${mark}`} />
      <div className="min-w-0 flex-1 text-sm">
        <p className="font-semibold">{title}</p>
        {children && <div className="mt-0.5 text-ink-soft">{children}</div>}
      </div>
      {action && <div className="shrink-0">{action}</div>}
    </div>
  );
}
