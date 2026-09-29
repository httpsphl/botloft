// The Botloft mascot in the bot's own color (spec 15.3). The color only
// tells bots apart; state is shown elsewhere with an icon and a label.
//
// A bot's avatar has no background: just the mascot, with a thin edge so
// light colors keep their shape on the light theme. `framed` puts the
// mascot on the black square of the app icon, for Botloft itself (the
// title bar, messages from the daemon), whose white mascot needs it.

const BODY =
  "M468 58C600 105 730 180 800 290C830 340 845 400 870 440C885 465 915 475 930 455C948 430 945 400 938 378C1060 440 1170 600 1210 800C1240 950 1235 1120 1190 1254L70 1254L60 1254C10 1130 0 950 60 800C100 700 150 650 180 590C210 520 220 440 260 385C290 340 330 312 372 292C350 330 340 370 348 390C355 410 380 412 395 398C440 355 500 300 510 220C518 150 495 100 468 58Z";

export function BotAvatar({
  color,
  size = 28,
  framed = false,
}: {
  color: string;
  size?: number;
  framed?: boolean;
}) {
  return (
    <svg
      aria-hidden
      width={size}
      height={size}
      // A little room around the body, so its edge is not cut at the sides.
      viewBox={framed ? "0 0 1254 1254" : "-24 -24 1302 1302"}
      className={`shrink-0 ${framed ? "rounded-lg bg-[#0b0b0b]" : ""}`}
    >
      <path
        d={BODY}
        fill={color}
        stroke={framed ? undefined : "var(--avatar-edge)"}
        strokeWidth={framed ? undefined : 1}
        vectorEffect="non-scaling-stroke"
      />
      <ellipse cx="537" cy="862" rx="112" ry="110" fill="#0b0b0b" />
      <ellipse cx="546" cy="811" rx="33" ry="29" fill="#ffffff" />
      <ellipse cx="945" cy="768" rx="98" ry="104" transform="rotate(-14 945 768)" fill="#0b0b0b" />
      <ellipse cx="934" cy="720" rx="30" ry="26" fill="#ffffff" />
    </svg>
  );
}
