import { CircleAlert, Info, TriangleAlert } from "lucide-react";
import type { ReactNode } from "react";

type Tone = "info" | "warn" | "danger";

const tones: Record<Tone, { icon: typeof Info; frame: string; mark: string }> = {
  info: { icon: Info, frame: "border-line-strong", mark: "text-work" },
  warn: { icon: TriangleAlert, frame: "border-warn/60", mark: "text-warn" },
  danger: { icon: CircleAlert, frame: "border-danger/60", mark: "text-danger" },
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
      className={`flex items-start gap-3 border border-l-4 bg-panel px-3 py-2.5 ${frame}`}
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
