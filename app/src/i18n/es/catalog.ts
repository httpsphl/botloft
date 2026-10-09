import type { Messages } from "../en";
import { base } from "./catalogBase";
import { contentAndLearning } from "./catalogContent";
import { design } from "./catalogDesign";
import { engineering } from "./catalogEngineering";
import { finance } from "./catalogFinance";
import { marketing } from "./catalogMarketing";
import { paidMedia } from "./catalogPaidMedia";
import { people } from "./catalogPeople";
import { product } from "./catalogProduct";

export const catalog: Messages["catalog"] = {
  title: "Agencia de bots",
  open: "Agencia de bots",
  openHint: "Bots listos que puedes añadir a este equipo",
  intro:
    "Elige un bot para el trabajo. Entra en este equipo listo para trabajar, y después puedes cambiar lo que quieras de él.",
  invite: {
    title: "¿A quién quieres en tu equipo?",
    body: "¿No sabes qué crear? Elige un bot para el trabajo. Después puedes cambiar lo que quieras de él.",
  },
  search: "Buscar bots",
  filter: "Tipos de bot",
  categories: {
    all: "Todos",
    code: "Código",
    design: "Diseño",
    content: "Contenido",
    research: "Investigación",
    business: "Negocios",
    product: "Producto y gestión",
    marketing: "Marketing y ventas",
    learning: "Aprendizaje",
    finance: "Finanzas",
    people: "Personas y RR. HH.",
  },
  none: "Ningún bot coincide con eso.",
  loading: "Cargando los bots…",
  add: "Añadir al equipo",
  learnMore: "Saber más",
  back: "Volver a todos los bots",
  seeAll: "Ver todos los bots",
  joined: (name) => `${name} entró en el equipo`,
  customize: "Personalizar",
  failed: {
    load: "No se pudieron cargar los bots",
    add: "No se pudo añadir el bot",
  },
  detail: {
    what: "Qué hace",
    when: "Cuándo llamarlo",
    pairs: "Combina bien con",
    technical: "Lo que recibe el bot cuando empieza",
  },
  roles: {
    ...base,
    ...product,
    ...marketing,
    ...engineering,
    ...contentAndLearning,
    ...design,
    ...finance,
    ...paidMedia,
    ...people,
  },
};
