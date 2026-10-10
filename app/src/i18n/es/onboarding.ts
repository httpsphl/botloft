import type { Messages } from "../en";

export const onboarding: Messages["onboarding"] = {
  tagline: "Equipos de Claude Code siempre encendidos.",
  opening: "Abriendo Botloft…",
  connecting: "Conectando…",
  restartInBackground: "Reiniciar Botloft en segundo plano",
  installing: {
    install: "Preparando Botloft…",
    update: "Actualizando Botloft…",
    restart: "Reiniciando Botloft…",
    start: "Iniciando tus bots…",
  },
  installNote: (system: string) =>
    `Botloft ejecuta tus bots en segundo plano. En Configuración eliges si siguen trabajando después de cerrar esta ventana y si Botloft se inicia con ${system}.`,
  stopped: {
    title: "Botloft no pudo iniciar",
    body: "Botloft ejecuta tus bots en segundo plano, y esa parte no se inició.",
    notRunning: "No está en ejecución.",
  },
  outdated: {
    title: "Botloft no pudo terminar de actualizarse",
    body: "La parte que ejecuta tus bots en segundo plano sigue en la versión anterior.",
    running: (version: string) => `Versión en ejecución: ${version}.`,
    stillOld: (version: string) => `Sigue informando la versión ${version}.`,
  },
  late: "Se inició, pero no respondió a tiempo. Su registro está en la carpeta logs.",
  foreign: {
    title: "Otro programa está en el camino",
    body: (port: number) =>
      `Botloft necesita el puerto ${port} en este equipo, y otro programa lo está usando. Cierra ese programa y vuelve a intentarlo.`,
    detail: (port: number) =>
      `127.0.0.1:${port} responde, pero no es Botloft. Para usar otro puerto, define "port" en el config.toml de Botloft.`,
  },
  mismatch: {
    title: "Esta app no coincide con el Botloft en ejecución",
    body: "El Botloft en segundo plano es más nuevo que esta app. Instala la versión más reciente de Botloft.",
    detail: (version: string, theirs: number, ours: number) =>
      `Versión en ejecución ${version}, protocolo ${theirs}. Esta app usa el protocolo ${ours}.`,
  },
  cantConnect: {
    title: "Botloft no pudo conectarse",
    body: "Vuelve a intentarlo. Si sigue pasando, reinstala Botloft.",
  },
  welcome: {
    title: "Te damos la bienvenida a Botloft",
    intro:
      "Un equipo es un grupo de bots que siguen funcionando, se escriben entre sí y comparten una carpeta. Cada bot corre en el agente que elijas: Claude Code, u otro que actives. Cada equipo empieza con un jefe: dile para qué sirve el equipo, y él planea el trabajo y sugiere los bots que necesita.",
    botloft: "Botloft",
    running: (system: string) =>
      `Funcionando en segundo plano. Se inicia con ${system}, así que tus bots siguen trabajando después de cerrar esta ventana.`,
    runningNotAtStart:
      "Funcionando en segundo plano, así que tus bots siguen trabajando después de cerrar esta ventana.",
    runningWhileOpen:
      "Funcionando mientras esta ventana está abierta. Al cerrarla, tus bots se detienen.",
    claudeCode: "Claude Code",
    checking: "Comprobando…",
    version: (version: string) => `Versión ${version}`,
    account: "Cuenta de Claude",
    signedIn: "Sesión iniciada.",
    signedOut: "Tus bots trabajan con tu cuenta de Claude. Inicia sesión una vez y quedan listos.",
    ready: "listo",
    notReady: "no está listo",
    stillChecking: "comprobando",
    askFirstTitle: "Los bots preguntan antes de cambiar cosas",
    askFirstBody:
      "Cuando un bot quiere ejecutar un comando o editar un archivo, lo pregunta en su chat y espera a que lo permitas o lo deniegues.",
    ideasTitle: "O toca una idea",
    claudeOptional: "No instalado. Solo lo necesitas para los bots que corren en Claude Code.",
    agentMissing: "No encontrado. Instálalo y abre Botloft otra vez.",
    notNeeded: "no hace falta",
    createCrew: "Crear tu primer equipo",
  },
  claudeCode: {
    help: "Botloft ejecuta tus bots con Claude Code. Instálalo o actualízalo, ábrelo una vez para iniciar sesión, y Botloft lo detecta en menos de 30 segundos.",
    install: "Cómo instalar Claude Code",
    openFailed: "No se pudo abrir el enlace",
  },
  signIn: {
    button: "Iniciar sesión en Claude",
    waiting: "Esperando a que inicies sesión…",
    waitingNote:
      "Se abrió una ventana con el inicio de sesión de Claude. Termínalo en el navegador; Botloft continúa solo.",
    failedTitle: "El inicio de sesión no terminó",
    failedBody:
      "Vuelve a intentarlo y termina el inicio de sesión en el navegador antes de cerrar su ventana.",
  },
};
