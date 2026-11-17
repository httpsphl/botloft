// The Overview button's icon (spec 33.2): a list in a rounded square, each
// line with its dot, drawn like the lucide icons around it.

import { createLucideIcon } from "lucide-react";

export const OverviewIcon = createLucideIcon("overview", [
  ["rect", { width: "18", height: "18", x: "3", y: "3", rx: "3", key: "frame" }],
  ["path", { d: "M7.5 8h.01", key: "dot-1" }],
  ["path", { d: "M11 8h5.5", key: "line-1" }],
  ["path", { d: "M7.5 12h.01", key: "dot-2" }],
  ["path", { d: "M11 12h5.5", key: "line-2" }],
  ["path", { d: "M7.5 16h.01", key: "dot-3" }],
  ["path", { d: "M11 16h5.5", key: "line-3" }],
]);
