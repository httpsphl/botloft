// One screen on its own (spec 22.5): the width of the panel and alive, to
// scroll and click, with its device, Open and Show in folder.

import { ArrowLeft, ExternalLink, FolderOpen, Monitor, Smartphone, Tablet } from "lucide-react";
import { useRef } from "react";
import { useT } from "../../i18n";
import type { Bot, Screen, ScreenDevice } from "../../lib/protocol.gen";
import { useHost } from "../../store/context";
import { Button } from "../../ui/Button";
import { attempt } from "../../ui/toast";
import { BotAvatar } from "../bots/BotAvatar";
import { DEVICES, LiveFrame } from "./LiveFrame";
import { useWidth } from "./prefs";

const ICONS = { desktop: Monitor, tablet: Tablet, mobile: Smartphone } as const;

export function ScreenFocus({
  bot,
  screen,
  device,
  onDevice,
  onBack,
}: {
  bot: Bot;
  screen: Screen;
  device: ScreenDevice;
  onDevice(device: ScreenDevice): void;
  onBack(): void;
}) {
  const t = useT().screens;
  const host = useHost();
  const stage = useRef<HTMLDivElement>(null);
  const width = useWidth(stage);
  const scale = Math.min(1, (width - 32) / DEVICES[device].width);

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <div className="flex shrink-0 flex-wrap items-center gap-2 border-line border-b px-3 py-2">
        <Button variant="ghost" size="sm" icon={ArrowLeft} onClick={onBack}>
          {t.back}
        </Button>
        <span className="min-w-0 flex-1 truncate font-medium text-sm" title={screen.path}>
          {screen.name}
        </span>
        {screen.writing && (
          <span className="inline-flex items-center gap-1 text-xs" style={{ color: bot.color }}>
            <BotAvatar color={bot.color} size={14} mood="working" />
            {t.writing}
          </span>
        )}
        <fieldset className="flex rounded-lg bg-sunken p-0.5">
          <legend className="sr-only">{t.device}</legend>
          {(Object.keys(ICONS) as ScreenDevice[]).map((one) => {
            const Icon = ICONS[one];
            return (
              <button
                key={one}
                type="button"
                aria-pressed={one === device}
                title={t.devices[one]}
                aria-label={t.devices[one]}
                onClick={() => onDevice(one)}
                className={`grid size-7 place-items-center rounded-md transition-colors ${
                  one === device ? "bg-panel text-ink shadow-sm" : "text-muted hover:text-ink"
                }`}
              >
                <Icon aria-hidden size={14} />
              </button>
            );
          })}
        </fieldset>
        <Button
          variant="ghost"
          size="sm"
          icon={ExternalLink}
          label={t.openFile}
          onClick={() => attempt(t.failed.open, () => host.openFile(screen.path))}
        />
        <Button
          variant="ghost"
          size="sm"
          icon={FolderOpen}
          label={t.reveal}
          onClick={() => attempt(t.failed.reveal, () => host.revealFile(screen.path))}
        />
      </div>
      <div ref={stage} className="design-board min-h-0 flex-1 overflow-auto p-4">
        <div className="mx-auto w-fit overflow-hidden rounded-md shadow-lift ring-1 ring-line">
          <LiveFrame
            url={screen.url}
            device={device}
            scale={scale}
            title={screen.name}
            interactive
            writer={screen.writing ? bot : null}
          />
        </div>
      </div>
    </div>
  );
}
