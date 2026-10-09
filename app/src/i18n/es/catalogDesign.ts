import type { design as en } from "../en/catalogDesign";

export const design: typeof en = {
  "brand-guardian": {
    name: "Guardián de la marca",
    role: "Mantiene todo fiel a la marca: escribe una guía breve con tu material y revisa las piezas con ella",
    summary: "Escribe una guía breve de la marca con tu material y revisa que todo la siga",
    about:
      "Reúne tu logo, colores, fuentes y ejemplos en una guía de una página, marcando lo que vio y lo que sugiere. Luego revisa páginas, publicaciones y presentaciones con la guía y lista exactamente dónde se aparta cada una. Nunca inventa un dato de la marca ni cambia una regla sin preguntarte.",
    when: "Cuando tienes una marca y varias personas o bots crean cosas para ella.",
    pairs: "Diseñador, que aplica los ajustes visuales, y Redactor, que aplica los ajustes de voz.",
  },
  "ui-designer": {
    name: "Diseñador de interfaz",
    role: "Construye el sistema visual de una interfaz: colores, fuentes, espaciado y componentes, en una página de estilo",
    summary:
      "Construye colores, fuentes, espaciado y botones de una interfaz y los muestra en una página de estilo",
    about:
      "Elige una paleta pequeña, una escala de fuentes y un paso de espaciado, y construye botones, campos, tarjetas y avisos con todos sus estados en una página de estilo que ves. Cada pantalla siguiente reutiliza esas piezas, y el producto parece una sola cosa.",
    when: "Al empezar un producto, o cuando tus pantallas dejaron de parecer de la misma familia.",
    pairs:
      "Diseñador, que hace las páginas, Desarrollador front-end, que las construye, y Revisor de accesibilidad.",
  },
  "ux-architect": {
    name: "Arquitecto de experiencia",
    role: "Planifica cómo se organiza un producto y cómo se mueve la gente por él: mapas, flujos y bocetos simples",
    summary:
      "Planifica cómo se organiza un producto y cómo lo recorre la gente, con flujos y bocetos simples",
    about:
      "Parte de quiénes son las personas y qué quieren hacer, agrupa el contenido como ellas lo piensan y dibuja cada flujo, con errores, pantallas vacías y el volver atrás. Los bocetos son pantallas grises y simples, para que juzgues la estructura antes de cualquier adorno. Lista los lugares donde la gente puede perderse.",
    when: "Antes de diseñar o construir un producto, un sitio o una novedad grande.",
    pairs:
      "Investigador de usuarios, para probar las hipótesis, y Diseñador de interfaz, que da aspecto a los bocetos.",
  },
  "accessibility-reviewer": {
    name: "Revisor de accesibilidad",
    role: "Comprueba que páginas y diseños sirvan a personas con distintas capacidades y explica cada arreglo en palabras sencillas",
    summary:
      "Comprueba que tus páginas funcionen para personas con distintas capacidades y explica cada arreglo",
    about:
      "Lee el código de una página y comprueba lo que se puede ver en él: contraste, descripción de imágenes, etiquetas de campos, títulos, uso con teclado, foco visible e información dada solo por el color. Cada problema trae a quién afecta y el arreglo exacto. Dice lo que no puede comprobar, como un lector de pantalla real, y nunca llama certificada a una página.",
    when: "Antes de lanzar una página y después de un cambio grande en ella.",
    pairs:
      "Desarrollador front-end, que aplica los arreglos, y Diseñador de interfaz, que lo planifica desde el principio.",
  },
  "presentation-designer": {
    name: "Diseñador de presentaciones",
    role: "Convierte tu mensaje en una presentación clara: la historia, una idea por diapositiva y un aspecto limpio, en pantallas",
    summary:
      "Convierte lo que quieres decir en una presentación clara, una idea por diapositiva, que ves nacer",
    about:
      "Escribe primero la historia y luego diseña cada diapositiva como una pantalla que ves aparecer en el área de diseño: una idea, un título que dice el punto, texto grande y como mucho una imagen o gráfico. Los detalles van a las notas del orador. Las diapositivas son pantallas, no un archivo de PowerPoint.",
    when: "Cuando tienes que presentar, proponer o enseñar algo y quieres claridad.",
    pairs:
      "Investigador, para los datos, Redactor, para las palabras, y Diseñador de infografías, para los gráficos.",
  },
  "infographic-designer": {
    name: "Diseñador de infografías",
    role: "Convierte números e ideas en gráficos e infografías claros, dibujados como pantallas, sin torcer los datos",
    summary: "Convierte números e ideas en gráficos e infografías claros, sin torcer los datos",
    about:
      "Comprueba de dónde vienen los números, elige la forma que aclara el mensaje y la dibuja como pantalla: escalas honestas, etiquetas directas, pocos colores, la fuente y la fecha en la imagen. Dice lo que la imagen no muestra y nunca inventa un número para tapar un hueco.",
    when: "Cuando una tabla o un informe es difícil de leer y una imagen lo resolvería mejor.",
    pairs:
      "Analista de datos, para los números, y Diseñador de presentaciones, para llevarlo a una presentación.",
  },
  "image-prompt-writer": {
    name: "Redactor de descripciones de imagen",
    role: "Escribe descripciones claras y coherentes para pegar en un generador de imágenes o vídeo y las mejora con los resultados",
    summary:
      "Escribe descripciones claras y coherentes para el generador de imágenes o vídeo que usas y las mejora",
    about:
      "No hace las imágenes: escribe las descripciones que pegas en la herramienta que usas, con el tema, el escenario, el estilo, la luz y el encuadre, en dos o tres versiones. Para un conjunto que debe verse igual, escribe un párrafo de estilo para repetir. Enséñale el resultado y cambia solo la parte que causó el problema.",
    when: "Cuando usas un generador de imágenes o vídeo y los resultados salen al azar o distintos entre sí.",
    pairs:
      "Guardián de la marca, para mantener el estilo de la marca, y Redes sociales, para las publicaciones.",
  },
  "design-critic": {
    name: "Crítico de diseño",
    role: "Revisa pantallas y páginas con ojos nuevos y entrega una lista corta y ordenada de qué mejorar y por qué",
    summary: "Mira tus pantallas con ojos nuevos y te dice, en orden, qué mejorar y por qué",
    about:
      "Lee el objetivo y los archivos de la pantalla y te dice primero las tres cosas que más importan: qué ve primero el ojo, si la acción principal está clara, el espaciado, las fuentes, el contraste y cómo funciona en una pantalla pequeña. Separa lo que es un problema claro de lo que es gusto, dice qué conservar y no rehace la pantalla si no se lo pides.",
    when: "Cuando una pantalla se siente rara y no sabes decir por qué, o antes de publicar.",
    pairs: "Diseñador, que hace los cambios, y Revisor de accesibilidad, para el acceso.",
  },
};
