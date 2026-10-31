import type { Messages } from "../en";

export const permissions: Messages["permissions"] = {
  intro:
    "O que cada agente pode fazer sem te perguntar de novo. Tire o que não quiser mais: o agente pede de novo na próxima vez.",
  agent: "Agente",
  everyone: "Todos os agentes",
  tabs: {
    desktop: "Desktop",
    sites: "Sites",
    rules: "Comandos",
    crews: "Outras equipes",
  },
  empty: {
    desktop:
      "Nenhum agente pode ver apps do seu computador. Eles pedem no chat na primeira vez que precisam de um.",
    sites:
      "Nenhum site liberado de vez. Os agentes pedem no chat na primeira vez que abrem um no navegador deles.",
    rules: 'Nada liberado de vez. Quando um agente pede, "Permitir sempre" coloca aqui.',
    crews:
      'Cada agente só alcança a própria equipe. "Sempre" num pedido para alcançar outra coloca aqui.',
  },
  sitesModes: 'Nos modos "Automático" e "Sem perguntar", o agente usa qualquer site sem pedir.',
  allowedOn: (date) => `Liberado em ${date}`,
  siteRemove: (site) => `Tirar ${site}`,
  siteRemoveFailed: "Não deu para tirar",
  give: "Dar um app",
  giveTitle: "Dar um app a um agente",
  giveApp: "App",
  giveLevel: "Ele pode",
  giveRefresh: "Ler de novo os apps abertos",
  giveNoApps:
    "Nenhum app que pode ser dado está aberto. Abra o app no computador e leia a lista de novo.",
  giveHint: "A lista mostra só os apps abertos agora, nunca os que nunca são liberados.",
  giveConfirm: "Dar",
  giveFailed: "Não deu para dar",
  cancel: "Cancelar",
  stopAll: "Parar todos agora",
  stopAllHint:
    "Ou aperte Ctrl+Alt+Esc em qualquer lugar: todos os agentes param no seu desktop na hora.",
  stopAllFailed: "Não deu para parar",
  never: "O que nunca é liberado",
  neverItems: [
    "O próprio Botloft",
    "Apps rodando como administrador",
    "As telas de segurança do Windows: UAC, Ctrl+Alt+Del e a tela de bloqueio",
    "Terminais e caixas de comando: Prompt de Comando, PowerShell, Windows Terminal, WSL, Git Bash, a caixa Executar, o Editor do Registro e o Gerenciador de Tarefas",
    "Gerenciadores de senha e cofres: Gerenciador de Credenciais, Segurança do Windows, 1Password, Bitwarden, KeePass, LastPass e Dashlane",
    "Campos de senha, em qualquer app",
  ],
  section: "Permissões",
  seeAll: "Permissões de todos os agentes",
  sites: "Sites no navegador",
  sitesNone: (bot) =>
    `Nenhum site ainda. ${bot} pede no chat na primeira vez que abre um no navegador dele.`,
  manage: "Permissões",
  manageHint: "Ver e tirar o que cada agente pode fazer",
};
