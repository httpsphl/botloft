// El Creador (spec 36): su chat, sus planes y sus ajustes.

import type { Messages } from "../en";

export const maker: Messages["maker"] = {
  emptyTitle: "Cuéntame qué haces, y armo tus equipos",
  emptyBody:
    "Describe tu trabajo en pocas líneas: qué haces cada día, qué se repite, qué te gustaría delegar. Armo equipos, agentes y rutinas listos para usar, y tú revisas todo antes de que se cree nada.",
  startHere: "Dejar que el Creador arme",
  startHereHint: "Cuéntale de tu trabajo y arma los equipos por ti.",
  card: {
    title: (title) => `El Creador quiere armar: ${title}`,
    why: "Por qué",
    newCrew: "Equipo nuevo",
    include: (name) => `Incluir ${name}`,
    goal: "Meta",
    goalHint: "Las instrucciones del jefe.",
    memory: "Memoria del equipo",
    memoryHint: "Todos los agentes del equipo leen esto.",
    chief: (name) => `${name}, el jefe`,
    agents: "Agentes",
    routines: "Corre solo",
    forAgent: (agent) => `Para ${agent}`,
    runs: (count) =>
      count === 1
        ? "1 rutina correrá sola. Cada vez usa un poco de tu plan."
        : `${count} rutinas correrán solas. Cada vez usa un poco de tu plan.`,
    nothingLeft: "Deja al menos un paso en el plan.",
    changesIn: (crew) => `Cambios en ${crew}`,
    addAgent: (name) => `Añadir a ${name}`,
    changeAgent: (name) => `Cambiar a ${name}`,
    archiveAgent: (name) => `Archivar a ${name}`,
    archiveHint: "Márcalo solo si lo quieres: el agente va al archivo con sus rutinas.",
    addRoutine: (name) => `Rutina nueva: ${name}`,
    changeRoutine: (name) => `Cambiar la rutina ${name}`,
    deleteRoutine: (name) => `Borrar la rutina ${name}`,
    fieldName: "Nombre",
    fieldRole: "Papel",
    fieldModel: "Modelo",
    fieldEffort: "Esfuerzo",
    fieldWhen: "Cuándo",
    effortDefault: "El del modelo",
    newInstructions: "Instrucciones nuevas",
    addedInstructions: "Se añade a las instrucciones",
    oldInstructions: "Instrucciones de ahora",
    newText: "Texto nuevo",
    oldText: "Texto de ahora",
    turnOff: "Apagarla",
    turnOn: "Encenderla",
    build: "Armar",
    decline: "Ahora no",
    noteLabel: "Qué decirle al Creador si dices que no",
    notePlaceholder: "Si dices que no, dile al Creador por qué (opcional)",
    buildFailed: "No se pudo armar el plan",
    declineFailed: "No se pudo enviar la respuesta",
    built: (title) => `Armaste: ${title}`,
    builtAlone: (title) => `El Creador armó: ${title}`,
    declined: (title) => `No armaste: ${title}`,
    expired: (title) => `${title} no se respondió a tiempo`,
    open: "Abrir",
    undo: "Deshacer",
    undoTitle: "¿Deshacer lo que armó el Creador?",
    undoBody: (count) =>
      count === 1
        ? "El equipo va al archivo con sus agentes y rutinas. Lo traes de vuelta en Ajustes > Archivados."
        : `Los ${count} equipos van al archivo con sus agentes y rutinas. Los traes de vuelta en Ajustes > Archivados.`,
    undone: "Deshecho: los equipos están en el archivo",
    undoChanges: "Lo que cambió en tus equipos vuelve a ser como era.",
    undoneChanges: "Deshecho: tus equipos volvieron a ser como eran",
    undoFailed: "No se pudo deshacer",
  },
  settings: {
    title: "El Creador",
    show: "Mostrar el Creador",
    showOn: "Queda arriba de la lista a la izquierda.",
    showOff: "Oculto de la lista. No se borra nada de él.",
    withoutAsking: "Dejar que el Creador arme sin preguntar",
    withoutAskingOn: "Arma sus planes enseguida; puedes deshacer cada uno durante una semana.",
    withoutAskingOff: "Revisas cada plan antes de que se cree nada.",
    confirmTitle: "¿Dejar que el Creador arme sin preguntar?",
    confirmBody:
      "Creará equipos, agentes y rutinas en cuanto los planee, sin mostrártelos antes. Archivar o pausar un agente sigue preguntando, y puedes deshacer cada armado durante una semana.",
    confirm: "Dejar que arme",
  },
  history: {
    open: "Traer mi historial de Claude Code",
    title: "Traer tu historial de Claude Code",
    intro:
      "Botloft lee el historial que Claude Code guarda en esta computadora y hace un resumen para el Creador. Eliges los proyectos y lees el resumen entero antes de que salga nada.",
    takes:
      "Solo los pedidos que escribiste, los títulos de las sesiones, las herramientas usadas, tus CLAUDE.md y los nombres de tus skills. Nunca las respuestas de Claude, lo que las herramientas leyeron o devolvieron, ni adjuntos.",
    loading: "Leyendo tu historial de Claude Code…",
    loadingHint:
      "Con muchos proyectos, esto tarda hasta un minuto. Puedes dejar esta ventana abierta y esperar.",
    scanError:
      "No se pudo leer el historial. A veces la primera lectura tarda: inténtalo de nuevo.",
    retry: "Intentar de nuevo",
    elapsed: (seconds: number) => `${seconds} s`,
    none: "Claude Code todavía no tiene historial en esta computadora.",
    scanFailed: "No se pudo leer el historial",
    all: "Todos los proyectos",
    project: (sessions: number, prompts: number, period: string) =>
      `${sessions} ${sessions === 1 ? "sesión" : "sesiones"} · ${prompts} ${prompts === 1 ? "pedido" : "pedidos"} · ${period}`,
    make: "Armar el resumen",
    making: "Armando el resumen…",
    makingHint:
      "Leyendo los proyectos elegidos y cambiando claves, correos y números. Esto tarda hasta un minuto.",
    makeFailed: "No se pudo armar el resumen",
    size: (thousands: number) =>
      `Cerca de ${thousands} mil tokens de tu plan cuando el Creador lo lea: como una conversación larga.`,
    swapped: "Cambiados antes de mostrártelo",
    kinds: {
      key: (count: number) =>
        `${count} ${count === 1 ? "clave o contraseña" : "claves o contraseñas"}`,
      email: (count: number) => `${count} ${count === 1 ? "correo" : "correos"}`,
      number: (count: number) => `${count} ${count === 1 ? "número" : "números"}`,
      card: (count: number) => `${count} ${count === 1 ? "tarjeta" : "tarjetas"}`,
    },
    notPerfect: "El cambio no es perfecto: léelo antes de enviarlo.",
    cut: "Quedaron fuera para caber en el límite",
    projects: "Proyectos en el resumen",
    remake: "Armarlo de nuevo sin los desmarcados",
    after:
      "Una vez enviado, el resumen es un archivo en la conversación del Creador y sale con ella. Un resumen que no envías se olvida después de un día.",
    readWith: "El Creador lo leerá con",
    readWithHint: "Un modelo más fuerte entiende mejor un historial grande y usa más de tu plan.",
    forget: "Olvidarlo",
    send: "Enviar al Creador",
    sendFailed: "No se pudo enviar el resumen",
    sent: "Enviado al Creador",
  },
  claudeAi: {
    open: "Traer mi exportación de claude.ai",
    title: "Traer tu exportación de claude.ai",
    intro:
      "Botloft no entra en claude.ai: allí descargas una copia de tus conversaciones y eliges el archivo aquí.",
    steps: [
      "En claude.ai, abre Configuración > Privacidad y haz clic en Exportar datos.",
      "Abre el correo de claude.ai y descarga el ZIP. El enlace sirve por 24 horas.",
      "Elige el ZIP aquí. Botloft lo lee donde está, sin copiarlo.",
    ],
    openSite: "Abrir claude.ai",
    pick: "Elegir el ZIP",
    other: "Elegir otro archivo",
    pickTitle: "Exportación de claude.ai",
    kind: "Archivo ZIP",
    takes:
      "De cada conversación, solo el título, la fecha, cuántos mensajes tiene y los tres primeros mensajes que escribiste. Nunca las respuestas de Claude ni los adjuntos.",
    scanFailed: "No se pudo leer este archivo",
    reading: "Leyendo el archivo…",
    readingHint: "Una exportación grande tarda hasta un minuto. Botloft la lee donde está.",
    found: (count: number, period: string) =>
      `${count} ${count === 1 ? "conversación" : "conversaciones"} · ${period}`,
    projects: (names: string) => `Proyectos: ${names}`,
    periodLabel: "Período",
    periods: {
      all: "Todo",
      year: "Último año",
      half: "Últimos 6 meses",
      quarter: "Últimos 3 meses",
    },
    cut: (count: number) =>
      `${count} ${count === 1 ? "conversación más vieja quedó" : "conversaciones más viejas quedaron"} fuera para caber en el límite`,
  },
};
