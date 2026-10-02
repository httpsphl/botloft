// Los archivos que hizo un bot (spec 15.1, 8.5): el panel junto al chat.

import type { Messages } from "../en";

export const files: Messages["files"] = {
  heading: "Archivos",
  panel: (bot) => `Archivos de ${bot}`,
  show: "Mostrar archivos",
  showInPanel: "Ver en archivos",
  hide: "Ocultar archivos",
  close: "Cerrar",
  refresh: "Actualizar",
  fresh: (count) => (count === 1 ? "1 archivo nuevo" : `${count} archivos nuevos`),
  newTag: "Nuevo",
  list: "Archivos",
  emptyTitle: (bot) => `${bot} todavía no hizo ningún archivo`,
  emptyBody: "Lo que cree para ti, como un informe o una hoja de cálculo, aparece aquí.",
  loadFailed: "No se pudieron cargar los archivos",
  madeByBot: (bot) => `Hecho por ${bot}`,
  back: "Todos los archivos",
  open: "Abrir",
  reveal: "Mostrar en la carpeta",
  shared: (bot) => `Archivos compartidos por ${bot}`,
  save: "Guardar como…",
  saveOne: (name) => `Guardar una copia de ${name}`,
  previewOne: "Ver",
  failed: {
    open: "No se pudo abrir el archivo",
    reveal: "No se pudo mostrar el archivo",
    save: "No se pudo guardar el archivo",
  },
  preview: {
    loading: "Cargando…",
    failed: "No se pudo mostrar este archivo",
    none: "Este tipo de archivo no tiene vista previa. Ábrelo para verlo.",
    tooBig: "Este archivo es demasiado grande para mostrarlo aquí. Ábrelo para verlo.",
    cut: "Solo se muestra el comienzo del archivo.",
    gone: "Este archivo ya no está allí.",
  },
};
