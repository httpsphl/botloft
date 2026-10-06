import type { product as en } from "../en/catalogProduct";

export const product: typeof en = {
  "product-manager": {
    name: "Gerente de producto",
    role: "Convierte ideas y comentarios en un plan claro: qué construir, en qué orden y por qué",
    summary: "Decide qué construir primero y lo deja por escrito para que otros lo construyan",
    about:
      "Mira tus ideas, los comentarios que recibes y los números, y te ayuda a elegir qué vale la pena construir ahora. Escribe cada decisión como un resumen corto, con el que el resto del equipo puede construir.",
    when: "Cuando tienes más ideas que tiempo y necesitas elegir.",
    pairs:
      "Analista de requisitos, para detallar cada punto, y Desarrollador y Diseñador, que lo construyen.",
  },
  "project-manager": {
    name: "Gerente de proyectos",
    role: "Planifica un proyecto, sigue quién hace qué y para cuándo, y avisa de lo que se atrasa o se bloquea",
    summary: "Divide un proyecto en pasos, sigue el avance y avisa pronto cuando algo se retrasa",
    about:
      "Convierte un objetivo en un plan con pasos, responsables y fechas, lo mantiene al día y avisa pronto cuando algo se atrasa o se atasca.",
    when: "Para cualquier cosa con varios pasos y personas: un lanzamiento, una reforma, una mudanza, una campaña.",
    pairs: "Facilitador ágil, para el ritmo del trabajo, y Secretario de reuniones.",
  },
  "business-analyst": {
    name: "Analista de requisitos",
    role: "Convierte un pedido vago en requisitos claros, preguntas y criterios para saber cuándo está listo",
    summary: "Hace las preguntas correctas y escribe exactamente qué hay que construir",
    about:
      "Toma un pedido vago como 'necesito una forma de seguir los pedidos' y lo convierte en requisitos precisos, con las excepciones y la manera de que todos sepan que está bien.",
    when: "Antes de construir cualquier cosa que aún no está clara.",
    pairs:
      "Gerente de producto, que fija las prioridades, y Desarrollador y Tester, que usan los requisitos.",
  },
  "ux-researcher": {
    name: "Investigador de usuarios",
    role: "Planifica cómo descubrir lo que necesitan los usuarios y convierte entrevistas y comentarios en conclusiones",
    summary:
      "Descubre lo que tus usuarios realmente necesitan, por lo que dicen y por lo que hacen",
    about:
      "Planifica entrevistas, encuestas y pruebas, y convierte las notas y respuestas que le traes en conclusiones con evidencia. No habla con nadie por su cuenta: prepara y analiza.",
    when: "Antes de una decisión importante sobre un producto, una página o un servicio.",
    pairs:
      "Gerente de producto y Diseñador, que usan las conclusiones, y Analista de datos, para los números de las encuestas.",
  },
  "agile-facilitator": {
    name: "Facilitador ágil",
    role: "Dirige la planificación, los seguimientos rápidos y las retrospectivas, y mantiene el trabajo del equipo en marcha",
    summary:
      "Mantiene el trabajo del equipo en marcha con pasos pequeños, seguimiento regular y retrospectivas honestas",
    about:
      "Mantiene un tablero simple de lo que viene, lo que está en curso y lo que está hecho, pide actualizaciones rápidas a los bots y dirige una retrospectiva al final de cada ronda para cambiar una cosa.",
    when: "Cuando el equipo tiene muchos bots y el trabajo se atasca o se olvida.",
    pairs: "Gerente de proyectos, para el plan, y Secretario de reuniones.",
  },
  "goals-coach": {
    name: "Coach de metas",
    role: "Ayuda a fijar metas y medidas claras y comprueba el avance respecto a ellas",
    summary: "Convierte lo que quieres en metas que se pueden medir y comprueba cómo vas",
    about:
      "Te ayuda a decir lo que quieres de una forma que se pueda comprobar, elige pocas medidas que muestran el avance y mira los números contigo con regularidad, siendo honesto cuando te sales del rumbo.",
    when: "Cuando quieres hacer crecer algo y no estás seguro de que funcione.",
    pairs: "Analista de datos, que trae los números, y Gerente de proyectos.",
  },
  "meeting-secretary": {
    name: "Secretario de reuniones",
    role: "Prepara agendas, toma notas y convierte las reuniones en decisiones y tareas",
    summary:
      "Prepara la reunión, la pone por escrito y la convierte en decisiones y cosas por hacer",
    about:
      "Prepara agendas y, a partir de las notas o la transcripción que le des, escribe las decisiones, las tareas y las preguntas abiertas. No entra en llamadas y no envía nada por su cuenta.",
    when: "Cuando las reuniones terminan sin un registro claro de quién hará qué.",
    pairs: "Gerente de proyectos y Facilitador ágil, que dan seguimiento a las tareas.",
  },
  "process-analyst": {
    name: "Analista de procesos",
    role: "Mapea cómo se hace el trabajo hoy, encuentra el desperdicio y escribe una mejor forma en pasos claros",
    summary:
      "Dibuja cómo ocurre realmente el trabajo, encuentra lo que hace perder tiempo y escribe una forma más simple",
    about:
      "Mapea un proceso como funciona de verdad, encuentra pasos repetidos, esperas y errores, y escribe una versión más simple como una lista que una persona nueva podría seguir.",
    when: "Cuando el mismo trabajo es lento, tiene muchos errores o solo existe en la cabeza de alguien.",
    pairs: "Gerente de operaciones y Analista de datos, que ayudan a medir.",
  },
  "operations-manager": {
    name: "Gerente de operaciones",
    role: "Mantiene el día a día funcionando: rutinas, listas de comprobación, proveedores y lo que siempre se retrasa",
    summary:
      "Mantiene el día a día en marcha: rutinas, listas de comprobación y las cosas que siempre se retrasan",
    about:
      "Sigue lo que debe ocurrir cada día, cada semana y cada mes, convierte lo que tienes en la cabeza en listas de comprobación, vigila renovaciones y suministros y avisa antes de que algo se olvide. Puede proponer rutinas que se ejecutan en horarios fijos.",
    when: "Cuando llevar el negocio te quita el tiempo que necesitas para hacerlo crecer.",
    pairs: "Analista de procesos, que mejora las rutinas, y Asistente personal.",
  },
  recruiter: {
    name: "Reclutador",
    role: "Escribe ofertas de empleo, ordena las candidaturas con criterios claros y prepara entrevistas",
    summary:
      "Escribe la oferta, ordena las candidaturas con criterios claros y prepara buenas preguntas de entrevista",
    about:
      "Escribe la oferta, define cómo es un buen candidato, compara las candidaturas que le des con esos criterios y prepara las mismas preguntas de entrevista para todos. Recomienda, tú decides, y nunca contacta a los candidatos.",
    when: "Cuando estás contratando y quieres un proceso justo y ordenado.",
    pairs: "Redactor, para pulir la oferta, e Investigador, para mirar el mercado.",
  },
  "onboarding-coach": {
    name: "Coach de incorporación",
    role: "Prepara las primeras semanas de una persona nueva: qué aprender, a quién conocer y qué hacer primero",
    summary:
      "Planifica las primeras semanas de una persona nueva para que sea útil pronto y se sienta bienvenida",
    about:
      "Planifica el primer día, la primera semana y el primer mes de quien llega, escribe el mensaje de bienvenida y una guía de cómo funcionan las cosas, y enumera lo que debe estar listo antes de que empiece.",
    when: "Cuando alguien se incorpora y quieres que sea útil rápido.",
    pairs: "Analista de procesos, que mapea el trabajo, y Redactor, para pulir la guía.",
  },
  "customer-success": {
    name: "Éxito del cliente",
    role: "Ayuda a los clientes a sacar valor: hace seguimiento, detecta a los que están en riesgo y planifica el siguiente paso",
    summary:
      "Ayuda a los clientes a sacar valor de verdad, detecta quién está insatisfecho y sugiere qué hacer",
    about:
      "Mantiene una tabla simple de tus clientes, detecta a los que están en riesgo (menos uso, pagos atrasados, quejas) y redacta un mensaje corto y cálido para cada uno que necesita atención. No envía nada sin tu aprobación.",
    when: "Cuando tienes clientes recurrentes y quieres que se vayan menos.",
    pairs: "Atención al cliente, para las dudas, y Analista de datos, para los números de uso.",
  },
  "event-planner": {
    name: "Organizador de eventos",
    role: "Planifica un evento de principio a fin: presupuesto, programa, proveedores y lista de comprobación",
    summary:
      "Planifica tu evento desde el presupuesto hasta el último punto de la lista, para que nada se olvide",
    about:
      "Planifica eventos de cualquier tamaño: presupuesto por partidas, un calendario contado hacia atrás desde la fecha, opciones de proveedores para comparar, el programa del día y una lista para el día siguiente. No reserva ni invita a nadie.",
    when: "Para un lanzamiento, una fiesta, un taller o una reunión familiar.",
    pairs: "Investigador, para comparar proveedores, y Redactor, para las invitaciones.",
  },
  "travel-planner": {
    name: "Planificador de viajes",
    role: "Planifica un viaje: ruta, alojamiento, programa por día y presupuesto, con opciones y qué reservar",
    summary:
      "Planifica tu viaje con opciones, un programa día a día y un presupuesto, y te dice qué reservar primero",
    about:
      "Investiga rutas, alojamientos y actividades, propone opciones, arma un plan día a día con tiempos realistas y un presupuesto, y enumera qué reservar y hasta cuándo. Nunca reserva y nunca escribe tus documentos ni tus tarjetas en ningún sitio.",
    when: "Para unas vacaciones, un viaje de trabajo o una visita familiar.",
    pairs:
      "Investigador, para mirar un lugar más a fondo, y Analista de datos, para comparar costos.",
  },
};
