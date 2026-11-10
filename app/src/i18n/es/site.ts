// El sitio público (spec 39).

import type { SiteMessages } from "../en/site";
import { siteDemos } from "./siteDemos";

export const site: SiteMessages = {
  meta: {
    title: "Botloft: tus agentes de IA, siempre encendidos, trabajando juntos",
    description:
      "Una app gratuita para la computadora que mantiene un equipo de agentes de IA trabajando, sobre Claude Code. Se pasan trabajo entre ellos, siguen después de que cierras la ventana y piden permiso antes de actuar.",
  },
  skip: "Ir al contenido",
  nav: {
    label: "Secciones",
    features: "Funciones",
    how: "Cómo empezar",
    privacy: "Privacidad",
    faq: "Preguntas",
    github: "GitHub",
    language: "Idioma",
  },
  download: {
    windows: "Descargar para Windows",
    macos: "Descargar para macOS",
    linux: "Descargar para Linux",
    phone: "Descárgalo en tu computadora",
    short: "Descargar",
    all: "Todas las descargas",
    windowsFile: "Windows 10 u 11",
    macosFile: "macOS, Apple Silicon (versión preliminar)",
    debFile: "Linux .deb (versión preliminar)",
    appImageFile: "Linux AppImage (versión preliminar)",
  },
  hero: {
    badge: "Gratis · Windows, Linux y macOS",
    title: "Tus agentes de IA, siempre encendidos, trabajando juntos.",
    lead: "Botloft mantiene un equipo de agentes trabajando en tu computadora, cada uno sobre Claude Code. Se pasan trabajo entre ellos, siguen después de que cierras la ventana y te piden permiso antes de actuar.",
    github: "Código en GitHub",
    needs: "Necesitas Claude Code, con tu propia cuenta.",
    needsLink: "Cómo instalarlo",
    demo: "Un equipo trabajando en Botloft: Scout pide usar un sitio, encuentra las fuentes y se las pasa a Writer",
  },
  facts: {
    label: "En resumen",
    items: [
      "Funciona en tu computadora",
      "Sigue cuando lo cierras",
      "Pide permiso antes de actuar",
      "Claude Code, Antigravity, Codex",
      "Gratis, con el código público",
    ],
  },
  cards: {
    title: "Un equipo que trabaja mientras haces otra cosa",
    lead: "Cada agente tiene su carpeta, su memoria y su conversación. Hablas con cada uno como en una app de mensajes y decides qué pueden hacer.",
    browser: {
      title: "Míralos trabajar",
      body: "Ve en vivo cada página que abre un agente y toma el control para iniciar sesión por él.",
    },
    ask: {
      title: "Tú decides qué pueden hacer",
      body: "Un comando, una edición de archivo, un sitio nuevo: una tarjeta te pregunta antes, salvo que ya lo hayas permitido.",
    },
    handoff: {
      title: "Se pasan trabajo entre ellos",
      body: "Los agentes se envían mensajes y tareas. Cada uno se guarda antes de salir y se reintenta hasta que llega.",
    },
    routines: {
      title: "Rutinas con horario",
      body: "Las mañanas de días hábiles, cada dos horas o el horario que quieras. O un atajo que se ejecuta con un clic.",
    },
    phone: {
      title: "Desde tu teléfono",
      body: "Aprueba, responde y conversa desde cualquier lugar, todo sellado de extremo a extremo entre el teléfono y tu computadora.",
    },
    maker: {
      title: "El Creador arma por ti",
      body: "Pide equipos, agentes y rutinas con tus palabras. Ves el plan completo antes de que se cree nada, y Deshacer lo revierte.",
    },
    sleep: {
      title: "Siempre encendidos, sin estorbar",
      body: "Los agentes siguen después de cerrar Botloft. Los inactivos duermen, sin gastar memoria, y despiertan con la misma conversación.",
    },
  },
  demos: siteDemos,
  steps: {
    title: "Funcionando en tres pasos",
    items: {
      engine: {
        title: "Instala Claude Code",
        body: "Instálalo e inicia sesión una vez con tu propia cuenta. Tus agentes usan tu plan.",
        link: "Instalar Claude Code",
      },
      app: {
        title: "Descarga Botloft",
        body: "Ábrelo y se configura solo: no hay nada que ajustar. También se actualiza solo.",
      },
      crew: {
        title: "Crea tu primer equipo",
        body: "Dile para qué es y conoce los agentes que sugiere.",
      },
    },
  },
  engines: {
    title: "Elige el motor",
    lead: "Cada agente funciona sobre el motor de IA para programar que elijas, instalado en tu computadora y con tu propia cuenta.",
    claude: {
      title: "Claude Code",
      tag: "Experiencia completa",
      body: "Todas las funciones: pide permiso antes de cada herramienta, con “permitir siempre”, el navegador en vivo, las rutinas y cuánto queda de tu plan.",
    },
    others: {
      title: "Antigravity y Codex",
      tag: "Experimentales",
      body: "Apagados hasta que los actives en Configuración. Conversan, reciben tareas y hablan con el equipo; algunas funciones aún no se comprobaron con ellos.",
    },
  },
  privacy: {
    title: "Tuyo, en tu computadora",
    lead: "Botloft funciona en tu computadora. La cuenta es opcional, y nada sale sin que lo pidas.",
    items: {
      local: {
        title: "Sin telemetría",
        body: "Sin cuenta, Botloft usa la red solo para las actualizaciones y el trabajo de tus agentes.",
      },
      sealed: {
        title: "Copias selladas",
        body: "La copia opcional en la nube va sellada con una contraseña que nunca sale de tu computadora. El servidor no puede abrirla.",
      },
      keys: {
        title: "Claves fuera de la conversación",
        body: "Cuando un agente necesita una contraseña o un token, lo escribes en un campo oculto. Nunca entra en la conversación.",
      },
      code: {
        title: "Código público",
        body: "Lee cada línea en GitHub. Cada versión también pasa a ser Apache 2.0 dos años después de publicarse.",
      },
    },
  },
  faq: {
    title: "Preguntas",
    items: {
      cost: {
        q: "¿Botloft cuesta algo?",
        a: "No. Botloft es gratis, en casa o en el trabajo. Los agentes funcionan sobre el motor que elijas con tu propia cuenta, así que usan los límites de tu plan; con Claude Code, Botloft muestra cuánto queda.",
      },
      window: {
        q: "¿Tengo que dejar la ventana abierta?",
        a: "No. Los agentes funcionan en segundo plano, incluso después de cerrar Botloft, y arrancan cuando inicias sesión en tu computadora.",
      },
      systems: {
        q: "¿En qué computadoras funciona?",
        a: "Windows 10 y 11 (64 bits). Linux (64 bits, .deb o AppImage) y macOS con Apple Silicon lo ejecutan como versión preliminar; las Mac con Intel aún no son compatibles.",
      },
      warning: {
        q: "¿Por qué mi computadora me avisa al instalarlo?",
        a: "El instalador de Windows aún no tiene firma de código, así que SmartScreen no conoce a quien lo publica: elige “Más información” y luego “Ejecutar de todas formas”. Apple tampoco certificó aún la app de macOS: permítela una vez en Ajustes del Sistema, Privacidad y seguridad. Cada versión la arma GitHub a partir del código público, y las actualizaciones se firman y se comprueban antes de instalarse.",
      },
      anthropic: {
        q: "¿Botloft lo hace Anthropic?",
        a: "No. Botloft es un proyecto independiente. Usa Claude Code, Antigravity y Codex tal como están instalados en tu computadora y no tiene relación ni respaldo de Anthropic, Google ni OpenAI.",
      },
    },
  },
  closing: {
    title: "Dale a tus agentes un lugar para trabajar.",
    body: "Gratis, y listo en un par de minutos.",
  },
  footer: {
    links: "Enlaces",
    tagline: "Tus agentes de IA, siempre encendidos, trabajando juntos.",
    releases: "Versiones",
    license: "Licencia (FSL-1.1-ALv2)",
    signing: "Política de firma de código",
    security: "Seguridad",
    account: "Qué guarda la cuenta",
    notAffiliated: "Sin relación con Anthropic, Google ni OpenAI.",
  },
};
