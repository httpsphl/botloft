import { useEffect, useState } from "react";
import { useStore } from "zustand";
import { type Connect, createLink } from "./features/onboarding/link";
import { Onboarding } from "./features/onboarding/Onboarding";
import type { Host } from "./lib/host";
import { TitleBar } from "./shell/TitleBar";
import { useThemeRoot } from "./shell/theme";
import { Workspace } from "./shell/Workspace";
import { useZoomRoot } from "./shell/zoom";
import { DaemonProvider, HostProvider } from "./store/context";
import { Toaster } from "./ui/toast";

export function App({ host, connect }: { host: Host; connect: Connect }) {
  useThemeRoot();
  useZoomRoot(host);
  const [link] = useState(() => createLink(host, connect));
  const current = useStore(link, (state) => state.current);
  useEffect(() => {
    link.getState().check();
  }, [link]);

  return (
    <HostProvider host={host}>
      {current.step === "connected" ? (
        <DaemonProvider api={current.api}>
          <Workspace />
        </DaemonProvider>
      ) : (
        <div className="flex h-full flex-col">
          <TitleBar />
          <Onboarding link={link} />
        </div>
      )}
      <Toaster />
    </HostProvider>
  );
}
