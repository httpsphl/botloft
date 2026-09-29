import type { Messages } from "../en";

export const account: Messages["account"] = {
  open: (name: string) => `${name}: cuenta y configuración`,
  plan: (plan: string) => `Plan ${plan}`,
  notSignedIn: "Sin sesión en Claude",
  menu: {
    usage: "Uso",
    settings: "Configuración",
    language: "Idioma",
    whatsNew: "Novedades",
    help: "Ayuda",
    openFailed: "No se pudo abrir la página",
  },
  usage: {
    title: "Uso",
    intro:
      "Cuánto de tu plan de Claude han usado tus bots. Se actualiza cada vez que un bot trabaja.",
    empty: "El uso aparece después de la primera respuesta de un bot.",
    windows: {
      five_hour: "Ventana de 5 horas",
      seven_day: "Esta semana",
      seven_day_opus: "Esta semana, Opus",
      seven_day_sonnet: "Esta semana, Sonnet",
    },
    used: (percent: number) => `${percent}% usado`,
    resets: (relative: string) => `Se renueva ${relative}`,
    limited: "Límite alcanzado. Tus bots esperan hasta que se renueve.",
    updated: (relative: string) => `Actualizado ${relative}`,
  },
  settings: {
    title: "Configuración",
    appearance: "Apariencia",
    theme: "Tema",
    themes: { system: "Sistema", light: "Claro", dark: "Oscuro" },
    size: "Tamaño",
    sizeHint: "Ctrl+= y Ctrl+- también cambian el tamaño, y Ctrl+0 vuelve al predeterminado.",
    language: "Idioma",
    about: "Acerca de",
    claudeCode: (version: string) => `Claude Code ${version}`,
    botloft: (version: string) => `Botloft ${version}`,
  },
};
