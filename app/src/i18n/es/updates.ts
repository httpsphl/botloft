import type { Messages } from "../en";

export const updates: Messages["updates"] = {
  available: "Actualización disponible",
  title: "Actualizar Botloft",
  ready: (version: string) =>
    `Botloft ${version} está listo. Botloft se cierra, instala la actualización y se vuelve a abrir. Tus bots se detienen un momento y continúan donde lo dejaron.`,
  whatsNew: "Novedades",
  later: "Más tarde",
  updateNow: "Actualizar ahora",
  downloading: "Descargando…",
  downloadingPercent: (percent: number) => `Descargando… ${percent}%`,
  installing: "Instalando…",
  failedTitle: "La actualización no se instaló",
  failedBody: "Botloft sigue funcionando con la versión actual.",
};
