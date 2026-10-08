// The phone's page (spec 28.7): connecting it, and approving and answering
// from it. What the bots ask is data and never passes through here.

export const phone = {
  pair: {
    title: "Connect this phone",
    intro:
      "This phone will approve your bots' requests and answer their questions, from anywhere. Chats and files stay on your computer.",
    nameLabel: "Name of this phone",
    connect: "Connect",
    connecting: "Connecting…",
    compare: (code: string) =>
      `Check that your computer shows ${code} too. Then press Connect on the computer.`,
    waiting: "Waiting for you to accept on the computer…",
    notALink:
      "To connect a phone, point its camera at the code on your computer: Settings, Backup, Phone.",
    expired: "The code ran out or was already used. Ask for a new one on the computer.",
    offline: "Could not reach Botloft. Check the connection and try again.",
    failed: "Could not connect. Ask for a new code on the computer.",
    names: { iphone: "iPhone", ipad: "iPad", android: "Android phone", other: "Phone" },
  },
  inbox: {
    title: "Waiting for you",
    empty: "Nothing is waiting for you",
    emptyBody: "When a bot needs you, it shows up here.",
    loading: "Checking what is waiting…",
    computerOff: "Your computer is off or has no internet. Requests keep waiting there.",
    noConnection: "No connection. Trying again…",
    thisPhone: "This phone",
    inCrew: (crew: string) => `in ${crew}`,
  },
  approval: {
    asks: (bot: string) => `${bot} asks to`,
    explanationBy: (bot: string) => `${bot} wrote this`,
    noExplanation: (bot: string) =>
      `${bot} did not say what this is for. Consider denying it and asking.`,
    showAll: "Show the whole request",
    hideAll: "Hide it",
    allow: "Allow",
    deny: "Deny",
    sending: "Sending…",
    noteLabel: "Note for the bot (optional)",
    cut: "Too long to read on a phone. You can deny it here, or answer it at the computer.",
    atComputer: "This one needs you at the computer. Here you can only deny it.",
    failed: "Not sent. Check the connection and try again.",
    ended: { allowed: "Allowed", denied: "Denied", expired: "Ran out" },
  },
  question: {
    asks: (bot: string) => `${bot} asks`,
    pick: "Pick an answer",
    answerLabel: "Your answer",
    placeholder: "Write your answer",
    send: "Answer",
    sending: "Sending…",
    dismiss: "Dismiss",
    dismissHint: (bot: string) =>
      `Closes the question without telling ${bot}. To let it know, answer instead.`,
    failed: "Not sent. Check the connection and try again.",
    ended: { answered: "You answered", dismissed: "Dismissed" },
  },
  settings: {
    notices: {
      turnOn: "Turn on notices",
      turnOff: "Turn off notices",
      on: "Notices are on. You get one when a bot needs you.",
      off: "Notices are off.",
      blocked: "Notices are blocked for this page. Allow them in the browser's settings.",
      unsupported: "This browser cannot give notices.",
    },
    title: "This phone",
    back: "Back",
    name: "Name",
    disconnect: "Disconnect this phone",
    disconnectTitle: "Disconnect this phone?",
    disconnectText:
      "It stops receiving requests. To use it again, connect it anew on your computer.",
    cancel: "Cancel",
    install: "To get notices, add Botloft to your Home Screen: tap Share, then Add to Home Screen.",
  },
  cut: {
    revokedTitle: "This phone was disconnected",
    leftTitle: "Phone disconnected",
    body: "To use it again, connect it from Botloft on your computer: Settings, Backup, Phone.",
    ok: "OK",
  },
};
