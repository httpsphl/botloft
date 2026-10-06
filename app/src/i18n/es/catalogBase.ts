import type { base as en } from "../en/catalogBase";

export const base: typeof en = {
  developer: {
    name: "Desarrollador",
    role: "Escribe y cambia código, ejecuta las pruebas y explica qué cambió",
    summary: "Crea y arregla software, revisa su propio trabajo y cuenta qué hizo",
    about:
      "Escribe funciones nuevas, corrige errores y ordena el código de tu proyecto. Ejecuta las pruebas después de cada cambio y te cuenta, con palabras sencillas, qué hizo y qué falta.",
    when: "Cuando tienes un sitio web, una app o un script que crear o arreglar.",
    pairs:
      "Revisor de código y Tester, que revisan su trabajo, y Diseñador, que le pasa las pantallas para construir.",
  },
  "code-reviewer": {
    name: "Revisor de código",
    role: "Revisa lo que cambiaron otros bots y señala errores, riesgos y complicaciones innecesarias",
    summary: "Lee los cambios con ojo crítico y dice qué está mal o es arriesgado",
    about:
      "Lee los cambios que hizo otro bot y enumera lo que puede romperse, lo que es arriesgado y lo que está más complicado de lo necesario. Nunca edita el código, solo informa, y la decisión es tuya. Piensa más a fondo que la mayoría de los bots, así que gasta un poco más de tu plan.",
    when: "Después de que un Desarrollador cambie algo importante: un pago, un acceso, cualquier cosa difícil de deshacer.",
    pairs: "Desarrollador, cuyo trabajo revisa, y Tester.",
  },
  "qa-tester": {
    name: "Tester",
    role: "Prueba lo que se hizo, intenta romperlo e informa cómo repetir cada problema",
    summary: "Usa el trabajo como lo haría una persona y anota cómo repetir cada problema",
    about:
      "Prueba lo que se construyó como lo haría una persona de verdad, y luego como lo haría alguien descuidado, e informa de cada problema con los pasos para repetirlo. No lo arregla, lo encuentra.",
    when: "Antes de mostrar o publicar algo, o cuando la gente dice que se rompió.",
    pairs: "Desarrollador, que arregla lo que encuentra, y Revisor de código.",
  },
  designer: {
    name: "Diseñador",
    role: "Diseña pantallas y páginas en HTML que ves en vivo y puedes pedir que cambie",
    summary: "Diseña páginas y pantallas que ves tomar forma y luego las afina contigo",
    about:
      "Diseña páginas y pantallas en HTML. Ves cada una tomar forma en el área de diseño mientras trabaja, y le pides cambios como se los pedirías a una persona.",
    when: "Para una página de ventas, una pantalla de app, un menú, una tarjeta o cualquier idea visual que quieras ver antes de construirla.",
    pairs:
      "Desarrollador, que puede construir lo que diseña, y Redactor, para las palabras de la página.",
  },
  writer: {
    name: "Redactor",
    role: "Escribe artículos, correos y páginas con tu tono de voz",
    summary: "Escribe textos claros que suenan como tú y sirven a quien los va a leer",
    about:
      "Escribe artículos, correos, textos de producto y páginas con tu tono de voz. Dale una muestra de tu forma de escribir y la sigue.",
    when: "Cuando las palabras tienen que ser claras y sonar como tú.",
    pairs: "Investigador, para los datos, y Traductor, para otros idiomas.",
  },
  "social-media": {
    name: "Redes sociales",
    role: "Planifica contenido y escribe publicaciones para cada red, pero nunca publica por su cuenta",
    summary: "Propone ideas, calendario y publicaciones para que las apruebes",
    about:
      "Planifica qué publicar y escribe las publicaciones para cada red. Nunca publica por su cuenta: tú apruebas cada una.",
    when: "Cuando quieres presencia constante sin empezar de cero cada vez.",
    pairs: "Diseñador, para las imágenes, y Redactor, para textos más largos.",
  },
  translator: {
    name: "Traductor",
    role: "Traduce y adapta textos entre idiomas, manteniendo el tono y avisando de lo que se pierde",
    summary: "Traduce para que suene natural y avisa dónde cambia el sentido",
    about:
      "Traduce textos y los adapta para quien los va a leer, manteniendo el tono y avisando dónde se pierde el sentido.",
    when: "Para un sitio web en otro idioma, un correo a un cliente en el extranjero o un documento que no puedes leer.",
    pairs: "Redactor, para rehacer un texto que necesita más que una traducción.",
  },
  researcher: {
    name: "Investigador",
    role: "Investiga una pregunta en la web, comprueba las fuentes e informa qué encontró y qué no",
    summary: "Busca en la web, comprueba las fuentes y te da una respuesta corta con enlaces",
    about:
      "Busca en la web con su propio navegador, que puedes mirar, compara fuentes y te da una respuesta corta con los enlaces. Dice lo que no pudo encontrar.",
    when: "Para precios, competidores, normas, tutoriales o cualquier duda que de otro modo te llevaría una hora buscar.",
    pairs: "Redactor y Analista de datos, que usan lo que encuentra.",
  },
  "data-analyst": {
    name: "Analista de datos",
    role: "Lee hojas de cálculo y datos, hace las cuentas, muestra cómo llegó y explica qué significa",
    summary:
      "Convierte hojas de cálculo y números en respuestas, muestra la cuenta y hace gráficos",
    about:
      "Lee hojas de cálculo y datos, hace las cuentas, muestra cómo llegó a cada resultado y explica qué significa, con gráficos que puedes ver.",
    when: "Cuando tienes números (ventas, gastos, resultados) y quieres respuestas, no tablas.",
    pairs: "Investigador, para datos de fuera de tus datos.",
  },
  "sales-prospector": {
    name: "Prospector de ventas",
    role: "Encuentra y evalúa posibles clientes y redacta los mensajes, pero no envía nada sin ti",
    summary:
      "Encuentra personas y empresas que encajan, las ordena y redacta los primeros mensajes",
    about:
      "Encuentra personas y empresas que encajan con lo que vendes, explica por qué y redacta el primer mensaje para cada una. No envía nada sin tu aprobación. Funciona mejor con una herramienta conectada a la red donde buscas clientes.",
    when: "Cuando necesitas una lista de buenos contactos y un comienzo para cada conversación.",
    pairs: "Redactor, para pulir los mensajes, e Investigador, para indagar en una empresa.",
  },
  "customer-support": {
    name: "Atención al cliente",
    role: "Responde dudas de clientes con el material que le diste y pasa lo que no sabe",
    summary: "Responde dudas con amabilidad usando tu material y te pasa lo que no sabe",
    about:
      "Responde dudas de clientes con el material que le das y te pasa los casos que no resuelve. Redacta las respuestas y, al principio, espera tu aprobación.",
    when: "Cuando las mismas preguntas no dejan de llegar y quieres ayuda para responderlas bien.",
    pairs: "Desarrollador, para problemas técnicos, y Redactor, para la forma de decirlo.",
  },
  "personal-assistant": {
    name: "Asistente personal",
    role: "Mantiene tus tareas en orden, resume lo que necesitas leer y redacta correos",
    summary: "Organiza tus tareas, resume lo que necesitas leer y redacta tus correos",
    about:
      "Mantiene tus tareas en orden, resume lo que necesitas leer, redacta correos y te recuerda lo que importa. La agenda y el correo solo funcionan si conectas una herramienta para ello.",
    when: "Cuando tu día está lleno de pequeñas cosas que se acumulan.",
    pairs: "Investigador y Redactor, para trabajos más grandes.",
  },
};
