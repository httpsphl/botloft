import type { contentAndLearning as en } from "../en/catalogContent";

export const contentAndLearning: typeof en = {
  editor: {
    name: "Corrector de textos",
    role: "Revisa y edita textos: corrige errores, depura la escritura y mantiene la voz del autor",
    summary: "Corrige errores y depura tu escritura, manteniendo el estilo que suena a ti",
    about:
      "Corrige ortografía, gramática y frases torpes y recorta lo que sobra, en el nivel que pidas: solo los errores, la forma de decirlo o también la estructura. Muestra cada cambio con su motivo y nunca altera un dato, una cifra ni una cita.",
    when: "Antes de publicar o enviar algo que importa.",
    pairs:
      "Verificador de datos, para comprobar afirmaciones, y Redactor, cuando una parte hay que rehacerla.",
  },
  ghostwriter: {
    name: "Escritor fantasma",
    role: "Escribe con tu voz: publicaciones, discursos, biografías breves y textos más largos que publicas como tuyos",
    summary:
      "Escribe con tu voz lo que vas a publicar con tu nombre, a partir de tus ideas e historias",
    about:
      "Aprende cómo suenas, te saca las historias y opiniones con preguntas y escribe borradores con tu voz. Marca cada hueco que rellenó para que lo confirmes, nunca inventa experiencias y no escribe trabajos que debes entregar como tuyos, como un examen o una tesis.",
    when: "Cuando tienes algo que decir, pero no el tiempo ni las palabras.",
    pairs: "Investigador, para los datos, y Corrector de textos, para pulir.",
  },
  "fact-checker": {
    name: "Verificador de datos",
    role: "Comprueba afirmaciones en fuentes fiables y dice qué tan seguro está, o que no pudo saberlo",
    summary: "Comprueba si una afirmación es cierta en fuentes fiables y dice qué tan seguro está",
    about:
      "Toma las afirmaciones de un texto y comprueba cada una en las mejores fuentes que encuentre, prefiriendo el documento original. Da un veredicto con palabras sencillas, con el enlace y qué tan seguro está, y dice con honestidad cuando no pudo saberlo.",
    when: "Antes de publicar, y siempre que algo que leíste suena demasiado bueno para ser cierto.",
    pairs: "Corrector de textos y Redactor, que aplican las correcciones.",
  },
  "academic-researcher": {
    name: "Investigador académico",
    role: "Encuentra y resume fuentes académicas sobre un tema, con citas reales y un relato honesto de la evidencia",
    summary:
      "Encuentra y resume trabajos académicos sobre un tema, con citas reales y una visión honesta de la evidencia",
    about:
      "Encuentra artículos, libros e informes sobre tu pregunta, abre cada uno antes de citarlo y resume la conclusión, el método y los límites. Nunca inventa una referencia, dice cuando solo leyó el resumen y no escribe trabajos que debes entregar como tuyos.",
    when: "Cuando necesitas saber qué dice realmente la investigación sobre un tema.",
    pairs:
      "Analista de datos, para los números de un estudio, y Redactor, para un resumen legible.",
  },
  tutor: {
    name: "Tutor",
    role: "Enseña un tema paso a paso, a tu nivel, y comprueba que de verdad entendiste",
    summary: "Te enseña un tema paso a paso, a tu nivel, y comprueba que de verdad entendiste",
    about:
      "Descubre lo que ya sabes y enseña una idea a la vez, con ejemplos de tu mundo. Te pide que se lo expliques de vuelta o hagas un ejercicio pequeño, da pistas antes que respuestas y corrige con amabilidad. No hace por ti tu trabajo con nota.",
    when: "Cuando quieres aprender un tema bien, o ayudar a alguien que estudia.",
    pairs: "Investigador, para fuentes, y Redactor, para apuntes de estudio.",
  },
  "language-teacher": {
    name: "Profesor de idiomas",
    role: "Enseña un idioma con conversación, correcciones amables, vocabulario y ejercicios cortos",
    summary:
      "Te ayuda a aprender un idioma conversando, con correcciones amables, vocabulario y ejercicios cortos",
    about:
      "Conversa contigo al nivel adecuado en el idioma que aprendes, corrige los errores que más importan con la regla en una línea, enseña palabras en contexto y termina cada sesión con un resumen y un ejercicio pequeño. Trabaja con texto, así que no oye tu pronunciación.",
    when: "Cuando quieres practicar un idioma todos los días, sin vergüenza.",
    pairs: "Traductor, para comprobar una frase difícil, y Redactor, para material de lectura.",
  },
};
