import type { sectors as en } from "../en/catalogSectors";

export const sectors: typeof en = {
  "real-estate-assistant": {
    name: "Asistente inmobiliario",
    role: "Te ayuda a comprar, vender o alquilar un inmueble: compara anuncios, prepara preguntas y listas y redacta anuncios y mensajes",
    summary:
      "Te ayuda a comprar, vender o alquilar un inmueble: compara anuncios, prepara preguntas y redacta el anuncio",
    about:
      "Compara los inmuebles en una sola tabla, con los mismos datos de cada uno, incluidos los costos reales, y te dice lo que el anuncio omite. Prepara listas y preguntas para las visitas y ayuda a escribir un anuncio honesto y preguntas para compradores o inquilinos. Nunca afirma un valor de mercado como un hecho, hace una oferta ni firma, y no es agente ni abogado.",
    when: "Cuando buscas un inmueble, o preparas el tuyo para vender o alquilar.",
    pairs: "Investigador, para datos de una zona, y Redactor, para pulir el anuncio.",
  },
  "hospitality-guest-assistant": {
    name: "Asistente de alojamiento",
    role: "Ayuda a un hotel, posada o anfitrión de alquiler a atender a los huéspedes: respuestas, mensajes de bienvenida, guías y qué mejorar",
    summary:
      "Ayuda a un hotel o anfitrión a atender a los huéspedes: respuestas, bienvenida, guías y mejoras",
    about:
      "Mantiene una hoja corta de datos de tu lugar y redacta las respuestas a los huéspedes a partir de ella, sin inventar un horario, un precio ni una promesa. Escribe el mensaje de bienvenida y la guía del huésped, redacta respuestas tranquilas a las quejas y lee tus reseñas para ver qué les gusta a los huéspedes y qué se repite como problema. Los reembolsos y descuentos siguen siendo decisión tuya.",
    when: "Cuando los huéspedes preguntan lo mismo todo el día, o quieres mejores reseñas.",
    pairs: "Traductor, para huéspedes que hablan otro idioma, y Redes sociales.",
  },
  "returns-assistant": {
    name: "Asistente de devoluciones y reembolsos",
    role: "Gestiona los pedidos de cambio, devolución y reembolso según tu política: los clasifica, redacta las respuestas y detecta patrones y abusos",
    summary:
      "Gestiona los pedidos de devolución y reembolso según tu política: los clasifica, redacta respuestas y detecta patrones",
    about:
      "Lee cada pedido frente a tu política escrita y lo clasifica como dentro, fuera o duda, y luego redacta una respuesta clara y amable. Las dudas y excepciones te llegan a ti. Cada semana muestra lo que se repite, como productos con muchas devoluciones, sin acusar a ningún cliente. Avisa cuando un cliente puede tener derecho legal a un reembolso. Nunca hace un reembolso por su cuenta.",
    when: "Cuando los pedidos de devolución y reembolso te quitan demasiado tiempo.",
    pairs:
      "Atención al cliente, para el tono, y Gestor de comercio electrónico, para los arreglos en los anuncios.",
  },
  "supply-chain-planner": {
    name: "Planificador de compras y existencias",
    role: "Planifica qué comprar y cuándo: niveles de existencias, puntos de reposición, comparación de proveedores y riesgos de entrega, para un negocio pequeño",
    summary:
      "Planifica qué comprar y cuándo: existencias, puntos de reposición y comparación de proveedores para un negocio pequeño",
    about:
      "Trabaja con tus ventas, existencias, precios y plazos para proponer un punto de reposición y un tamaño de pedido para cada artículo, mostrando la fórmula. Marca los artículos que no pueden agotarse y los parados que atan dinero, compara proveedores por el costo total y lista los riesgos con un plan B. Nunca hace un pedido ni contacta a un proveedor.",
    when: "Cuando te quedas sin existencias, o guardas demasiado de lo que no se vende.",
    pairs:
      "Auxiliar contable, para los costos, y Previsor de flujo de caja, para lo que puedes pagar.",
  },
  "grant-writer": {
    name: "Redactor de subvenciones y convocatorias",
    role: "Encuentra y prepara solicitudes a subvenciones y convocatorias públicas: lee las bases, planifica la respuesta y la redacta con tus datos reales",
    summary:
      "Prepara solicitudes a subvenciones y convocatorias públicas: lee las bases, planifica la respuesta y la redacta",
    about:
      "Lee la convocatoria completa, la resume con el plazo en primer lugar y comprueba con honestidad si cumples los requisitos antes de escribir nada. Luego planifica la solicitud frente a la puntuación y redacta cada parte con tus datos reales, marcando cada hueco como pregunta. Nunca inventa un resultado, un socio ni una cifra, y nunca presenta nada por ti.",
    when: "Cuando encuentras una subvención, una convocatoria pública o un programa de financiación que podría encajar.",
    pairs: "Analista financiero, para el presupuesto, y Corrector de textos, para pulir.",
  },
  "restaurant-manager": {
    name: "Asistente de restaurante",
    role: "Ayuda a llevar un restaurante o cafetería pequeños: carta y costos, turnos, pedidos y compras, y respuestas a los clientes",
    summary:
      "Ayuda a llevar un restaurante o cafetería pequeños: costo de la carta, turnos, compras y respuestas a los clientes",
    about:
      "Calcula cuánto cuesta realmente cada plato y dónde pierdes dinero, planifica los turnos según tus horas punta, mantiene la lista de la compra y una hoja diaria que muestra lo que se repite. Nunca dice que un plato no lleva un alérgeno o sirve para una dieta por intuición: eso sale de tus fichas de receta, y cuando no lo dicen, te manda comprobar antes.",
    when: "Cuando llevas un pequeño negocio de comida y el papeleo te quita el tiempo.",
    pairs:
      "Auxiliar contable, para los costos, y Redes sociales, para publicaciones sobre la carta.",
  },
  "course-creator": {
    name: "Creador de cursos en línea",
    role: "Te ayuda a convertir lo que sabes en un curso o taller en línea: la promesa, el esquema, las lecciones, los ejercicios y la página que lo presenta",
    summary:
      "Te ayuda a convertir lo que sabes en un curso en línea: esquema, lecciones, ejercicios y la página de venta",
    about:
      "Parte del alumno y de una promesa honesta que puedes cumplir, construye el esquema de atrás hacia adelante desde el resultado y redacta las lecciones y los ejercicios con tu voz y tus ejemplos. La página del curso usa solo contenido y resultados reales: nada de testimonios inventados, plazos falsos ni promesas de ganancias. Tú publicas y fijas el precio.",
    when: "Cuando sabes algo que la gente pagaría por aprender y quieres enseñarlo.",
    pairs:
      "Diseñador de presentaciones, para las diapositivas, Guionista de video, para los guiones, y Corrector de textos, para revisar.",
  },
};
