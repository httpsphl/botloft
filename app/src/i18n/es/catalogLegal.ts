import type { legal as en } from "../en/catalogLegal";

export const legal: typeof en = {
  "contract-reader": {
    name: "Lector de contratos",
    role: "Lee un contrato en palabras sencillas: qué debe hacer cada parte, las fechas y el dinero, las cláusulas arriesgadas y las preguntas para un abogado",
    summary:
      "Lee un contrato en palabras sencillas y señala las cláusulas arriesgadas y qué preguntar a un abogado",
    about:
      "Lee el contrato entero y te da una página: quién debe hacer qué, el dinero, las fechas y cómo termina. Señala las cláusulas que merecen atención, como renovación automática, penalizaciones o renuncia de derechos, y marca cada una como común, inusual o que vale la pena negociar. Nunca dice que una cláusula es válida o exigible. No es abogado y termina con las preguntas para llevar a uno.",
    when: "Antes de firmar un contrato, o cuando quieres entender uno que ya firmaste.",
    pairs:
      "Control de plazos de contratos, para las fechas, y Asistente de investigación jurídica.",
  },
  "terms-drafter": {
    name: "Redactor de términos y privacidad",
    role: "Redacta los términos de uso y el aviso de privacidad de un sitio o app en palabras sencillas, según cómo funciona de verdad el negocio, para que un abogado los revise",
    summary:
      "Redacta los términos y el aviso de privacidad de tu sitio en palabras sencillas, para que un abogado los revise",
    about:
      "Pregunta cómo funciona de verdad tu negocio y qué datos recoge, y luego redacta los términos de uso, el aviso de privacidad y la política de cambios y envíos en frases cortas y sencillas. Los pasajes que dependen de la ley quedan como preguntas para un abogado, y lista las promesas que tendrás que cumplir. Nunca publica nada en tu sitio.",
    when: "Cuando lanzas un sitio, una app o una tienda y necesitas los textos que la acompañan.",
    pairs: "Ingeniero de privacidad, para el mapa de datos, y Corrector de textos, para pulir.",
  },
  "legal-research-assistant": {
    name: "Asistente de investigación jurídica",
    role: "Encuentra y explica las leyes, normas y decisiones sobre una cuestión en fuentes oficiales, con citas exactas, y dice qué no pudo verificar",
    summary:
      "Encuentra las leyes y decisiones sobre una cuestión en fuentes oficiales, con citas exactas y límites honestos",
    about:
      "Necesita el lugar, la fecha y los hechos, y luego busca primero en fuentes oficiales, abre cada fuente antes de citarla y te entrega un memorando corto con referencias exactas. Separa lo que dice la ley de lo que dicen los tribunales sobre ella y de lo que sigue abierto. Nunca inventa una ley ni un caso, y dice cuando no pudo verificar uno.",
    when: "Cuando necesitas saber qué dice la ley sobre una cuestión, antes de hablar con un abogado.",
    pairs:
      "Verificador de datos, para comprobar una cita, y Corrector de textos, para pulir el memorando.",
  },
  "legal-intake-assistant": {
    name: "Asistente de admisión de clientes",
    role: "Ayuda a un pequeño despacho o asesor a reunir los hechos de un cliente nuevo: un cuestionario claro, un resumen ordenado del asunto y los documentos que faltan",
    summary:
      "Ayuda a un pequeño despacho a reunir los hechos de un cliente nuevo: cuestionario claro, resumen ordenado y lo que falta",
    about:
      "Prepara un formulario sencillo para el tipo de asunto y luego convierte las respuestas del cliente en una línea de tiempo y un resumen de una página, con los hechos separados de las opiniones. Lista los documentos que faltan y pone lo urgente arriba. Nunca dice al cliente cuáles son sus derechos ni si tiene un caso, y nunca lo contacta.",
    when: "En el primer paso con un cliente nuevo, antes de que el abogado revise el asunto.",
    pairs: "Asistente de horas y facturación, y Lector de contratos.",
  },
  "legal-billing-assistant": {
    name: "Asistente de horas y facturación",
    role: "Convierte tu registro de trabajo en partes de horas claros y borradores de factura para un despacho o consultor, y muestra lo que aún no se ha facturado",
    summary:
      "Convierte tu registro de trabajo en partes de horas y borradores de factura, y muestra lo que falta por facturar",
    about:
      "Convierte tus notas y tu agenda en partes de horas con descripciones que un cliente entendería y que no revelan más de lo necesario. Muestra lo que aún no se ha facturado por cliente, señala partes que parecen erróneos sin cambiarlos y prepara el borrador de la factura con los partes que apruebes. Nunca redondea hacia arriba ni exagera el trabajo.",
    when: "Cuando facturas por horas y el tiempo se escapa antes de que factures.",
    pairs: "Asistente de cobros, para seguir los pagos, y Auxiliar contable.",
  },
  "contract-deadline-tracker": {
    name: "Control de plazos de contratos",
    role: "Sigue las fechas de tus contratos: renovaciones, preavisos, pagos y plazos, y te avisa con mucha antelación",
    summary:
      "Sigue las fechas de tus contratos, como renovaciones y preavisos, y te avisa con antelación",
    about:
      "Lee tus contratos y lista cada fecha y obligación con la cláusula de la que viene, calculando la fecha real y diciendo en palabras sencillas cómo se contó. Pone las trampas primero, como renovaciones automáticas con preaviso, y avisa pronto. Si una cláusula no está clara, lo dice en lugar de adivinar. Nunca envía un aviso ni termina un contrato.",
    when: "Cuando tienes varios contratos y temes perder una renovación o un aviso.",
    pairs:
      "Lector de contratos, para explicar una cláusula, y Redactor, para redactar un aviso que tú envías.",
  },
  "complaint-letter-drafter": {
    name: "Redactor de cartas de reclamación",
    role: "Redacta cartas claras, firmes y educadas para una queja o disputa con una empresa o un propietario, a partir de los hechos y los papeles que tienes",
    summary:
      "Redacta cartas de reclamación claras, firmes y educadas para una disputa con una empresa o propietario",
    about:
      "Ordena los hechos contigo, arma una línea de tiempo comprobada con tus papeles y redacta una carta corta, educada y firme: qué pasó, qué pides y un plazo razonable para responder. No amenaza ni exagera. Lista los anexos y los lugares donde probar después, como preguntas para tu país. La carta la envías tú.",
    when: "Cuando una empresa, un propietario o un vendedor no ha actuado bien contigo.",
    pairs: "Lector de contratos, para las cláusulas, y Corrector de textos, para revisar.",
  },
};
