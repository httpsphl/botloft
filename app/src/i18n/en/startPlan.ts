// The plan a bot shows before long work (spec 10.5).

export const startPlan = {
  ready: (bot: string) => `${bot} will start like this`,
  alternative: "Or a smaller way",
  startsIn: (seconds: number) => `Starts by itself in ${seconds} s`,
  starting: "Starting…",
  startNow: "Start now",
  onlyThis: "Do only this",
  change: "Change",
  sendChange: "Send",
  noteLabel: (bot: string) => `What ${bot} should do instead`,
  notePlaceholder: "What should change?",
  cancel: "Cancel",
  failed: "Could not answer the plan",
  started: "You said start",
  startedAlone: "Started by itself",
  didAlternative: "Doing only the smaller way",
  changed: "You asked for changes",
  cancelled: "You cancelled it",
};
