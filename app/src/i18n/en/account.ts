// The account area at the bottom of the sidebar: who the owner is, the
// plan's usage, settings, language, what's new and help.

export const account = {
  open: (name: string) => `${name}: account and settings`,
  plan: (plan: string) => `${plan} plan`,
  notSignedIn: "Not signed in to Claude",
  menu: {
    usage: "Usage",
    settings: "Settings",
    language: "Language",
    whatsNew: "What's new",
    help: "Help",
    openFailed: "Could not open the page",
  },
  usage: {
    title: "Usage",
    intro: "How much of your Claude plan your bots have used. It updates whenever a bot works.",
    empty: "Usage shows up after a bot's first reply.",
    windows: {
      five_hour: "5-hour window",
      seven_day: "This week",
      seven_day_opus: "This week, Opus",
      seven_day_sonnet: "This week, Sonnet",
    } as Record<string, string>,
    used: (percent: number) => `${percent}% used`,
    resets: (relative: string) => `Resets ${relative}`,
    limited: "Limit reached. Your bots wait until it resets.",
    updated: (relative: string) => `Updated ${relative}`,
    tokens: {
      title: "Use by bot",
      intro:
        "How much of your weekly plan each bot used, and the tokens behind it: the pieces of text a bot reads and writes.",
      period: "Period",
      periods: {
        hour: "Last hour",
        today: "Today",
        week: "7 days",
        month: "30 days",
        all: "All time",
      },
      empty: "No bot worked in this period.",
      failed: "Could not load the tokens",
      archived: "archived",
      /** `crew` is null on the total of all bots. */
      detail: (crew: string | null, times: number) =>
        `${crew === null ? "" : `${crew} · `}Worked ${times === 1 ? "once" : `${times} times`}`,
      total: "All bots",
      share: (percent: string) => `≈ ${percent} of the week`,
      tokensUsed: (count: string) => `${count} tokens`,
      estimate:
        "≈ Estimated: Botloft learns how much of your plan the bots' work takes from how the weekly plan rises while they work. What you use outside Botloft also raises it, so the bots may show a little more than they used.",
      learning:
        "Each bot's share of your weekly plan shows up once the plan has gone up a few points while the bots work. Until then, the tokens.",
    },
  },
  backup: {
    exportTitle: "Save a copy",
    exportIntro:
      "Keeps your crews, bots, chats and each bot's memory in one file locked with a passphrase, to bring back after a new install or on another computer. Work folders you chose outside Botloft are not in it.",
    passphrase: "Passphrase",
    passphraseHint:
      "At least 8 characters. Without it the copy cannot be opened, and Botloft does not keep it anywhere.",
    repeat: "Passphrase again",
    mismatch: "The two passphrases are not the same.",
    export: "Save a copy…",
    exporting: "Making the copy…",
    saved: (size: string) => `Copy saved (${size}).`,
    notSaved: "The copy was not saved.",
    exportFailed: "Could not make the copy",
    importTitle: "Restore a copy",
    importIntro:
      "Replaces everything in this Botloft with a copy. What is here now goes to a folder aside: nothing is deleted.",
    pick: "Choose a copy…",
    pickTitle: "Choose a Botloft copy",
    kind: "Botloft copy",
    copyPassphrase: "The copy's passphrase",
    open: "Open",
    opening: "Opening…",
    from: (when: string) => `Copy from ${when}`,
    crew: (name: string, bots: number) =>
      `${name}: ${bots === 0 ? "no bots" : bots === 1 ? "1 bot" : `${bots} bots`}`,
    workFolders:
      "These work folders are not in the copy. Put their files back yourself if this is another computer:",
    replaces: "Everything in this Botloft is replaced. Botloft restarts to do it.",
    restore: "Restore and restart",
    restoring: "Restarting…",
    cancel: "Cancel",
    openFailed: "Could not open the copy",
    restoreFailed: "Could not restore the copy",
    reasons: {
      short_passphrase: "The passphrase needs at least 8 characters.",
      wrong_passphrase: "The passphrase is wrong, or the file is damaged.",
      not_a_backup: "This file is not a Botloft copy.",
      newer_backup: "This copy is from a newer Botloft. Update Botloft first.",
    } as Record<string, string>,
  },
  settings: {
    title: "Settings",
    background: "In the background",
    keepWorking: "Keep working after you close Botloft",
    keepWorkingOn: "Your bots go on working and answering after you close this window.",
    keepWorkingOff:
      "Closing Botloft stops every bot. They pick up where they left off when you open it again.",
    tray: "Show Botloft near the clock",
    trayOn:
      "With the window closed, its icon stays near the clock: a click opens Botloft, and it tells you when a bot needs you.",
    trayOff:
      "Closing the window closes Botloft. The bots keep working, with no icon or notifications.",
    startWithSystem: (system: string) => `Start with ${system}`,
    startWithSystemOn: (system: string) =>
      `When you sign in to ${system}, your bots get back to work on their own, without opening this window.`,
    startWithSystemOff: "After you restart the computer, the bots wait until you open Botloft.",
    openAtSignIn: (system: string) => `Open the window when you sign in to ${system}`,
    openAtSignInOn: (system: string) => `Botloft's window opens when you sign in to ${system}.`,
    openAtSignInNearClock: "Botloft starts near the clock, without opening the window.",
    openAtSignInOff: "The window opens only when you open Botloft.",
    alerts: "Notifications",
    notifyNeeds: "When a bot needs you",
    notifyNeedsHint: (system: string) =>
      `A ${system} notification when a bot asks for permission or needs you to sign in, if Botloft is not in front.`,
    notifyDone: "When a bot finishes",
    notifyDoneHint: "A notification when a bot finishes what it was doing.",
    markReplies: "Mark the icon when a bot replies",
    markRepliesHint: "A dot on the Botloft icon in the taskbar while a conversation is unread.",
    sound: "Play a sound",
    soundHint: "A short sound with each notification, also with Botloft in front.",
    appSounds: "App sounds",
    appSoundsHint:
      "Soft sounds while Botloft is in front: a message sent, a reply or a file in the open chat, a new bot or crew.",
    alertsNeedTray:
      "With the window closed, notifications only come with Botloft near the clock (in General).",
    keepAwake: "Keep the computer awake while bots work",
    keepAwakeHint: "It still sleeps when you close the lid or choose Sleep.",
    saveFailed: "Could not change the setting",
    pages: "Parts of Settings",
    general: "General",
    chat: "Chat",
    enterSends: "Enter sends the message",
    enterSendsOn: "Shift+Enter starts a new line.",
    enterSendsOff: "Enter starts a new line, and Ctrl+Enter sends.",
    followBot: "Open the browser and screens when a bot starts using them",
    followBotOn: "The panel opens beside the chat, so you see what the bot does.",
    followBotOff: "The panel's button gets a dot, and you open it when you want.",
    approvalWait: "How long a request for permission waits for you",
    approvalWaitHint:
      "With no answer by then, the bot's request is denied and it goes on without it.",
    waitFor: (minutes: number) =>
      minutes < 60 || minutes % 60 !== 0
        ? `${minutes} min`
        : minutes === 60
          ? "1 hour"
          : `${minutes / 60} hours`,
    lessMotion: "Less motion",
    lessMotionHint: (system: string) =>
      `A still window, as when the ${system} animation effects are off.`,
    appearance: "Appearance",
    theme: "Theme",
    themes: { system: "System", light: "Light", dark: "Dark" },
    size: "Size",
    sizeHint: "Ctrl+= and Ctrl+- change the size too, and Ctrl+0 goes back to the default.",
    language: "Language",
    archived: "Archived",
    archivedIntro:
      "Bots and crews you archived are stopped and out of sight, and Botloft still keeps their conversations. Delete one to remove it for good.",
    archivedEmpty: "Nothing is archived.",
    archivedLoadFailed: "Could not load what is archived",
    /** `when` is relative, like "2 days ago". */
    archivedCrew: (bots: number, when: string) =>
      `Crew · ${bots === 1 ? "1 bot" : `${bots} bots`} · archived ${when}`,
    archivedBot: (crew: string, when: string) => `Bot in ${crew} · archived ${when}`,
    deleteArchived: "Delete",
    deleteArchivedOne: (name: string) => `Delete ${name}`,
    about: "About",
    backup: "Backup",
    tools: "Connected tools",
    claudeCode: (version: string) => `Claude Code ${version}`,
    botloft: (version: string) => `Botloft ${version}`,
    checkUpdates: "Check for updates",
    checking: "Checking…",
    upToDate: "You have the latest version.",
    updateFound: (version: string) => `Botloft ${version} is out.`,
    seeUpdate: "See the update",
    checkFailed: "Could not check for updates. Try again later.",
  },
};
