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
    background: "En segundo plano",
    keepWorking: "Seguir trabajando después de cerrar Botloft",
    keepWorkingOn: "Tus bots siguen trabajando y respondiendo después de que cierras esta ventana.",
    keepWorkingOff:
      "Al cerrar Botloft, todos los bots se detienen. Siguen donde se quedaron cuando lo vuelves a abrir.",
    tray: "Mostrar Botloft junto al reloj",
    trayOn:
      "Con la ventana cerrada, su icono queda junto al reloj: un clic abre Botloft, y te avisa cuando un bot te necesita.",
    trayOff: "Cerrar la ventana cierra Botloft. Los bots siguen trabajando, sin icono ni avisos.",
    startWithWindows: "Iniciar con Windows",
    startWithWindowsOn:
      "Cuando inicias sesión en Windows, tus bots vuelven a trabajar solos, sin abrir esta ventana.",
    startWithWindowsOff: "Después de reiniciar el equipo, los bots esperan a que abras Botloft.",
    openAtSignIn: "Abrir la ventana al iniciar sesión en Windows",
    openAtSignInOn: "La ventana de Botloft se abre cuando inicias sesión en Windows.",
    openAtSignInNearClock: "Botloft empieza junto al reloj, sin abrir la ventana.",
    openAtSignInOff: "La ventana solo se abre cuando abres Botloft.",
    alerts: "Avisos",
    notifyNeeds: "Cuando un bot te necesite",
    notifyNeedsHint:
      "Un aviso de Windows cuando un bot pide permiso o necesita que inicies sesión, si Botloft no está al frente.",
    notifyDone: "Cuando un bot termine",
    notifyDoneHint: "Un aviso cuando un bot termina lo que estaba haciendo.",
    sound: "Reproducir un sonido",
    soundHint: "Un sonido corto con cada aviso, también con Botloft al frente.",
    alertsNeedTray:
      "Con la ventana cerrada, los avisos solo llegan con Botloft junto al reloj (en General).",
    keepAwake: "Mantener el equipo despierto mientras los bots trabajan",
    keepAwakeHint: "Igual se suspende si cierras la tapa o eliges Suspender.",
    saveFailed: "No se pudo cambiar el ajuste",
    pages: "Partes de la configuración",
    general: "General",
    chat: "Chat",
    enterSends: "Enter envía el mensaje",
    enterSendsOn: "Shift+Enter empieza una nueva línea.",
    enterSendsOff: "Enter empieza una nueva línea, y Ctrl+Enter envía.",
    followBot: "Abrir el navegador y las pantallas cuando un bot empiece a usarlos",
    followBotOn: "El panel se abre junto al chat, para que veas lo que hace el bot.",
    followBotOff: "El botón del panel muestra un punto, y lo abres cuando quieras.",
    approvalWait: "Cuánto espera tu respuesta un pedido de permiso",
    approvalWaitHint: "Sin respuesta en ese tiempo, el pedido del bot se deniega y sigue sin eso.",
    waitFor: (minutes: number) =>
      minutes < 60 || minutes % 60 !== 0
        ? `${minutes} min`
        : minutes === 60
          ? "1 hora"
          : `${minutes / 60} horas`,
    lessMotion: "Menos animaciones",
    lessMotionHint:
      "La ventana queda quieta, como cuando los efectos de animación de Windows están desactivados.",
    appearance: "Apariencia",
    theme: "Tema",
    themes: { system: "Sistema", light: "Claro", dark: "Oscuro" },
    size: "Tamaño",
    sizeHint: "Ctrl+= y Ctrl+- también cambian el tamaño, y Ctrl+0 vuelve al predeterminado.",
    language: "Idioma",
    about: "Acerca de",
    claudeCode: (version: string) => `Claude Code ${version}`,
    botloft: (version: string) => `Botloft ${version}`,
    checkUpdates: "Buscar actualizaciones",
    checking: "Buscando…",
    upToDate: "Tienes la versión más reciente.",
    updateFound: (version: string) => `Salió Botloft ${version}.`,
    seeUpdate: "Ver la actualización",
    checkFailed: "No se pudieron buscar actualizaciones. Inténtalo más tarde.",
  },
};
