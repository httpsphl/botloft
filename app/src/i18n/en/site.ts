// The public site (spec 39), one page per language made when it is built.
// It speaks to people who have not met Botloft yet: plain words, each
// agent is an "agent" and what runs it is the "engine".

export const site = {
  meta: {
    title: "Botloft: your AI agents, always on, working together",
    description:
      "A free desktop app that keeps a crew of AI agents running on your computer, on Claude Code. They pass work to each other, keep going after you close the window and ask before they act.",
  },
  skip: "Skip to the content",
  nav: {
    label: "Sections",
    features: "Features",
    how: "Get started",
    privacy: "Privacy",
    faq: "Questions",
    github: "GitHub",
    language: "Language",
  },
  download: {
    windows: "Download for Windows",
    macos: "Download for macOS",
    linux: "Download for Linux",
    phone: "Get it on your computer",
    short: "Download",
    all: "All downloads",
    windowsFile: "Windows 10 or 11",
    macosFile: "macOS, Apple Silicon (preview)",
    debFile: "Linux .deb (preview)",
    appImageFile: "Linux AppImage (preview)",
  },
  hero: {
    badge: "Free · Windows, Linux and macOS",
    title: "Your AI agents, always on, working together.",
    lead: "Botloft keeps a crew of agents running on your computer, each on Claude Code. They pass work to each other, keep going after you close the window and ask you before they act.",
    github: "Code on GitHub",
    needs: "You need Claude Code, signed in with your own plan.",
    needsLink: "How to install it",
    shot: "Botloft with a research crew open: seven agents, each with its own color, state and job",
  },
  features: {
    title: "A crew that works while you do something else",
    lead: "Each agent has its own folder, memory and conversation. You talk to each one like in a chat app, and you decide what they may do.",
    items: {
      crew: {
        title: "A crew for every goal",
        body: "Say what the crew is for. Its Chief plans the work and suggests the agents it needs; you add each one with a click.",
      },
      agency: {
        title: "Dozens of ready-made agents",
        body: "A developer, a designer, a writer, a researcher, customer support: pick one for a job and change anything about it.",
      },
      maker: {
        title: "The Maker builds for you",
        body: "Ask for crews, agents and routines in your own words. You see the whole plan before anything is made, and Undo takes it back.",
      },
      chat: {
        title: "Chat with every agent",
        body: "Replies stream in as they are written. Send images and files, search every chat and open each edit an agent made.",
      },
      ask: {
        title: "You decide what they may do",
        body: "Before an agent runs a command, edits a file or opens a website, a card asks you first, unless you told it it may.",
      },
      handoff: {
        title: "They hand work to each other",
        body: "Agents send each other messages and tasks. Every message is saved before it is sent and retried until it arrives.",
      },
      always: {
        title: "Always on",
        body: "Agents keep working after you close Botloft and start when you sign in. Idle ones sleep, so they use no memory.",
      },
      routines: {
        title: "Routines on a schedule",
        body: "Weekday mornings, every two hours or any schedule you like. Or make a routine a shortcut you run with one click.",
      },
      phone: {
        title: "From your phone",
        body: "Approve requests, answer questions and chat from anywhere. What passes between the phone and your computer is sealed end to end.",
      },
    },
  },
  shots: {
    title: "See them at work",
    items: {
      browser: {
        title: "Chat and live browser",
        body: "Scout asks before using a new site, and you watch it browse.",
        alt: "Scout's chat with a request to use the browser, next to the live browser where Scout types in a search box",
      },
      screens: {
        title: "Screens",
        body: "Designer's pages take shape while it writes them.",
        alt: "Designer writing a bakery landing page that takes shape in the screens area",
      },
      chief: {
        title: "A Chief for each crew",
        body: "It suggests the agents the work needs.",
        alt: "The Chief suggesting a new agent, Designer, with its name, model, role and instructions ready to edit",
      },
      routines: {
        title: "Routines",
        body: "Work that runs on a schedule, with the last result at a glance.",
        alt: "A crew's routines: a morning summary on weekdays, an inbox check run with a click, a one-time reminder, a link check every two hours and a source recheck that waits for another agent's signal",
      },
    },
    note: "The pictures show sample crews, not real data.",
  },
  steps: {
    title: "Up and running in three steps",
    items: {
      engine: {
        title: "Install Claude Code",
        body: "Install it and sign in once with your own account. Your agents run on your plan.",
        link: "Claude Code setup",
      },
      app: {
        title: "Download Botloft",
        body: "Open it and it sets itself up: there is nothing to configure. It updates itself, too.",
      },
      crew: {
        title: "Create your first crew",
        body: "Tell it what it is for and meet the agents it suggests.",
      },
    },
  },
  engines: {
    title: "Choose the engine",
    lead: "Each agent runs on the AI coding engine you pick for it, installed on your computer and signed in with your own account.",
    claude: {
      title: "Claude Code",
      tag: "Full experience",
      body: "Every feature: asks before each tool, with “always allow”, the live browser, routines and how much of your plan is left.",
    },
    others: {
      title: "Antigravity and Codex",
      tag: "Experimental",
      body: "Off until you turn them on in Settings. They chat, take tasks and talk to the crew; some features are not checked on them yet.",
    },
  },
  privacy: {
    title: "Yours, on your computer",
    lead: "Botloft runs on your computer. An account is optional, and nothing leaves without you asking.",
    items: {
      local: {
        title: "No telemetry",
        body: "Without an account, Botloft uses the network only for updates and your agents' own work.",
      },
      sealed: {
        title: "Sealed copies",
        body: "The optional cloud copy is sealed with a password that never leaves your computer. The server cannot open it.",
      },
      keys: {
        title: "Keys stay out of the chat",
        body: "When an agent needs a password or a token, you type it in a hidden field. It never goes into the chat.",
      },
      code: {
        title: "Public code",
        body: "Read every line on GitHub. Each version also becomes Apache 2.0 two years after it ships.",
      },
    },
  },
  faq: {
    title: "Questions",
    items: {
      cost: {
        q: "Does Botloft cost anything?",
        a: "No. Botloft is free, at home or at work. The agents run on the engine you choose with your own account, so they use your plan's limits; with Claude Code, Botloft shows how much is left.",
      },
      window: {
        q: "Do I need to keep the window open?",
        a: "No. The agents run in the background, even after you close Botloft, and start when you sign in to your computer.",
      },
      systems: {
        q: "Which computers does it run on?",
        a: "Windows 10 and 11 (64-bit). Linux (64-bit, .deb or AppImage) and macOS on Apple Silicon run it as a preview; Intel Macs are not supported yet.",
      },
      warning: {
        q: "Why does my computer warn me when I install it?",
        a: "The Windows installer is not code-signed yet, so SmartScreen does not know its publisher: choose “More info”, then “Run anyway”. Apple has not notarized the macOS app either: allow it once in System Settings, Privacy & Security. Each release is built by GitHub from the public code, and updates are signed and checked before they install.",
      },
      anthropic: {
        q: "Is Botloft made by Anthropic?",
        a: "No. Botloft is an independent project. It uses Claude Code, Antigravity and Codex as installed on your computer and is not affiliated with or endorsed by Anthropic, Google or OpenAI.",
      },
    },
  },
  closing: {
    title: "Give your agents a place to work.",
    body: "Free, and ready in a couple of minutes.",
  },
  footer: {
    links: "Links",
    tagline: "Your AI agents, always on, working together.",
    releases: "Releases",
    license: "License (FSL-1.1-ALv2)",
    signing: "Code signing policy",
    security: "Security",
    account: "What the account keeps",
    notAffiliated: "Not affiliated with Anthropic, Google or OpenAI.",
  },
};

export type SiteMessages = typeof site;
