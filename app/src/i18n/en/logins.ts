// Shared site logins (spec 35): the owner signs in to a site once and
// chooses which agents use the login. Plain words: "sign in", "login".

export const logins = {
  title: "Logins",
  intro:
    "Sign in to a site once and choose which agents use that login. Whoever you choose acts on the site as you.",
  signIn: "Sign in to a site",
  address: "Site address",
  placeholder: "reddit.com",
  open: "Open window",
  inWindow: (site: string) =>
    `Sign in to ${site} in the window that opened, then close it. The login is saved when the window closes.`,
  signedIn: (site: string) => `Signed in to ${site}. Choose who can use it.`,
  nothing: (site: string) => `No login was found on ${site}. Sign in before you close the window.`,
  none: "No login yet. Sign in to a site to give it to your agents.",
  list: "Your logins",
  signedAt: (when: string) => `Signed in ${when}`,
  agents: (count: number) => (count === 1 ? "1 agent" : `${count} agents`),
  noAgent: "No agent yet",
  who: "Who can use it",
  warning:
    "Whoever you choose opens the site already in your account. They read freely and ask you before they post, send, buy or change a setting.",
  canUse: (bot: string) => `${bot} can use it`,
  noAgents: "No agents yet.",
  again: "Sign in again",
  remove: "Remove login",
  removing: {
    title: (site: string) => `Remove the login to ${site}?`,
    text: "Every agent that has it is signed out of the site.",
    confirm: "Remove",
  },
  failed: "Could not change who uses the login",
  openFailed: "Could not open the window",
  removeFailed: "Could not remove the login",
};
