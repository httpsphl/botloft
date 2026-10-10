import type { Messages } from "../en";
import { base } from "./catalogBase";
import { contentAndLearning } from "./catalogContent";
import { design } from "./catalogDesign";
import { engineering } from "./catalogEngineering";
import { engineeringMore } from "./catalogEngineeringMore";
import { finance } from "./catalogFinance";
import { games } from "./catalogGames";
import { health } from "./catalogHealth";
import { legal } from "./catalogLegal";
import { marketing } from "./catalogMarketing";
import { paidMedia } from "./catalogPaidMedia";
import { people } from "./catalogPeople";
import { product } from "./catalogProduct";
import { sectors } from "./catalogSectors";
import { security } from "./catalogSecurity";

export const catalog: Messages["catalog"] = {
  title: "Agencia de agentes",
  open: "Agencia de agentes",
  openHint: "Agentes listos que puedes añadir a este equipo",
  intro:
    "Elige un agente para el trabajo. Entra en este equipo listo para trabajar, y después puedes cambiar lo que quieras de él.",
  invite: {
    title: "¿A quién quieres en tu equipo?",
    body: "¿No sabes qué crear? Elige un agente para el trabajo. Después puedes cambiar lo que quieras de él.",
  },
  search: "Buscar agentes",
  filter: "Tipos de agente",
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
    security: "Seguridad",
    health: "Salud",
    games: "Juegos",
    legal: "Jurídico",
  },
  none: "Ningún agente coincide con eso.",
  loading: "Cargando los agentes…",
  add: "Añadir al equipo",
  learnMore: "Saber más",
  back: "Volver a todos los agentes",
  seeAll: "Ver todos los agentes",
  joined: (name) => `${name} entró en el equipo`,
  customize: "Personalizar",
  failed: {
    load: "No se pudieron cargar los agentes",
    add: "No se pudo añadir el agente",
  },
  detail: {
    what: "Qué hace",
    when: "Cuándo llamarlo",
    pairs: "Combina bien con",
    technical: "Lo que recibe el agente cuando empieza",
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
    ...engineeringMore,
    ...security,
    ...sectors,
    ...legal,
    ...games,
    ...health,
  },
};
