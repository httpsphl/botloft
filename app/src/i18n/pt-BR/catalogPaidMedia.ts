import type { paidMedia as en } from "../en/catalogPaidMedia";

export const paidMedia: typeof en = {
  "ppc-strategist": {
    name: "Estrategista de anúncios de busca",
    role: "Planeja campanhas de anúncios na busca: as palavras, a estrutura, os anúncios e um orçamento pequeno para testar, num plano que você executa",
    summary:
      "Planeja seus anúncios de busca, das palavras aos anúncios, com um orçamento pequeno para testar primeiro",
    about:
      "Parte do quanto um cliente vale para você, monta os grupos de palavras pelo que as pessoas querem, escreve os anúncios com a sua oferta real e define um teste pequeno com regras claras de quando parar e quando aumentar. Nunca cria nem paga um anúncio: quem faz é você, com o plano dele.",
    when: "Quando você quer começar anúncios na busca, ou os que você tem não estão dando retorno.",
    pairs:
      "Redator publicitário, para afiar os anúncios, e Analista de buscas, para ler os termos pesquisados.",
  },
  "paid-social-strategist": {
    name: "Estrategista de anúncios em redes sociais",
    role: "Planeja campanhas pagas nas redes sociais: o público, os formatos, o que criar e um orçamento de teste, sem segmentação sensível",
    summary:
      "Planeja suas campanhas pagas nas redes: quem alcançar, o que mostrar e como testar com pouco dinheiro",
    about:
      "Escolhe um só objetivo, define quem alcançar por interesses, lugar e as listas de clientes que você decidir usar, e planeja os formatos e as variações a testar. Nunca segmenta por traços sensíveis como saúde ou religião e nunca explora inseguranças. A campanha, você mesmo cria e paga.",
    when: "Quando você quer alcançar gente nova nas redes sem desperdiçar o orçamento.",
    pairs:
      "Estrategista de criativos de anúncios, para o que dizer e mostrar, e Roteirista de vídeo, para vídeos curtos.",
  },
  "ad-creative-strategist": {
    name: "Estrategista de criativos de anúncios",
    role: "Planeja o que os anúncios devem dizer e mostrar: ângulos, ganchos e variações para testar, e aprende com o que funciona",
    summary:
      "Planeja o que seus anúncios devem dizer e mostrar, com alguns ângulos para testar e o que cada um ensina",
    about:
      "Pega as palavras que seus clientes usam e escreve três ou quatro ângulos diferentes, cada um com o gancho, o texto e um briefing para a imagem ou o vídeo. Muda uma coisa por teste, para o resultado dizer por que ganhou. Só usa afirmações que você pode provar: nada de avaliação inventada nem falsa escassez.",
    when: "Quando seus anúncios parecem todos iguais, ou você não sabe o que testar a seguir.",
    pairs:
      "Designer, para as imagens, Redator publicitário, para polir, e Estrategista de anúncios em redes sociais.",
  },
  "ad-auditor": {
    name: "Auditor de contas de anúncios",
    role: "Revisa como suas campanhas de anúncios estão montadas e rodando, acha dinheiro desperdiçado e chances perdidas e lista os ajustes em ordem",
    summary:
      "Revisa suas campanhas de anúncios, acha dinheiro desperdiçado e chances perdidas e lista os ajustes em ordem",
    about:
      "Lê os relatórios que você der e acha os vazamentos de sempre: gasto sem resultado, gente alcançada duas vezes, anúncios cansados, buscas que não combinam e resultados contados errado. Cada achado vem com os números, o dinheiro em jogo e o ajuste, em ordem. Não tem acesso à sua conta e não muda nada nela.",
    when: "Quando o gasto com anúncios cresce, mas você não tem certeza de que está funcionando.",
    pairs:
      "Especialista em medição, para conferir as contagens, e Estrategista de anúncios de busca, para aplicar os ajustes.",
  },
  "tracking-specialist": {
    name: "Especialista em medição",
    role: "Confere se seus anúncios e seu site medem os resultados direito: eventos, tags, links e o que conta como resultado",
    summary:
      "Confere se seus anúncios e seu site contam os resultados direito, para você confiar nos números",
    about:
      "Lista cada lugar em que um resultado é contado e compara: plataforma de anúncios, site, loja e planilha. Explica cada diferença em palavras simples, como contagem em dobro ou links que perdem a marcação, e propõe um nome limpo para eventos e links. Diz até onde os números merecem confiança e não coleta dos visitantes mais do que você precisa.",
    when: "Antes de gastar com anúncios, ou quando os números de lugares diferentes não batem.",
    pairs: "Desenvolvedor front-end, que coloca as tags, e Auditor de contas de anúncios.",
  },
  "search-query-analyst": {
    name: "Analista de buscas",
    role: "Lê as buscas reais por trás dos seus anúncios ou do seu site, agrupa por intenção e acha o que incluir, excluir ou escrever",
    summary:
      "Lê as buscas reais por trás dos seus anúncios e do seu site e diz o que incluir, excluir ou escrever",
    about:
      "Pega as palavras que as pessoas realmente digitaram e agrupa pelo que querem: comprar, comparar, aprender. Entrega três listas: o que excluir porque gasta dinheiro à toa, o que incluir porque funciona e sobre o que escrever porque as pessoas perguntam. Diz o quanto tem certeza, já que um termo com três cliques não prova nada.",
    when: "Quando você faz anúncios de busca ou quer saber o que as pessoas procuram antes de achar você.",
    pairs:
      "Estrategista de anúncios de busca, que usa as listas, e Especialista em SEO, para as ideias de conteúdo.",
  },
};
