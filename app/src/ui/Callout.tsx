import { CircleAlert, Info, TriangleAlert } from "lucide-react";
import type { ReactNode } from "react";

type Tone = "info" | "warn" | "danger";

const tones: Record<Tone, { icon: typeof Info; frame: string; mark: string }> = {
  info: { icon: Info, frame: "border-line bg-panel", mark: "text-work" },
  warn: { icon: TriangleAlert, frame: "border-warn/45 bg-warn/6", mark: "text-warn" },
  danger: { icon: CircleAlert, frame: "border-danger/45 bg-danger/6", mark: "text-danger" },
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
