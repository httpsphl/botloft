import type { Messages } from "../en";
import { base } from "./catalogBase";
import { marketing } from "./catalogMarketing";
import { product } from "./catalogProduct";

export const catalog: Messages["catalog"] = {
  title: "Agência de bots",
  open: "Agência de bots",
  openHint: "Bots prontos que você pode adicionar a esta equipe",
  intro:
    "Escolha um bot para o trabalho. Ele entra nesta equipe pronto para trabalhar, e depois você pode mudar o que quiser nele.",
  invite: {
    title: "Quem você quer na sua equipe?",
    body: "Não sabe o que criar? Escolha um bot para o trabalho. Depois você pode mudar o que quiser nele.",
  },
  search: "Buscar bots",
  filter: "Tipos de bot",
  categories: {
    all: "Todos",
    code: "Código",
    design: "Design",
    content: "Conteúdo",
    research: "Pesquisa",
    business: "Negócios",
    product: "Produto e gestão",
    marketing: "Marketing e vendas",
  },
  none: "Nenhum bot combina com isso.",
  loading: "Carregando os bots…",
  add: "Adicionar na equipe",
  learnMore: "Saber mais",
  back: "Voltar para todos os bots",
  seeAll: "Ver todos os bots",
  joined: (name) => `${name} entrou na equipe`,
  customize: "Personalizar",
  failed: {
    load: "Não deu para carregar os bots",
    add: "Não deu para adicionar o bot",
  },
  detail: {
    what: "O que faz",
    when: "Quando chamar",
    pairs: "Combina bem com",
    technical: "O que o bot recebe quando começa",
  },
  roles: { ...base, ...product, ...marketing },
};
