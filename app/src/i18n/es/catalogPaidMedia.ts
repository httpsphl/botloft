import type { paidMedia as en } from "../en/catalogPaidMedia";

export const paidMedia: typeof en = {
  "ppc-strategist": {
    name: "Estratega de anuncios de búsqueda",
    role: "Planifica campañas de anuncios en buscadores: las palabras, la estructura, los anuncios y un presupuesto pequeño para probar, en un plan que tú ejecutas",
    summary:
      "Planifica tus anuncios de búsqueda, de las palabras a los anuncios, con un presupuesto pequeño para probar primero",
    about:
      "Parte de lo que vale un cliente para ti, arma los grupos de palabras por lo que la gente quiere, escribe los anuncios con tu oferta real y define una prueba pequeña con reglas claras de cuándo parar y cuándo subir. Nunca crea ni paga un anuncio: lo haces tú, con su plan.",
    when: "Cuando quieres empezar con anuncios de búsqueda, o los que tienes no dan resultado.",
    pairs:
      "Redactor publicitario, para afinar los anuncios, y Analista de búsquedas, para leer los términos buscados.",
  },
  "paid-social-strategist": {
    name: "Estratega de anuncios en redes sociales",
    role: "Planifica campañas de pago en redes sociales: el público, los formatos, lo que hay que crear y un presupuesto de prueba, sin segmentación sensible",
    summary:
      "Planifica tus campañas de pago en redes: a quién llegar, qué mostrar y cómo probarlo con poco dinero",
    about:
      "Elige un solo objetivo, define a quién llegar por intereses, lugar y las listas de clientes que decidas usar, y planifica los formatos y las variantes a probar. Nunca segmenta por rasgos sensibles como la salud o la religión ni se aprovecha de inseguridades. La campaña la creas y la pagas tú.",
    when: "Cuando quieres llegar a gente nueva en las redes sin desperdiciar el presupuesto.",
    pairs:
      "Estratega de creatividades de anuncios, para qué decir y mostrar, y Guionista de video, para videos cortos.",
  },
  "ad-creative-strategist": {
    name: "Estratega de creatividades de anuncios",
    role: "Planifica qué deben decir y mostrar los anuncios: ángulos, ganchos y variantes para probar, y aprende de lo que funciona",
    summary:
      "Planifica qué deben decir y mostrar tus anuncios, con algunos ángulos para probar y lo que enseña cada uno",
    about:
      "Toma las palabras que usan tus clientes y escribe tres o cuatro ángulos distintos, cada uno con el gancho, el texto y una guía para la imagen o el video. Cambia una sola cosa por prueba, para que el resultado diga por qué ganó. Solo usa afirmaciones que puedes demostrar: nada de reseñas inventadas ni falsa escasez.",
    when: "Cuando tus anuncios se parecen todos, o no sabes qué probar a continuación.",
    pairs:
      "Diseñador, para las imágenes, Redactor publicitario, para pulir, y Estratega de anuncios en redes sociales.",
  },
  "ad-auditor": {
    name: "Auditor de cuentas de anuncios",
    role: "Revisa cómo están montadas y cómo funcionan tus campañas de anuncios, encuentra dinero desperdiciado y oportunidades perdidas y lista los arreglos en orden",
    summary:
      "Revisa tus campañas de anuncios, encuentra dinero desperdiciado y oportunidades perdidas y lista los arreglos en orden",
    about:
      "Lee los informes que le des y encuentra las fugas de siempre: gasto sin resultados, gente alcanzada dos veces, anuncios cansados, búsquedas que no encajan y resultados mal contados. Cada hallazgo trae los números, el dinero en juego y el arreglo, en orden. No tiene acceso a tu cuenta y no cambia nada en ella.",
    when: "Cuando el gasto en anuncios crece, pero no estás seguro de que funcione.",
    pairs:
      "Especialista en medición, para comprobar los conteos, y Estratega de anuncios de búsqueda, para aplicar los arreglos.",
  },
  "tracking-specialist": {
    name: "Especialista en medición",
    role: "Comprueba que tus anuncios y tu sitio midan bien los resultados: eventos, etiquetas, enlaces y qué cuenta como resultado",
    summary:
      "Comprueba que tus anuncios y tu sitio cuenten bien los resultados, para que confíes en los números",
    about:
      "Lista cada lugar donde se cuenta un resultado y los compara: plataforma de anuncios, sitio, tienda y hoja de cálculo. Explica cada diferencia en palabras sencillas, como el conteo doble o los enlaces que pierden su etiqueta, y propone un nombre limpio para eventos y enlaces. Dice hasta dónde se puede confiar en los números y no recoge de los visitantes más de lo que necesitas.",
    when: "Antes de gastar en anuncios, o cuando los números de distintos sitios no coinciden.",
    pairs: "Desarrollador front-end, que pone las etiquetas, y Auditor de cuentas de anuncios.",
  },
  "search-query-analyst": {
    name: "Analista de búsquedas",
    role: "Lee las búsquedas reales detrás de tus anuncios o tu sitio, las agrupa por intención y encuentra qué añadir, excluir o escribir",
    summary:
      "Lee las búsquedas reales detrás de tus anuncios y tu sitio y dice qué añadir, excluir o escribir",
    about:
      "Toma las palabras que la gente realmente escribió y las agrupa por lo que quiere: comprar, comparar, aprender. Entrega tres listas: qué excluir porque gasta dinero sin sentido, qué añadir porque funciona y sobre qué escribir porque la gente pregunta. Dice qué tan seguro está, ya que un término con tres clics no prueba nada.",
    when: "Cuando haces anuncios de búsqueda o quieres saber qué busca la gente antes de encontrarte.",
    pairs:
      "Estratega de anuncios de búsqueda, que usa las listas, y Especialista en SEO, para las ideas de contenido.",
  },
};
