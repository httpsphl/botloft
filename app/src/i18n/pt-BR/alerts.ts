// O ícone perto do relógio e os avisos do Windows (spec 15.1).

import type { Messages } from "../en";

export const alerts: Messages["alerts"] = {
  tray: {
    tooltip: (status) => `Botloft: ${status}`,
    working: (count) => (count === 1 ? "1 agente trabalhando" : `${count} agentes trabalhando`),
    needsYou: (bot) => `${bot} precisa de você`,
    waiting: "Algo está esperando por você",
    idle: "Nenhum agente trabalhando",
    open: "Abrir o Botloft",
    pause: "Pausar todas as equipes",
    resume: "Retomar todas as equipes",
    quit: "Sair do Botloft",
  },
  approval: (bot) => `${bot} pede sua permissão`,
  signIn: (bot) => `${bot} precisa que você entre na conta do Claude`,
  signInBody: "Abra o Botloft para entrar.",
  done: (bot) => `${bot} terminou`,
  crew: (crew) => `Equipe ${crew}`,
};
