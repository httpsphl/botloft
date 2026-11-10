// Lo que los agentes dicen y muestran en las demostraciones del sitio (spec 39.4).

import type { SiteMessages } from "../en/site";

export const siteDemos: SiteMessages["demos"] = {
  window: {
    crew: "Investigación",
    agents: "4 agentes",
    message: "Mensaje para Scout",
  },
  states: {
    idle: "Inactivo",
    working: "Trabajando",
    approval: "Necesita aprobación",
    sleeping: "Durmiendo",
  },
  roles: {
    scout: "Encuentra fuentes y las resume",
    writer: "Convierte notas en publicaciones",
    analyst: "Hace las cuentas",
    designer: "Dibuja las páginas de la tienda",
  },
  hero: {
    ask: "Encuentra tres buenas fuentes sobre masa madre y pásaselas a Writer para la publicación del viernes.",
    wants: "Scout quiere usar el navegador en",
    site: "panaderos.example",
    allow: "Permitir",
    once: "Solo esta vez",
    deny: "Denegar",
    allowed: "Permitiste panaderos.example",
    steps: "Dio 4 pasos en el navegador",
    reply:
      "Encontré tres: una guía de panadero para alimentar la masa madre, un artículo de ciencia de los alimentos sobre levadura salvaje y las notas de un panadero casero. Se las paso a Writer.",
    sent: "Le envió una tarea a Writer",
    task: "Escribir la publicación del viernes con estas tres fuentes",
  },
  browser: {
    live: "En vivo",
    tab: "Recetas",
    page: "Busca una receta",
    query: "masa madre",
    search: "Buscar",
    results: [
      "Alimentar la masa madre, día a día",
      "Por qué tu masa madre huele agria",
      "¿Centeno o trigo para empezar?",
    ],
    control: "Scout tiene el control",
    take: "Tomar el control",
  },
  ask: {
    wants: "Writer quiere ejecutar un comando",
    command: "npm run build",
    allow: "Permitir",
    deny: "Denegar",
    allowed: "Lo permitiste",
    after: "La compilación pasó. Publicando el borrador para que lo revises.",
  },
  handoff: {
    task: "Escribir la publicación del viernes",
  },
  routines: {
    items: [
      { name: "Resumen de la mañana", when: "Días hábiles, 8:00" },
      { name: "Revisión de enlaces", when: "Cada 2 horas" },
      { name: "Números de la semana", when: "Lunes, 9:00" },
    ],
    running: "En curso",
    done: "Lista, 8:00",
  },
  phone: {
    time: "9:41",
    notice: "Writer te necesita",
    noticeBody: "Quiere ejecutar npm run build",
    allow: "Permitir",
    deny: "Denegar",
    allowed: "Permitido",
  },
  maker: {
    ask: "Arma un equipo para la tienda en línea de mi panadería.",
    plan: "El plan",
    items: [
      "Equipo: Tienda de la panadería, con un Jefe",
      "Designer: dibuja las páginas de la tienda",
      "Atención: recibe los pedidos",
      "Rutina: resumen de pedidos, días hábiles 9:00",
    ],
    build: "Crear",
    change: "Cambiar",
    built: "Creado. Deshacer lo revierte.",
  },
  sleep: {
    message: "Mensaje nuevo",
  },
};
