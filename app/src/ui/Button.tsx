import type { LucideIcon } from "lucide-react";
import type { ButtonHTMLAttributes } from "react";

export type ButtonVariant = "primary" | "secondary" | "ghost" | "danger";

const variants: Record<ButtonVariant, string> = {
  primary: "bg-ink text-canvas hover:bg-ink-soft border border-ink",
  secondary: "border border-line-strong bg-panel text-ink hover:bg-sunken",
  ghost: "border border-transparent text-ink-soft hover:bg-sunken hover:text-ink",
  danger: "border border-danger/60 bg-panel text-danger hover:bg-danger/10",
};

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  size?: "sm" | "md";
  icon?: LucideIcon;
  /** Shows only the icon; `label` becomes its accessible name and tooltip. */
  label?: string;
}

export function Button({
  variant = "secondary",
  size = "md",
  icon: Icon,
  label,
  className = "",
  children,
  type = "button",
  ...rest
}: ButtonProps) {
  const height = size === "sm" ? "h-7 text-sm" : "h-8 text-sm";
  const padding = children ? (size === "sm" ? "px-2" : "px-3") : size === "sm" ? "w-7" : "w-8";
  return (
    <button
      type={type}
      aria-label={label}
      title={label}
      className={`inline-flex shrink-0 items-center justify-center gap-1.5 rounded-lg font-medium transition-[color,background-color,border-color,transform] duration-150 active:scale-[0.97] disabled:pointer-events-none disabled:opacity-45 ${height} ${padding} ${variants[variant]} ${className}`}
      {...rest}
    >
      {Icon && <Icon aria-hidden size={size === "sm" ? 14 : 15} strokeWidth={2} />}
      {children}
    </button>
  );
}
