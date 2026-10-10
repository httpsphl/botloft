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
  title: "Agência de agentes",
  open: "Agência de agentes",
  openHint: "Agentes prontos que você pode adicionar a esta equipe",
  intro:
    "Escolha um agente para o trabalho. Ele entra nesta equipe pronto para trabalhar, e depois você pode mudar o que quiser nele.",
  invite: {
    title: "Quem você quer na sua equipe?",
    body: "Não sabe o que criar? Escolha um agente para o trabalho. Depois você pode mudar o que quiser nele.",
  },
  search: "Buscar agentes",
  filter: "Tipos de agente",
  categories: {
    all: "Todos",
    code: "Código",
    design: "Design",
    content: "Conteúdo",
    research: "Pesquisa",
    business: "Negócios",
    product: "Produto e gestão",
    marketing: "Marketing e vendas",
    learning: "Aprendizado",
    finance: "Finanças",
    people: "Pessoas e RH",
    security: "Segurança",
    health: "Saúde",
    games: "Jogos",
    legal: "Jurídico",
  },
  none: "Nenhum agente combina com isso.",
  loading: "Carregando os agentes…",
  add: "Adicionar na equipe",
  learnMore: "Saber mais",
  back: "Voltar para todos os agentes",
  seeAll: "Ver todos os agentes",
  joined: (name) => `${name} entrou na equipe`,
  customize: "Personalizar",
  failed: {
    load: "Não deu para carregar os agentes",
    add: "Não deu para adicionar o agente",
  },
  detail: {
    what: "O que faz",
    when: "Quando chamar",
    pairs: "Combina bem com",
    technical: "O que o agente recebe quando começa",
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
