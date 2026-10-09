import type { health as en } from "../en/catalogHealth";

export const health: typeof en = {
  "appointment-prep-assistant": {
    name: "Asistente de consultas médicas",
    role: "Te ayuda a aprovechar una consulta médica: un historial claro, un diario de síntomas, tus preguntas en orden y una nota sencilla de lo que se dijo",
    summary:
      "Te ayuda a aprovechar una consulta médica: tu historial, tus preguntas y una nota sencilla después",
    about:
      "Convierte lo que ha estado pasando en un resumen de una página que el profesional lee en un minuto, con una línea de tiempo, tus medicamentos tal como están escritos en el envase y las preguntas por orden de importancia. Tras la consulta te ayuda a anotar lo que se dijo y nada más. No adivina qué significa un síntoma. No es médico.",
    when: "Antes de cualquier consulta con un médico u otro profesional de la salud, para ti o para alguien a quien cuidas.",
    pairs:
      "Organizador de documentos de salud, para tus papeles, y Organizador del cuidado familiar.",
  },
  "elder-care-companion": {
    name: "Organizador del cuidado familiar",
    role: "Ayuda a una familia a cuidar de un familiar mayor o enfermo: citas, quién hace qué, papeles, preguntas para el equipo de cuidado y los límites de quien cuida",
    summary:
      "Ayuda a una familia a cuidar de un familiar mayor o enfermo: citas, tareas, papeles y preguntas para el equipo",
    about:
      "Mantiene un calendario de citas y renovaciones, un plan semanal de quién hace qué y un resumen de una página para compartir con el equipo de cuidado. Lista los medicamentos exactamente como están escritos y nunca te dice que cambies uno: esas preguntas van al médico o al farmacéutico. También cuida de ti, que cuidas, y ayuda en las conversaciones difíciles de la familia.",
    when: "Cuando cuidas de un padre, una madre u otro familiar y las tareas se acumulan.",
    pairs:
      "Asistente de consultas médicas, para cada cita, y Ayudante de facturas médicas, para los costos.",
  },
  "clinic-front-desk-assistant": {
    name: "Asistente de recepción de clínica",
    role: "Ayuda a una clínica o consulta pequeña a responder a los pacientes: horarios, cómo reservar, preparación de las visitas y respuestas amables, sin dar consejo médico",
    summary:
      "Ayuda a una clínica pequeña a responder a los pacientes: horarios, reservas, preparación y respuestas amables, sin consejo médico",
    about:
      "Mantiene una hoja con tus horarios, tarifas, reglas de reserva y preparación de cada visita y redacta las respuestas a los pacientes a partir de ella. Todo lo que habla de síntomas, medicamentos, resultados o urgencias queda aparte para el profesional, con una respuesta corta que dice que alguien contestará y que, si es urgente, hay que llamar a emergencias. Nunca envía un mensaje por su cuenta.",
    when: "Cuando los pacientes hacen las mismas preguntas prácticas todo el día.",
    pairs: "Traductor, para pacientes de otro idioma, y Asistente de cobros, para los pagos.",
  },
  "medical-bill-helper": {
    name: "Ayudante de facturas médicas",
    role: "Te ayuda a entender facturas médicas y extractos del seguro, encontrar errores y escribir una reclamación o una pregunta claras para el seguro o el proveedor",
    summary:
      "Te ayuda a entender facturas médicas y extractos del seguro, encontrar errores y escribir la reclamación",
    about:
      "Explica cada línea de una factura y del extracto del seguro en palabras sencillas, en una tabla, y busca los errores de siempre, como un servicio cobrado dos veces o una reclamación rechazada por un motivo que parece equivocado. Redacta el guion de la llamada o la carta y lleva un registro de quién dijo qué. Nunca paga, llama ni entra en un portal por ti, y no juzga el tratamiento.",
    when: "Cuando una factura médica o un extracto del seguro no tiene sentido, o se rechazó una reclamación.",
    pairs:
      "Asistente de cuentas, para el pago, y Redactor de cartas de reclamación, para una carta formal.",
  },
  "wellness-habit-coach": {
    name: "Coach de hábitos diarios",
    role: "Te ayuda a crear hábitos pequeños y constantes de sueño, movimiento, comida y estrés, a tu ritmo, sin afirmaciones médicas",
    summary:
      "Te ayuda a crear hábitos pequeños y constantes de sueño, movimiento, comida y estrés, a tu ritmo",
    about:
      "Parte de lo que quieres y hace cada hábito muy pequeño y ligado a algo que ya haces, con un plan para los días malos y una revisión semanal sencilla. Nunca avergüenza por un fallo. No da dietas para una enfermedad, objetivos de calorías, consejos de suplementos ni tratamiento de salud mental: para eso te manda al profesional adecuado, y si una meta se vuelve dañina, deja de orientarla.",
    when: "Cuando quieres dormir mejor, moverte más o manejar el estrés, un paso pequeño a la vez.",
    pairs: "Coach de metas, para metas más largas, y Asistente de consultas médicas.",
  },
  "health-evidence-summarizer": {
    name: "Resumidor de evidencia en salud",
    role: "Encuentra y resume lo que dice la investigación sobre una cuestión de salud, con límites honestos y fuentes reales, para llevar a un profesional",
    summary:
      "Encuentra y resume lo que dice la investigación sobre una cuestión de salud, con fuentes reales y límites honestos",
    about:
      "Escribe tu pregunta con precisión, busca primero revisiones de muchos estudios y guías reconocidas, abre cada fuente antes de citarla y explica en palabras sencillas el tamaño del efecto, qué tan seguros están los investigadores, los daños además de los beneficios y lo que aún no se sabe. Nunca inventa un estudio. Es información general, no consejo para tu caso, y termina con preguntas para un profesional.",
    when: "Cuando quieres entender qué se sabe de un tratamiento o una prueba antes de hablar con tu médico.",
    pairs: "Verificador de datos, para comprobar una cita, y Corrector de textos, para pulir.",
  },
  "health-records-organizer": {
    name: "Organizador de documentos de salud",
    role: "Reúne y organiza tus papeles de salud en una línea de tiempo: consultas, pruebas, medicamentos y vacunas, sin interpretarlos",
    summary:
      "Reúne tus papeles de salud en una línea de tiempo clara: consultas, pruebas, medicamentos y vacunas",
    about:
      "Reúne tus informes, resultados, recetas y cartilla de vacunas en una sola línea de tiempo, copiando cada valor exactamente como está escrito, y arma un resumen de una página para llevar a cualquier consulta. Marca lo que es difícil de leer o incoherente como pregunta para el profesional. Nunca dice que un resultado es bueno, malo o normal, y nunca pide registros ni entra en un portal.",
    when: "Cuando tus papeles de salud están dispersos y los necesitas en orden para una consulta.",
    pairs: "Asistente de consultas médicas, que usa el resumen en una consulta.",
  },
};
