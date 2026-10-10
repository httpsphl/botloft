import type { engineering as en } from "../en/catalogEngineering";

export const engineering: typeof en = {
  "software-architect": {
    name: "Arquitecto de software",
    role: "Diseña cómo encaja un sistema y registra las decisiones, los intercambios y los riesgos",
    summary: "Diseña cómo encajan las partes de un sistema y escribe el porqué",
    about:
      "Decide contigo cómo debe estructurarse un software, elige el diseño más simple que cubre lo que necesitas y escribe cada decisión importante con sus motivos, costos y riesgos. Diseña; no escribe el código del producto.",
    when: "Antes de construir algo grande, o cuando un sistema se volvió difícil de cambiar.",
    pairs:
      "Desarrollador, Desarrollador front-end, back-end y móvil, que construyen a partir del diseño, y Revisor de seguridad.",
  },
  "frontend-developer": {
    name: "Desarrollador front-end",
    role: "Construye y arregla la parte de un sitio o app que la gente ve y usa, rápida en el móvil y usable por todos",
    summary: "Construye las pantallas que la gente usa, rápidas en el móvil y usables por todos",
    about:
      "Construye y arregla páginas, formularios y menús, pensando primero en el móvil, el teclado, los lectores de pantalla y las conexiones lentas. Comprueba su trabajo en un navegador de verdad y ejecuta las pruebas del proyecto.",
    when: "Cuando un sitio o una app necesita una pantalla nueva, o lo que existe es lento o confuso.",
    pairs: "Diseñador, que dibuja las pantallas, y Desarrollador back-end, que da los datos.",
  },
  "backend-developer": {
    name: "Desarrollador back-end",
    role: "Construye y arregla el lado del servidor: APIs, manejo de datos, tareas en segundo plano y sus pruebas",
    summary:
      "Construye la parte que está detrás de la pantalla: APIs, datos y tareas en segundo plano, con pruebas",
    about:
      "Construye y arregla lo que corre detrás de la pantalla: servidores, APIs, reglas sobre datos y tareas, con pruebas y con atención a lo que pasa cuando algo falla. Nunca ejecuta cambios en datos reales sin preguntarte.",
    when: "Cuando tu producto necesita guardar, procesar o compartir datos, o conectarse a otro servicio.",
    pairs:
      "Ingeniero de bases de datos, para los datos, y Desarrollador front-end, que usa las APIs.",
  },
  "mobile-developer": {
    name: "Desarrollador móvil",
    role: "Construye y arregla apps de móvil y las comprueba en pantallas, redes y dispositivos realistas",
    summary:
      "Construye y arregla apps de móvil, pensando en pantallas pequeñas, redes lentas y batería",
    about:
      "Construye y arregla apps de móvil pensando en pantallas pequeñas, redes malas, interrupciones y permisos. Te dice con honestidad lo que no pudo probar sin un dispositivo real, y nunca toca tus cuentas de tienda ni tus claves de firma.",
    when: "Cuando quieres una app para móviles, o la que tienes es lenta o se cierra sola.",
    pairs: "Diseñador, para las pantallas, Desarrollador back-end, para la API, y Tester.",
  },
  "database-engineer": {
    name: "Ingeniero de bases de datos",
    role: "Diseña tablas, escribe y acelera consultas y planifica cambios seguros en una base de datos",
    summary: "Diseña tu base de datos, hace rápidas las consultas y cambia datos sin perder nada",
    about:
      "Diseña cómo se guardan tus datos, acelera las consultas lentas midiendo antes y planifica cada cambio para poder deshacerlo, con copia de seguridad primero. Prueba en una copia y nunca ejecuta nada destructivo sobre datos reales.",
    when: "Cuando una base de datos está lenta, desordenada o tiene que cambiar sin perder nada.",
    pairs:
      "Desarrollador back-end, que aplica los cambios, e Ingeniero de datos, que carga los datos.",
  },
  "devops-engineer": {
    name: "Ingeniero DevOps",
    role: "Monta compilaciones, pruebas, despliegues y monitoreo, y mantiene sanos los procesos y los servidores",
    summary: "Automatiza compilar, probar y publicar, y mantiene todo funcionando y vigilado",
    about:
      "Hace que publicar software sea seguro y sin drama: compilaciones y pruebas automáticas, despliegues repetibles, registros y alertas, y una forma de volver atrás en cada versión. Pregunta antes de tocar cualquier sistema real y nunca te pide pegar contraseñas en el chat.",
    when: "Cuando publicar es manual y da miedo, o algo se rompe siempre en producción.",
    pairs:
      "Desarrollador back-end, para lo que el servicio necesita, y Revisor de seguridad, para accesos y secretos.",
  },
  "security-reviewer": {
    name: "Revisor de seguridad",
    role: "Revisa tu propio código y tu configuración en busca de fallos de seguridad y explica cómo corregirlos",
    summary: "Busca fallos de seguridad en tu código y tu configuración y explica cómo corregirlos",
    about:
      "Revisa tu código y tu configuración en busca de fallos, explica cada uno con palabras sencillas, su gravedad y la corrección más pequeña. Trabaja a la defensiva, solo sobre lo que es tuyo, y nunca repite un secreto que encuentre. Piensa más a fondo que la mayoría de los agentes, así que gasta un poco más de tu plan.",
    when: "Antes de lanzar, después de un cambio grande, o cuando manejas datos de otras personas.",
    pairs: "Desarrollador e Ingeniero DevOps, que aplican las correcciones, y Revisor de código.",
  },
  "data-engineer": {
    name: "Ingeniero de datos",
    role: "Construye procesos fiables que mueven, limpian y guardan datos para que los analistas puedan confiar en ellos",
    summary: "Construye las tuberías que llevan datos limpios y fiables hasta donde los analizas",
    about:
      "Construye los caminos que recorren los datos desde donde nacen hasta donde se analizan: los recoge, limpia y comprueba en cada paso y se detiene cuando algo parece mal, en vez de pasar números malos. Mantiene tus originales intactos.",
    when: "Cuando tus números están en muchos sitios o no confías en tus informes.",
    pairs:
      "Ingeniero de bases de datos, para dónde guardar, y Analista de datos, que usa el resultado.",
  },
};
