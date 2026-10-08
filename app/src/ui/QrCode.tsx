// A QR code drawn as an image, made here: nothing about what it holds leaves
// the computer. Black on white whatever the theme, because scanners need it.

import qrcode from "qrcode-generator";
import { useMemo } from "react";

/** One path for every dark square, row by row. */
function draw(text: string): { path: string; size: number } {
  const code = qrcode(0, "M");
  code.addData(text);
  code.make();
  const count = code.getModuleCount();
  let path = "";
  for (let row = 0; row < count; row++) {
    let column = 0;
    while (column < count) {
      if (!code.isDark(row, column)) {
        column++;
        continue;
      }
      const start = column;
      while (column < count && code.isDark(row, column)) {
        column++;
      }
      path += `M${start} ${row}h${column - start}v1h-${column - start}z`;
    }
  }
  return { path, size: count };
}

/** `label` says what the code is, for whoever cannot see it. */
export function QrCode({ text, label }: { text: string; label: string }) {
  const { path, size } = useMemo(() => draw(text), [text]);
  const margin = 4;
  return (
    <svg
      role="img"
      aria-label={label}
      viewBox={`${-margin} ${-margin} ${size + margin * 2} ${size + margin * 2}`}
      shapeRendering="crispEdges"
      className="h-60 w-60 rounded-md"
    >
      <rect
        x={-margin}
        y={-margin}
        width={size + margin * 2}
        height={size + margin * 2}
        fill="#fff"
      />
      <path d={path} fill="#000" />
    </svg>
  );
}
