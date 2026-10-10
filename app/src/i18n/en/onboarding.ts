// Opening the app, setting it up and the welcome screen. Plain words only:
// no "daemon", "port" or "scheduled task" outside the folded Details.

export const onboarding = {
  tagline: "Always-on Claude Code crews.",
  opening: "Opening Botloft…",
  connecting: "Connecting…",
  restartInBackground: "Restart Botloft in the background",
  installing: {
    install: "Getting Botloft ready…",
    update: "Updating Botloft…",
    restart: "Restarting Botloft…",
    start: "Starting your agents…",
  },
  installNote: (system: string) =>
    `Botloft runs your agents in the background. In Settings you choose whether they keep working after you close this window and whether Botloft starts with ${system}.`,
  stopped: {
    title: "Botloft couldn't start",
    body: "Botloft runs your agents in the background, and that part didn't start.",
    notRunning: "It is not running.",
  },
  outdated: {
    title: "Botloft couldn't finish updating",
    body: "The part that runs your agents in the background is still on the previous version.",
    running: (version: string) => `Running version ${version}.`,
    stillOld: (version: string) => `It still reports version ${version}.`,
  },
  late: "It started but did not answer in time. Its log is in the logs folder.",
  foreign: {
    title: "Another program is in the way",
    body: (port: number) =>
      `Botloft needs port ${port} on this computer, and another program is using it. Close that program and try again.`,
    detail: (port: number) =>
      `127.0.0.1:${port} answers but is not Botloft. To use another port, set "port" in Botloft's config.toml.`,
  },
  mismatch: {
    title: "This app doesn't match the Botloft that is running",
    body: "The Botloft running in the background is newer than this app. Install the latest version of Botloft.",
    detail: (version: string, theirs: number, ours: number) =>
      `Running version ${version}, protocol ${theirs}. This app speaks protocol ${ours}.`,
  },
  cantConnect: {
    title: "Botloft couldn't connect",
    body: "Try again. If this keeps happening, reinstall Botloft.",
  },
  welcome: {
    title: "Welcome to Botloft",
    intro:
      "A crew is a group of agents that keep running, message each other and share a folder. Each agent runs on the engine you pick: Claude Code, or another one you turn on. Each crew starts with a chief: tell it what the crew is for, and it plans the work and suggests the agents it needs.",
    botloft: "Botloft",
    running: (system: string) =>
      `Running in the background. It starts with ${system}, so your agents keep working after you close this window.`,
    runningNotAtStart:
      "Running in the background, so your agents keep working after you close this window.",
    runningWhileOpen: "Running while this window is open. Closing it stops your agents.",
    claudeCode: "Claude Code",
    checking: "Checking…",
    version: (version: string) => `Version ${version}`,
    account: "Claude account",
    signedIn: "Signed in.",
    signedOut: "Your agents work with your Claude account. Sign in once and they are ready.",
    ready: "ready",
    notReady: "not ready",
    stillChecking: "checking",
    askFirstTitle: "Agents ask before they change things",
    askFirstBody:
      "When an agent wants to run a command or edit a file, it asks in its chat and waits for you to allow or deny it.",
    ideasTitle: "Or tap an idea",
    claudeOptional: "Not installed. You only need it for agents that run on Claude Code.",
    agentMissing: "Not found. Install it and open Botloft again.",
    notNeeded: "not needed",
    createCrew: "Create your first crew",
  },
  claudeCode: {
    help: "Botloft runs your agents with Claude Code. Install it or update it, open it once to sign in, and Botloft picks it up within 30 seconds.",
    install: "How to install Claude Code",
    openFailed: "Could not open the link",
  },
  signIn: {
    button: "Sign in to Claude",
    waiting: "Waiting for you to sign in…",
    waitingNote:
      "A window opened with Claude's sign-in. Finish it in your browser; Botloft continues by itself.",
    failedTitle: "The sign-in didn't finish",
    failedBody: "Try again, and finish the sign-in in the browser before closing its window.",
  },
};
