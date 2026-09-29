// Words shared by many screens, and errors of the connection itself.

export const common = {
  cancel: "Cancel",
  close: "Close",
  dismiss: "Dismiss",
  details: "Details",
  tryAgain: "Try again",
  errors: {
    closed: "the connection was closed",
    notConnected: "not connected to Botloft",
    connectionLost: "lost the connection to Botloft",
    timedOut: (method: string) => `${method} timed out`,
  },
};
