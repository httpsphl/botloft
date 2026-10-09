import type { finance as en } from "../en/catalogFinance";

export const finance: typeof en = {
  bookkeeper: {
    name: "Auxiliar contable",
    role: "Mantiene tus registros en orden: clasifica las transacciones, las empareja con los comprobantes y encuentra lo que no cuadra",
    summary:
      "Clasifica tus transacciones, las empareja con los comprobantes y avisa de lo que no cuadra",
    about:
      "Toma tus extractos, comprobantes y hojas de cálculo y mantiene un registro limpio: cada transacción en una categoría, ligada a su prueba, con los totales comprobados. Lista lo que no tiene comprobante en vez de adivinar y nunca cambia un archivo original. Lleva los registros; no es contador y nunca mueve dinero.",
    when: "A fin de cada mes, o cuando tus registros se han acumulado.",
    pairs: "Analista financiero y Planificador de presupuesto, que trabajan con registros limpios.",
  },
  "budget-planner": {
    name: "Planificador de presupuesto",
    role: "Arma un presupuesto realista con tus ingresos y gastos, lo sigue durante el mes y muestra a dónde va el dinero",
    summary:
      "Arma un presupuesto realista con tus ingresos y gastos y muestra a dónde va el dinero",
    about:
      "Parte de tus números reales y arma un plan sencillo: lo que entra, lo que sale y lo que quieres lograr. Si el plan no cierra, lo dice primero y muestra algunas formas de cerrarlo. Durante el mes muestra dónde vas por encima del plan. No aconseja sobre inversiones ni préstamos.",
    when: "Cuando quieres saber a dónde va el dinero, o planificar una meta.",
    pairs:
      "Auxiliar contable, para registros limpios, y Previsor de flujo de caja, para lo que viene.",
  },
  "financial-analyst": {
    name: "Analista financiero",
    role: "Lee estados e informes financieros, calcula las medidas clave y explica lo que dicen del negocio",
    summary: "Lee tus estados financieros, calcula las medidas clave y explica lo que significan",
    about:
      "Lee tus estados y hojas de cálculo, comprueba que los números cuadren y calcula lo que importa para tu pregunta: márgenes, crecimiento, punto de equilibrio, retorno de un proyecto. Muestra cada fórmula, compara con algo que tenga sentido y dice lo que los números no cuentan. No dice qué comprar, vender o en qué invertir.",
    when: "Antes de una decisión que depende de los números, o para entender cómo va el negocio.",
    pairs:
      "Auxiliar contable, para los registros, y Analista de datos, para un corte más profundo.",
  },
  "cash-flow-forecaster": {
    name: "Previsor de flujo de caja",
    role: "Proyecta el dinero que entra y sale semana a semana y avisa pronto cuando la caja puede quedar corta",
    summary:
      "Proyecta el dinero que entra y sale semana a semana y avisa pronto si la caja puede quedar corta",
    about:
      "Arma una previsión con el saldo de hoy, las cuentas por pagar y lo que esperas cobrar, y muestra el punto más bajo y cuándo llega. Da un caso esperado, uno cauteloso y uno bueno y marca cada estimación. Si la caja puede faltar, lista tus opciones y lo que cuesta cada una. Nunca mueve dinero ni aplaza un pago por su cuenta.",
    when: "Cuando el dinero está justo, se acerca una cuenta grande o vas a asumir un costo.",
    pairs: "Asistente de cuentas, para lo que vence, y Asistente de cobros, para lo que te deben.",
  },
  "bills-assistant": {
    name: "Asistente de cuentas",
    role: "Sigue lo que debes: lista las cuentas y vencimientos, las compara con pedidos y contratos y prepara cada pago para que lo hagas tú",
    summary:
      "Sigue tus cuentas y vencimientos, comprueba si hay errores y prepara cada pago para que lo hagas tú",
    about:
      "Reúne tus cuentas, lista el importe y el vencimiento de cada una y las compara con el pedido o el contrato: duplicadas, total equivocado y cargos que nadie acordó. Mantiene una lista por vencimiento y avisa cuando una cuenta pide pago urgente a una cuenta nueva, un fraude común. Cada pago lo haces tú.",
    when: "Cuando las cuentas llegan de muchos sitios y no quieres pagar tarde ni dos veces.",
    pairs: "Auxiliar contable, para las pagadas, y Previsor de flujo de caja, para lo que viene.",
  },
  "invoicing-assistant": {
    name: "Asistente de cobros",
    role: "Prepara tus facturas y recordatorios de pago y sigue lo que los clientes todavía te deben",
    summary:
      "Prepara tus facturas y recordatorios de pago amables y sigue lo que los clientes todavía te deben",
    about:
      "Prepara las facturas del trabajo que hiciste, las numera en orden y mantiene la lista de lo enviado, pagado y atrasado. Para una factura atrasada escribe un recordatorio en tres pasos, de amable a firme, sin amenazar. Marca los campos legales que no puede rellenar, para ti o tu contador, y tú mismo envías todo.",
    when: "Cuando facturas a clientes y perseguir pagos te quita demasiado tiempo.",
    pairs:
      "Auxiliar contable, para las pagadas, y Previsor de flujo de caja, para los vencimientos.",
  },
  "tax-organizer": {
    name: "Organizador de impuestos",
    role: "Reúne y organiza los documentos y números de tus impuestos durante el año, para que tu contador o tu declaración empiecen listos",
    summary:
      "Reúne y organiza tus papeles y números de impuestos durante el año para que la declaración empiece lista",
    about:
      "Reúne y clasifica los papeles que necesitas para los impuestos, mantiene una lista de lo completo y lo que falta y avisa antes de las fechas que importan. No decide qué es deducible: lista los casos dudosos como preguntas para tu contador. Nunca declara, paga ni entra en una web de impuestos, y no es contador.",
    when: "Durante el año y en las semanas antes de que tú o tu contador declaren.",
    pairs:
      "Auxiliar contable, para los registros, y Asistente de cobros, para el lado de los ingresos.",
  },
};
