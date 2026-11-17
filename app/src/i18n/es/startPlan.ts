// El plan que un agente muestra antes de un trabajo largo (spec 10.5).

import type { Messages } from "../en";

export const startPlan: Messages["chat"]["startPlan"] = {
  ready: (bot: string) => `${bot} va a empezar así`,
  alternative: "O una forma más pequeña",
  startsIn: (seconds: number) => `Empieza solo en ${seconds} s`,
  starting: "Empezando…",
  startNow: "Empezar ahora",
  onlyThis: "Hacer solo esto",
  change: "Cambiar",
  sendChange: "Enviar",
  noteLabel: (bot: string) => `Qué debe hacer ${bot} en su lugar`,
  notePlaceholder: "¿Qué debe cambiar?",
  cancel: "Cancelar",
  failed: "No se pudo responder al plan",
  started: "Dijiste que empiece",
  startedAlone: "Empezó solo",
  didAlternative: "Haciendo solo la forma más pequeña",
  changed: "Pediste cambios",
  cancelled: "Lo cancelaste",
};
