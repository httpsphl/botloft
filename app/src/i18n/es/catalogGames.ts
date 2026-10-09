import type { games as en } from "../en/catalogGames";

export const games: typeof en = {
  "game-designer": {
    name: "Diseñador de juegos",
    role: "Convierte una idea de juego en reglas claras y un ciclo divertido de repetir: el objetivo, las decisiones, la respuesta y una primera versión pequeña para probar",
    summary:
      "Convierte tu idea de juego en reglas claras y un ciclo divertido, con una primera versión pequeña para probar",
    about:
      "Parte de la experiencia que quieres que tenga el jugador, encuentra el ciclo que repite y escribe las reglas en frases cortas y exactas, comprobando si hay resquicios. Planifica la versión jugable más pequeña y una prueba, y cuando algo no es divertido ofrece dos cambios para intentar. El juego sigue siendo tuyo.",
    when: "Cuando tienes una idea de juego, digital o de mesa, y quieres que se convierta en algo jugable.",
    pairs:
      "Diseñador de narrativa, para la historia, y Diseñador de economía de juegos, para los números.",
  },
  "game-narrative-designer": {
    name: "Diseñador de narrativa",
    role: "Crea la historia, los personajes y el mundo de un juego, y escribe los textos para que encajen con la forma de jugar",
    summary:
      "Crea la historia, los personajes y el mundo de tu juego y escribe los textos para que encajen con el juego",
    about:
      "Arma el mundo en una página, crea personajes con un deseo, un defecto y una voz y planifica la historia en torno a las decisiones del jugador y la duración del juego. Escribe los textos en líneas cortas con su contexto, para que un desarrollador los coloque. Todo es original, y te dice para qué edad sirven los temas sensibles.",
    when: "Cuando tu juego necesita un mundo, una historia, misiones o personajes que la gente recuerde.",
    pairs: "Diseñador de juegos, para las reglas, y Traductor, para otros idiomas.",
  },
  "level-designer": {
    name: "Diseñador de niveles",
    role: "Diseña los niveles, mapas o escenarios de un juego para que enseñen, desafíen y sorprendan en el orden correcto",
    summary:
      "Diseña los niveles, mapas o escenarios de tu juego para que enseñen, desafíen y sorprendan en orden",
    about:
      "Planifica el orden en que el jugador aprende cada mecánica, una novedad a la vez, y describe cada nivel con un mapa en texto: el inicio, el objetivo, la ruta, los desafíos y las recompensas. Comprueba si hay sitios donde el jugador puede atascarse o saltarse el sentido del nivel, y escribe una lista de prueba para cada uno.",
    when: "Cuando tienes las reglas de un juego y necesitas los niveles o escenarios que las usan.",
    pairs:
      "Diseñador de juegos, para revisar las reglas, y Diseñador de narrativa, para los momentos de la historia.",
  },
  "game-economy-designer": {
    name: "Diseñador de economía de juegos",
    role: "Equilibra los números de un juego: costos, recompensas, progreso y precios, para que sea justo e interesante, sin engañar al jugador para que gaste",
    summary:
      "Equilibra los números de tu juego: costos, recompensas y progreso, de forma justa y sin trucos",
    about:
      "Dibuja los flujos de monedas, objetos y tiempo, pone los números en una hoja de cálculo con fórmulas y simula jugadores a distintos ritmos. Encuentra recursos que se acumulan y caminos que siempre son los mejores. Si tu juego gana dinero, propone formas justas, y nunca costos ocultos, cuentas atrás falsas, ventaja por pago ni premios aleatorios de pago dirigidos a niños.",
    when: "Cuando tu juego tiene monedas, objetos o progreso y los números parecen raros, o piensas ganar dinero con él.",
    pairs: "Diseñador de juegos, para los objetivos, y Analista de datos, para las simulaciones.",
  },
  "playtest-analyst": {
    name: "Analista de pruebas de juego",
    role: "Planifica pruebas con jugadores y convierte lo que hicieron y dijeron en hallazgos claros y en los cambios que más valen la pena",
    summary:
      "Planifica pruebas con jugadores y convierte lo que hicieron y dijeron en hallazgos y cambios que valen la pena",
    about:
      "Planifica la sesión para que no expliques ni defiendas el juego y luego convierte tus notas en hallazgos: el problema, cuántos jugadores lo encontraron, dónde y por qué. Se fía más de lo que hicieron los jugadores que de lo que dijeron, ordena los cambios por efecto frente al esfuerzo y sugiere la siguiente prueba. Nunca contacta a los jugadores.",
    when: "Después de que personas hayan probado tu juego y no sepas qué cambiar primero.",
    pairs: "Diseñador de juegos, que actúa sobre los hallazgos, y Diseñador de niveles.",
  },
  "game-audio-director": {
    name: "Redactor de guías de audio para juegos",
    role: "Planifica el sonido y la música de un juego y escribe guías claras para quien los hace: el ambiente, la lista de sonidos y cómo suenan",
    summary:
      "Planifica el sonido y la música de tu juego y escribe guías claras para quien los va a hacer",
    about:
      "Planifica la música y la lista de sonidos en una tabla: cuándo suena cada uno, cómo debe sentirse y qué debe entender el jugador con él. Planifica la mezcla y avisos visuales para quien no oye bien, y escribe una guía por pieza para que un compositor o diseñador de sonido empiece. No hace el audio y te recuerda comprobar los derechos.",
    when: "Cuando tu juego ya es jugable y necesita sonido, o vas a contratar a un compositor.",
    pairs:
      "Diseñador de juegos, para los momentos que piden sonido, y Desarrollador, para cómo se activan los sonidos.",
  },
  "tabletop-game-master": {
    name: "Director de juegos de rol de mesa",
    role: "Ayuda a dirigir un juego de rol de mesa: prepara aventuras, personajes y respuestas sobre reglas, y mantiene la historia en marcha en la mesa",
    summary:
      "Ayuda a dirigir un juego de rol de mesa: aventuras, personajes, respuestas sobre reglas e ideas para la mesa",
    about:
      "Pregunta por tu sistema, tu grupo y el tono, y prepara una aventura corta con un gancho, escenas con una decisión cada una y varios finales, además de personajes rápidos. Responde dudas de reglas con el libro que tienes y dice cuando no está seguro. Durante la partida da tres opciones cuando te atascas y pregunta al grupo por sus límites.",
    when: "Cuando diriges o juegas un juego de rol y quieres ayuda para preparar o improvisar.",
    pairs:
      "Diseñador de narrativa, para un trasfondo más profundo, y Diseñador, para mapas y materiales.",
  },
};
