// Ajustes > Instrucciones para todos (spec 7.10).

import type { instructions as en } from "../en/instructions";

export const instructions: typeof en = {
  page: "Instrucciones para todos",
  label: "Lo que sigue cada agente",
  intro: "Cada agente lee esto en cada conversación, con cualquier motor. Cuanto más corto, mejor.",
  placeholder:
    "Por ejemplo: Respóndeme en español. Pregúntame antes de borrar archivos. Guarda los informes en la carpeta Informes.",
  howItWorks: "Cómo funciona",
  explain:
    "Cuando esto no coincide con otra cosa, vale lo más específico: lo que pides en la conversación, luego las instrucciones de cada agente y después estas. Van antes de lo que el agente anotó para sí y de lo que otro agente le pide, y nunca se saltan una aprobación. Los agentes las leen al iniciar: un agente disponible empieza de nuevo ahora, en la misma conversación, y uno que está trabajando, cuando termine.",
  save: "Guardar",
  saved: "Guardado. Los agentes disponibles empiezan de nuevo para leerlo.",
  saveFailed: "No se pudieron guardar las instrucciones",
  loadFailed: "No se pudieron leer las instrucciones",
};
