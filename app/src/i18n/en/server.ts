// The app on a server instead of this computer (spec 42.6): the key, the
// tunnel and the devices that reach the agents. Plain words: "server" and
// "tunnel" are what the owner set up, "daemon" stays out.

export const server = {
  connecting: (name: string) => `Connecting to ${name}…`,
  /** Stands for the server's address in the tunnel command. */
  address: "you@your-server",
  copy: "Copy",
  copied: "Copied",
  sending: (name: string) => `Sending ${name} to the server…`,
  sendFailed: "Couldn't send the file",
  folder: "Folder on the server",
  folderHint:
    "A folder of the server's, as its path there. Leave it empty for a new one inside Botloft. For code, ask the crew to clone the repository into its folder.",
  useThisComputer: "Use this computer instead",
  unreachable: {
    title: (name: string) => `Can't reach ${name}`,
    body: "This app reaches the server through a tunnel that starts on this computer. Open it in a terminal and leave it open:",
    retrying: "Botloft keeps trying and connects as soon as the tunnel is open.",
  },
  refused: {
    title: (name: string) => `${name} didn't let this computer in`,
    body: "Its key may have been removed there. On the server, run sudo botloftd pair and paste the new key, or go back to this computer.",
  },
  /** The app's own tunnel to the server (spec 42.6), when the key says how. */
  own: {
    found: (host: string) =>
      `This computer connects to it by itself, at ${host}. Nothing to open in a terminal.`,
    address: "Server address",
    addressHint:
      "Where this computer reaches the server: an IP address or a name. Change it if you reach it another way.",
    use: "Use this address",
    trouble: {
      unreachable: {
        title: (name: string) => `Can't reach ${name}`,
        body: (host: string) =>
          `Botloft couldn't open its connection to ${host}. Check that the server is on and that this address reaches it. Botloft keeps trying.`,
      },
      refused: {
        title: (name: string) => `${name} didn't accept this computer`,
        body: (_host: string) =>
          "The server no longer accepts this computer's key; it may have been removed there. On the server, run sudo botloftd pair and paste the new key.",
      },
      host_key: {
        title: (name: string) => `${name} isn't the server this key was made on`,
        body: (host: string) =>
          `The computer at ${host} answered as another one, so Botloft didn't connect. If the server was set up again, run sudo botloftd pair there and paste the new key.`,
      },
      no_ssh: {
        title: (name: string) => `To reach ${name}, this computer needs OpenSSH`,
        body: (_host: string) =>
          "Botloft opens its connection to the server with OpenSSH's client. On Windows, add it in Settings > System > Optional features > OpenSSH Client, then try again.",
      },
      failed: {
        title: (name: string) => `Couldn't connect to ${name}`,
        body: (_host: string) =>
          "Opening the connection went wrong; the details say how. Try again, or paste the server's key again.",
      },
    },
  },
  form: {
    title: "Connect to a server",
    intro:
      "Run Botloft on a server of yours and your agents keep working with this computer off. On the server, run sudo botloftd pair and paste the key it shows.",
    key: "Connection key",
    show: "Show the key",
    hide: "Hide the key",
    placeholder: "botloft1.…",
    invalid: "That isn't a Botloft key. Copy the whole line that starts with botloft1.",
    found: (name: string) => `Server: ${name}`,
    tunnel: "This computer reaches it through a tunnel. Open it in a terminal and leave it open:",
    connect: "Connect",
    move: "To take this computer's crews along: in Settings > Backup, save a copy here; once connected, restore it there. Then pause the crews here, or they run twice.",
  },
  where: {
    title: "Where your agents run",
    thisComputer: "On this computer",
    thisComputerHint: "Botloft runs here in the background.",
    onServer: (name: string) => `On the server ${name}`,
    onServerHint:
      "This app shows the agents running on the server. They keep working with this computer off.",
    toServer: "Connect to a server",
    toThisComputer: "Use this computer",
    toThisComputerTitle: "Use Botloft on this computer?",
    toThisComputerText:
      "The agents on the server keep running. This app shows the ones on this computer instead, and you can connect to the server again with the same key.",
  },
  devices: {
    title: "Devices",
    hint: "Computers that reach these agents with a key of their own.",
    none: "No other device has a key.",
    thisOne: "This computer",
    seen: (time: string) => `Last connected ${time}`,
    never: "Never connected",
    remove: "Remove",
    removeTitle: (name: string) => `Remove ${name}?`,
    removeText: "Its key stops working at once, and it is disconnected.",
    failed: "Couldn't remove it",
  },
};
