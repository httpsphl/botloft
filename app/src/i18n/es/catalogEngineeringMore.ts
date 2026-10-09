import type { engineeringMore as en } from "../en/catalogEngineeringMore";

export const engineeringMore: typeof en = {
  "technical-writer": {
    name: "Redactor técnico",
    role: "Escribe documentación clara a partir del código y la configuración reales: guías, referencias y pasos que coinciden con lo que hace el sistema",
    summary:
      "Escribe documentación clara a partir de tu código y tu configuración reales, que sigue siendo verdadera",
    about:
      "Lee tu código y ejecuta en una copia los comandos que documenta, para que los pasos y la salida sean reales. Escribe para un lector cada vez: la tarea más común primero, pasos numerados, lo que deberías ver tras cada uno y qué hacer cuando falla. Dice lo que probó y lo que solo leyó.",
    when: "Cuando tu proyecto no tiene README, la documentación está vieja o alguien nuevo va a hacerse cargo.",
    pairs:
      "Guía del código, para el panorama general, y Revisor de código, para comprobar las afirmaciones técnicas.",
  },
  "sre-engineer": {
    name: "Ingeniero de fiabilidad",
    role: "Hace tu servicio fiable: define qué es bueno en números, monta alertas que importan y escribe los pasos para cuando falle",
    summary:
      "Hace tu servicio fiable: metas claras, alertas que importan y pasos para cuando falle",
    about:
      "Convierte lo que sienten tus usuarios en pocas metas en números, mapea dónde puede fallar el servicio y propone alertas que significan que alguien debe actuar ya, cada una con lo primero que revisar. Escribe guías cortas para los fallos comunes y una revisión sin culpables tras un incidente. Nunca reinicia ni cambia nada en producción por su cuenta.",
    when: "Cuando tu servicio se cae demasiado, o recibes alertas que ignoras.",
    pairs:
      "Ingeniero DevOps, para la infraestructura, y Desarrollador back-end, para las correcciones en el código.",
  },
  "release-engineer": {
    name: "Ingeniero de lanzamientos",
    role: "Planifica y prepara lanzamientos: números de versión, notas de cambios, comprobaciones antes de publicar y un camino de vuelta, sin publicar por ti",
    summary:
      "Planifica y prepara tus lanzamientos, con notas, lista de comprobación y camino de vuelta, y nunca publica solo",
    about:
      "Lee lo que cambió desde el último lanzamiento, lo agrupa por lo que notan los usuarios y propone el número de versión y las notas en palabras sencillas. Escribe una lista de comprobación, los pasos para publicar y los pasos para deshacer, y dice cuál es difícil de deshacer. La etiqueta, la subida y el anuncio los haces tú, y nunca toca tus claves de firma.",
    when: "Antes de cada lanzamiento de una app, una biblioteca o un sitio.",
    pairs: "Tester, para ejecutar la lista, e Ingeniero DevOps, para la cadena de publicación.",
  },
  "codebase-guide": {
    name: "Guía del código",
    role: "Explica un código desconocido: el panorama general, dónde está cada cosa, cómo recorre una petición el sistema y por dónde empezar a cambiar",
    summary:
      "Explica un código desconocido: el panorama general, dónde está cada cosa y por dónde empezar",
    about:
      "Adapta la visita a lo que necesitas hacer, sigue un camino real de principio a fin por archivo y función y señala las convenciones, las zonas de riesgo y las pruebas. Dice de qué está seguro y qué dedujo, y termina con una primera tarea pequeña. Solo lee; no cambia nada si no se lo pides.",
    when: "Cuando heredas código, entras en un proyecto o tienes que cambiar algo que no escribiste.",
    pairs: "Redactor técnico, para convertirlo en documentación, y Arquitecto de software.",
  },
  "prompt-engineer": {
    name: "Redactor de instrucciones para IA",
    role: "Escribe y mejora las instrucciones que das a modelos de IA y las prueba con ejemplos reales antes de que confíes en ellas",
    summary:
      "Escribe y mejora instrucciones para modelos de IA y las prueba con ejemplos reales antes de que confíes",
    about:
      "Pide ejemplos reales, también difíciles, escribe la instrucción con un objetivo claro, reglas, formato de la respuesta y ejemplos, y la prueba cambiando una cosa a la vez. Recibes una tabla de resultados de cada versión y los puntos débiles que quedan. Avisa cuando la instrucción va a leer un texto externo que podría dar órdenes.",
    when: "Cuando una función de IA en tu app o flujo da respuestas en las que no puedes confiar.",
    pairs: "Desarrollador back-end, para conectarlo, y Tester, para probar más casos.",
  },
  "rapid-prototyper": {
    name: "Creador de prototipos",
    role: "Monta rápido una versión que funciona de una idea para probarla con personas reales, de la forma más simple, y dice qué es de mentira",
    summary:
      "Monta rápido una versión de tu idea para probarla con personas reales y dice qué es de mentira",
    about:
      "Parte de la pregunta que el prototipo debe responder y construye solo eso, con datos inventados y sin cuentas reales. Escribe una prueba corta para quienes lo van a probar y una nota que dice qué es de mentira y qué necesita una versión real, para que nadie publique el prototipo por error.",
    when: "Cuando tienes una idea y quieres saber si a la gente le importa antes de construirla bien.",
    pairs: "Diseñador, para el aspecto, e Investigador de usuarios, para dirigir la prueba.",
  },
  "localization-engineer": {
    name: "Ingeniero de localización",
    role: "Prepara el software para varios idiomas y países: saca el texto del código, trata fechas, números y plurales y comprueba el resultado",
    summary: "Prepara tu software para varios idiomas y países y comprueba que nada se rompe",
    about:
      "Saca el texto de tu código a archivos de traducción, con una frase por clave y marcadores en lugar de trozos pegados. Trata plurales, fechas, números y monedas según la región del usuario, comprueba el diseño con palabras largas y crea una prueba que falla cuando falta un texto en un idioma. Entrega los textos nuevos a un traductor.",
    when: "Cuando quieres tu app o sitio en un segundo idioma, o en un país con formatos distintos.",
    pairs: "Traductor, para los textos, y Desarrollador front-end, para el diseño.",
  },
  "refactoring-engineer": {
    name: "Ingeniero de limpieza de código",
    role: "Ordena el código en pasos pequeños y seguros, sin cambiar lo que hace, demostrando cada paso con pruebas",
    summary:
      "Ordena tu código en pasos pequeños y seguros, sin cambiar lo que hace, con pruebas en cada paso",
    about:
      "Parte de un motivo, se asegura de que existan pruebas antes de tocar nada y cambia una cosa a la vez, cada una en un commit pequeño. No mezcla una limpieza con un cambio de comportamiento y nunca modifica una prueba solo para que la limpieza pase. Lista lo que vio y dejó quieto.",
    when: "Cuando el código es difícil de cambiar o de entender y quieres mejorarlo sin romperlo.",
    pairs: "Revisor de código, para comprobar cada paso, y Tester, para probar el comportamiento.",
  },
};
