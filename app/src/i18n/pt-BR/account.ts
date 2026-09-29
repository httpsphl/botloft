import type { Messages } from "../en";

export const account: Messages["account"] = {
  open: (name: string) => `${name}: conta e configurações`,
  plan: (plan: string) => `Plano ${plan}`,
  notSignedIn: "Sem conta do Claude conectada",
  menu: {
    usage: "Uso",
    settings: "Configurações",
    language: "Idioma",
    whatsNew: "Novidades",
    help: "Ajuda",
    openFailed: "Não foi possível abrir a página",
  },
  usage: {
    title: "Uso",
    intro:
      "Quanto do seu plano do Claude os seus bots já usaram. Atualiza sempre que um bot trabalha.",
    empty: "O uso aparece depois da primeira resposta de um bot.",
    windows: {
      five_hour: "Janela de 5 horas",
      seven_day: "Esta semana",
      seven_day_opus: "Esta semana, Opus",
      seven_day_sonnet: "Esta semana, Sonnet",
    },
    used: (percent: number) => `${percent}% usado`,
    resets: (relative: string) => `Renova ${relative}`,
    limited: "Limite atingido. Seus bots esperam até ele renovar.",
    updated: (relative: string) => `Atualizado ${relative}`,
  },
  settings: {
    title: "Configurações",
    appearance: "Aparência",
    theme: "Tema",
    themes: { system: "Sistema", light: "Claro", dark: "Escuro" },
    size: "Tamanho",
    sizeHint: "Ctrl+= e Ctrl+- também mudam o tamanho, e Ctrl+0 volta ao padrão.",
    language: "Idioma",
    about: "Sobre",
    claudeCode: (version: string) => `Claude Code ${version}`,
    botloft: (version: string) => `Botloft ${version}`,
  },
};
