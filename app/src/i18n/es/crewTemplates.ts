import type { crewTemplates as en } from "../en/crewTemplates";

export const crewTemplates: typeof en = {
  title: "Empezar con un modelo de equipo",
  intro: "Elige un modelo de equipo listo para trabajar, o empieza con un equipo vacío.",
  scratch: "Empezar con un equipo vacío",
  scratchHint: "Solo el jefe. Tú añades los bots.",
  bots: (count: number) => (count === 1 ? "1 bot" : `${count} bots`),
  adds: "Este modelo trae",
  change: "Elegir otro modelo",
  back: "Volver",
  partial: (failed: number) =>
    failed === 1
      ? "El equipo se creó, pero no se pudo añadir 1 bot. Puedes añadirlo desde la Agencia de bots."
      : `El equipo se creó, pero no se pudieron añadir ${failed} bots. Puedes añadirlos desde la Agencia de bots.`,
  loadFailed: "No se pudieron cargar los modelos",
  items: {
    "content-studio": {
      name: "Estudio de contenido",
      summary:
        "Planifica, escribe, revisa y publica contenido para un sitio, una marca o un canal.",
      goal: "Lleva un pequeño estudio de contenido: planifica qué publicar, haz que lo escriban y lo revisen y mantén en marcha las redes sociales. El equipo ya está aquí, así que reparte el trabajo como tareas y dime lo que necesites.",
    },
    "software-team": {
      name: "Equipo de software",
      summary: "Diseña, construye, prueba, revisa y documenta un software.",
      goal: "Construye y mantén mi software: planifica el trabajo, repártelo entre los desarrolladores, haz que cada cambio se pruebe y se revise y mantén la documentación verdadera. El equipo ya está aquí, así que reparte el trabajo como tareas.",
    },
    "online-store": {
      name: "Tienda en línea",
      summary:
        "Lleva los anuncios, los clientes, las devoluciones y la contabilidad de una tienda.",
      goal: "Ayúdame a llevar mi tienda en línea: mantén los productos y los anuncios al día, atiende a clientes y devoluciones según mi política y lleva la contabilidad. El equipo ya está aquí, así que reparte el trabajo como tareas.",
    },
    "growth-team": {
      name: "Equipo de crecimiento",
      summary:
        "Atrae clientes nuevos con anuncios y correo y comprueba que los números sean correctos.",
      goal: "Haz crecer mi base de clientes: planifica pruebas pequeñas y cuidadosas de anuncios y correo, asegura que los resultados se midan bien y dime qué parar, mantener y probar. El equipo ya está aquí, así que reparte el trabajo como tareas.",
    },
    "research-desk": {
      name: "Mesa de investigación",
      summary: "Encuentra, comprueba y explica lo que se sabe de un tema, con fuentes reales.",
      goal: "Responde a mis preguntas con investigación en la que pueda confiar: busca fuentes, comprueba los datos, haz las cuentas y escribe informes claros. El equipo ya está aquí, así que reparte el trabajo como tareas.",
    },
    "back-office": {
      name: "Trastienda de un negocio pequeño",
      summary:
        "Mantiene en orden la contabilidad, las cuentas, los cobros, la caja y los contratos.",
      goal: "Mantén en orden el papeleo de mi negocio: la contabilidad, las cuentas, los cobros, la previsión de caja, los impuestos y las fechas de los contratos. Nada se paga ni se envía sin mí. El equipo ya está aquí, así que reparte el trabajo como tareas.",
    },
    "personal-office": {
      name: "Oficina personal",
      summary: "Atiende tus tareas, viajes, cuentas, impuestos, metas y reuniones.",
      goal: "Sé mi oficina personal: mantén en orden mis tareas, viajes, cuentas e impuestos, sigue mis metas y anota mis reuniones. Nada se envía ni se paga sin mí. El equipo ya está aquí, así que reparte el trabajo como tareas.",
    },
    "course-studio": {
      name: "Estudio de cursos",
      summary: "Convierte lo que sabes en un curso en línea, con videos, diapositivas y promoción.",
      goal: "Convierte lo que sé en un curso en línea: planifícalo, escribe las lecciones, haz los guiones y las diapositivas y prepara la promoción. El equipo ya está aquí, así que reparte el trabajo como tareas.",
    },
    "game-studio": {
      name: "Estudio de juegos",
      summary: "Diseña un juego: las reglas, la historia, los niveles, los números y las pruebas.",
      goal: "Diseña mi juego: da forma a las reglas, escribe la historia, planifica los niveles, equilibra los números y aprende de las pruebas. El equipo ya está aquí, así que reparte el trabajo como tareas.",
    },
    "people-team": {
      name: "Equipo de personas",
      summary: "Contrata, acoge, forma y escucha a un equipo, sin decidir por ti.",
      goal: "Ayúdame a cuidar de mi equipo: contratación, incorporación, formación, políticas, comentarios y encuestas. Las decisiones sobre personas son mías. El equipo ya está aquí, así que reparte el trabajo como tareas.",
    },
  },
};
