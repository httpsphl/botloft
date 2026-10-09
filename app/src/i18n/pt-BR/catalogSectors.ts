import type { sectors as en } from "../en/catalogSectors";

export const sectors: typeof en = {
  "real-estate-assistant": {
    name: "Assistente imobiliário",
    role: "Ajuda você a comprar, vender ou alugar um imóvel: compara anúncios, prepara perguntas e roteiros e rascunha anúncios e mensagens",
    summary:
      "Ajuda você a comprar, vender ou alugar um imóvel: compara anúncios, prepara perguntas e rascunha o anúncio",
    about:
      "Compara os imóveis numa só tabela, com os mesmos dados de cada um, inclusive os custos reais, e diz o que o anúncio deixa de fora. Prepara roteiros e perguntas para as visitas e ajuda a escrever um anúncio honesto e perguntas para compradores ou inquilinos. Nunca afirma um valor de mercado como fato, faz proposta nem assina, e não é corretor nem advogado.",
    when: "Quando você procura um imóvel, ou prepara o seu para vender ou alugar.",
    pairs: "Pesquisador, para fatos sobre uma região, e Redator, para polir o anúncio.",
  },
  "hospitality-guest-assistant": {
    name: "Assistente de hospedagem",
    role: "Ajuda um hotel, pousada ou anfitrião de aluguel a cuidar dos hóspedes: respostas, mensagens de boas-vindas, guias e o que melhorar",
    summary:
      "Ajuda um hotel ou anfitrião a cuidar dos hóspedes: respostas, boas-vindas, guias e melhorias",
    about:
      "Mantém uma folha curta de fatos sobre o seu lugar e rascunha as respostas aos hóspedes a partir dela, sem inventar horário, preço ou promessa. Escreve a mensagem de boas-vindas e o guia do hóspede, rascunha respostas calmas a reclamações e lê as suas avaliações para ver o que os hóspedes gostam e o que se repete como problema. Reembolsos e descontos continuam sendo decisão sua.",
    when: "Quando os hóspedes perguntam as mesmas coisas o dia todo, ou você quer avaliações melhores.",
    pairs: "Tradutor, para hóspedes que falam outro idioma, e Social media.",
  },
  "returns-assistant": {
    name: "Assistente de trocas e devoluções",
    role: "Cuida dos pedidos de troca, devolução e reembolso pela sua política: separa, rascunha as respostas e enxerga padrões e abusos",
    summary:
      "Cuida dos pedidos de troca e reembolso pela sua política: separa, rascunha respostas e enxerga padrões",
    about:
      "Lê cada pedido contra a sua política por escrito e o classifica como dentro, fora ou dúvida, e então rascunha uma resposta clara e simpática. As dúvidas e exceções vêm para você. Toda semana mostra o que se repete, como produtos com muitas devoluções, sem acusar nenhum cliente. Avisa quando um cliente pode ter direito legal a reembolso. Nunca faz um reembolso sozinho.",
    when: "Quando pedidos de troca e reembolso tomam tempo demais do seu dia.",
    pairs:
      "Atendimento ao cliente, para o tom, e Gestor de e-commerce, para os ajustes nos anúncios.",
  },
  "supply-chain-planner": {
    name: "Planejador de compras e estoque",
    role: "Planeja o que comprar e quando: níveis de estoque, pontos de reposição, comparação de fornecedores e riscos de entrega, para um pequeno negócio",
    summary:
      "Planeja o que comprar e quando: estoque, pontos de reposição e comparação de fornecedores para um pequeno negócio",
    about:
      "Trabalha com as suas vendas, estoque, preços e prazos para propor um ponto de reposição e um tamanho de pedido para cada item, mostrando a fórmula. Marca os itens que não podem acabar e os parados que prendem dinheiro, compara fornecedores pelo custo total e lista os riscos com um plano B. Nunca faz um pedido nem fala com um fornecedor.",
    when: "Quando falta produto no estoque, ou você guarda demais do que não vende.",
    pairs:
      "Auxiliar contábil, para os custos, e Previsor de fluxo de caixa, para o que você pode pagar.",
  },
  "grant-writer": {
    name: "Redator de editais e financiamentos",
    role: "Encontra e prepara candidaturas a editais e chamadas públicas: lê as regras, planeja a resposta e a redige com os seus fatos reais",
    summary:
      "Prepara candidaturas a editais e chamadas públicas: lê as regras, planeja a resposta e a redige",
    about:
      "Lê o edital inteiro, resume com o prazo em primeiro lugar e confere com honestidade se você se enquadra antes de escrever qualquer coisa. Depois planeja a candidatura contra a pontuação e redige cada parte com os seus fatos reais, marcando cada lacuna como pergunta. Nunca inventa um resultado, um parceiro ou um número, e nunca envia por você.",
    when: "Quando você acha um edital, uma chamada pública ou um programa de financiamento que pode servir.",
    pairs: "Analista financeiro, para o orçamento, e Revisor de texto, para polir.",
  },
  "restaurant-manager": {
    name: "Assistente de restaurante",
    role: "Ajuda a tocar um restaurante ou café pequeno: cardápio e custos, escalas, pedidos e compras, e respostas aos clientes",
    summary:
      "Ajuda a tocar um restaurante ou café pequeno: custo do cardápio, escalas, compras e respostas aos clientes",
    about:
      "Calcula quanto cada prato realmente custa e onde você perde dinheiro, monta as escalas pelos seus horários de pico, mantém a lista de compras e uma folha diária que mostra o que se repete. Nunca diz que um prato não tem um alérgeno ou serve a uma dieta por palpite: isso vem das suas fichas de receita, e quando elas não dizem, manda você conferir antes.",
    when: "Quando você toca um pequeno negócio de comida e a papelada come o seu tempo.",
    pairs: "Auxiliar contábil, para os custos, e Social media, para posts sobre o cardápio.",
  },
  "course-creator": {
    name: "Criador de cursos online",
    role: "Ajuda você a transformar o que sabe num curso ou oficina online: a promessa, o roteiro, as aulas, os exercícios e a página que o apresenta",
    summary:
      "Ajuda você a transformar o que sabe num curso online: roteiro, aulas, exercícios e a página de venda",
    about:
      "Parte do aluno e de uma promessa honesta que você consegue cumprir, monta o roteiro de trás para a frente a partir do resultado e redige as aulas e os exercícios na sua voz, com os seus exemplos. A página do curso usa só conteúdo e resultados reais: nada de depoimentos inventados, prazos falsos ou promessas de ganho. Quem publica e define o preço é você.",
    when: "Quando você sabe algo que as pessoas pagariam para aprender e quer ensinar.",
    pairs:
      "Designer de apresentações, para os slides, Roteirista de vídeo, para os roteiros, e Revisor de texto, para revisar.",
  },
};
