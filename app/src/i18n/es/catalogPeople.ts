import type { people as en } from "../en/catalogPeople";

export const people: typeof en = {
  "training-designer": {
    name: "Diseñador de formación",
    role: "Diseña formación para un equipo: lo que la gente debe aprender, las lecciones y ejercicios, y cómo saber si funcionó",
    summary:
      "Diseña formación para tu equipo, con lecciones, ejercicios y una forma de saber si funcionó",
    about:
      "Parte de lo que la gente debe hacer distinto después, no del tema. Escribe objetivos que se pueden observar, lecciones cortas con una práctica cada una, ejercicios con respuestas y notas para quien enseña. También dice cómo sabrás que funcionó un mes después, y avisa si la formación no es lo que falta.",
    when: "Cuando un equipo debe aprender una habilidad, herramienta o forma de trabajar, o un recién llegado tiene que ponerse al día.",
    pairs:
      "Gestor del cambio, para la implantación, y Diseñador de presentaciones, para las diapositivas.",
  },
  "change-manager": {
    name: "Gestor del cambio",
    role: "Planifica cómo llevar un cambio a un equipo: a quién afecta, qué decir y cuándo, cómo escuchar las dudas y ver si cuajó",
    summary:
      "Planifica cómo llevar un cambio a tu equipo: a quién afecta, qué decir, cuándo, y cómo escuchar las dudas",
    about:
      "Toma un cambio, como una herramienta o un proceso nuevo, y planifica cómo llevarlo a quienes afecta: quién lo sabe primero y con qué palabras, las preguntas esperadas con respuestas honestas, cómo vuelven las dudas a ti y el apoyo después. No anuncia nada por su cuenta y nunca disfraza las partes difíciles.",
    when: "Antes de introducir algo que va a cambiar la forma de trabajar de la gente.",
    pairs: "Diseñador de formación, para el aprendizaje, y Redactor, para los mensajes.",
  },
  "performance-review-coach": {
    name: "Coach de evaluación de desempeño",
    role: "Ayuda a los responsables a preparar comentarios justos y concretos y conversaciones de evaluación, con los hechos y ejemplos que traen",
    summary:
      "Te ayuda a preparar comentarios justos y concretos y conversaciones de evaluación, con los hechos que traes",
    about:
      "Trabaja con los ejemplos que traes y escribe un comentario concreto, sobre el trabajo y equilibrado, y luego revisa sus propias palabras en busca de sesgo. Planifica la conversación y fija metas que se pueden comprobar. El sueldo, el ascenso y la sanción siguen siendo decisiones tuyas, y nunca contacta a la persona.",
    when: "Antes de una evaluación, una conversación difícil o una ronda de comentarios.",
    pairs: "Redactor, para pulir el texto, y Coach de metas, para las metas del próximo periodo.",
  },
  "hr-policy-writer": {
    name: "Redactor de políticas de RR. HH.",
    role: "Redacta políticas de trabajo y un manual del empleado en palabras sencillas, para que un abogado o asesor lo revise",
    summary:
      "Redacta políticas de trabajo y un manual del equipo en palabras sencillas, para que un profesional lo revise",
    about:
      "Redacta políticas sobre días libres, trabajo remoto, gastos, conducta y quejas, cada una con su propósito, regla y pasos, más una versión de dos minutos. No afirma lo que dice la ley: los pasajes que dependen del derecho laboral quedan como preguntas para un abogado o asesor de RR. HH. de tu país. Nada de lo que escribe está revisado legalmente.",
    when: "Cuando tu equipo ha crecido y las reglas solo existen en la cabeza de la gente.",
    pairs: "Corrector de textos, para pulir, y Gestor del cambio, para implantar las políticas.",
  },
  "engagement-survey-analyst": {
    name: "Analista de encuestas de clima laboral",
    role: "Diseña encuestas cortas para el equipo y lee las respuestas, protegiendo el anonimato, y las convierte en pocas acciones claras",
    summary:
      "Diseña encuestas cortas para el equipo, lee las respuestas manteniendo el anonimato y encuentra las acciones",
    about:
      "Pregunta primero si vas a actuar con las respuestas, luego escribe una encuesta corta y neutral y dice con claridad quién verá los resultados. Lee las respuestas por tema y nunca muestra el resultado de un grupo tan pequeño que alguien pueda ser reconocido. Termina con tres a cinco acciones y un mensaje para el equipo.",
    when: "Cuando quieres saber cómo se siente de verdad tu equipo y estás listo para hacer algo al respecto.",
    pairs:
      "Analista de datos, para muchas respuestas, y Redactor, para el mensaje de vuelta al equipo.",
  },
  "resume-tailor": {
    name: "Adaptador de currículums",
    role: "Reescribe tu currículum para un puesto concreto, a partir de tu experiencia real, y te prepara para la entrevista",
    summary:
      "Reescribe tu currículum para un puesto a partir de tu experiencia real y te prepara para la entrevista",
    about:
      "Lee la oferta de empleo, empareja cada requisito con algo que realmente hiciste y reescribe tu currículum y una carta breve para ese puesto. Te pide cifras y ejemplos, prepara las preguntas probables de la entrevista con respuestas hechas de tus propias historias y nunca inventa un empleo, un título, una habilidad ni una cifra.",
    when: "Siempre que te presentes a un puesto que importa.",
    pairs:
      "Coach de carrera, para el panorama general, y Corrector de textos, para revisar los textos finales.",
  },
  "career-coach": {
    name: "Coach de carrera",
    role: "Te ayuda a pensar en tu próximo paso en el trabajo: lo que quieres, en qué eres bueno, las opciones y un plan pequeño",
    summary:
      "Te ayuda a pensar en tu próximo paso en el trabajo y a convertirlo en un plan pequeño y posible",
    about:
      "Escucha primero y luego te ayuda a ver lo que quieres, en qué eres bueno y dos o tres opciones reales con lo que cuesta cada una. Convierte tu elección en tres pasos pequeños para el próximo mes y ayuda a preparar una conversación difícil, como pedir un aumento. No es terapeuta ni asesor, y la elección es tuya.",
    when: "Cuando dudas sobre tu próximo paso en el trabajo, o te enfrentas a una elección difícil.",
    pairs:
      "Adaptador de currículums, para la candidatura, e Investigador, para datos sobre un sector o una empresa.",
  },
};
