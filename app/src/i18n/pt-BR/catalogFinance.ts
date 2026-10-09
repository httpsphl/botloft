import type { finance as en } from "../en/catalogFinance";

export const finance: typeof en = {
  bookkeeper: {
    name: "Auxiliar contábil",
    role: "Mantém seus registros em ordem: separa as transações em categorias, casa com os comprovantes e acha o que não fecha",
    summary:
      "Separa suas transações em categorias, casa com os comprovantes e avisa o que não fecha",
    about:
      "Pega seus extratos, comprovantes e planilhas e mantém um registro limpo: cada transação numa categoria, ligada à sua prova, com os totais conferidos. Lista o que está sem comprovante em vez de chutar e nunca altera um arquivo original. Cuida dos registros; não é contador e nunca mexe em dinheiro.",
    when: "No fim de todo mês, ou quando seus registros se acumularam.",
    pairs: "Analista financeiro e Planejador de orçamento, que trabalham com registros limpos.",
  },
  "budget-planner": {
    name: "Planejador de orçamento",
    role: "Monta um orçamento realista com sua renda e seus gastos, acompanha ao longo do mês e mostra para onde o dinheiro vai",
    summary:
      "Monta um orçamento realista com sua renda e seus gastos e mostra para onde o dinheiro vai",
    about:
      "Parte dos seus números reais e monta um plano simples: o que entra, o que sai e o que você quer alcançar. Se o plano não fecha, diz isso primeiro e mostra algumas formas de fechar. Durante o mês mostra onde você está acima do plano. Não dá conselho sobre investimentos nem empréstimos.",
    when: "Quando você quer saber para onde o dinheiro vai, ou planejar uma meta.",
    pairs:
      "Auxiliar contábil, para registros limpos, e Previsor de fluxo de caixa, para o que vem.",
  },
  "financial-analyst": {
    name: "Analista financeiro",
    role: "Lê demonstrações e relatórios financeiros, calcula as medidas principais e explica o que dizem sobre o negócio",
    summary:
      "Lê suas demonstrações financeiras, calcula as medidas principais e explica o que significam",
    about:
      "Lê suas demonstrações e planilhas, confere se os números fecham e calcula o que importa para a sua pergunta: margens, crescimento, ponto de equilíbrio, retorno de um projeto. Mostra cada fórmula, compara com algo que faz sentido e diz o que os números não contam. Não diz o que comprar, vender ou investir.",
    when: "Antes de uma decisão que depende dos números, ou para entender como o negócio vai.",
    pairs: "Auxiliar contábil, para os registros, e Analista de dados, para um recorte mais fundo.",
  },
  "cash-flow-forecaster": {
    name: "Previsor de fluxo de caixa",
    role: "Projeta o dinheiro que entra e sai semana a semana e avisa cedo quando o caixa pode apertar",
    summary:
      "Projeta o dinheiro que entra e sai semana a semana e avisa cedo se o caixa pode apertar",
    about:
      "Monta uma previsão a partir do saldo de hoje, das contas a pagar e do que você espera receber, e mostra o ponto mais baixo e quando ele chega. Dá um caso esperado, um cauteloso e um bom e marca cada estimativa. Se o caixa puder faltar, lista suas opções e o que cada uma custa. Nunca mexe em dinheiro nem adia um pagamento sozinho.",
    when: "Quando o dinheiro está curto, uma conta grande se aproxima ou você vai assumir um custo.",
    pairs:
      "Assistente de contas, para o que vence, e Assistente de cobrança, para o que devem a você.",
  },
  "bills-assistant": {
    name: "Assistente de contas",
    role: "Acompanha o que você deve: lista as contas e os vencimentos, confere com pedidos e contratos e prepara cada pagamento para você fazer",
    summary:
      "Acompanha suas contas e vencimentos, confere se há erros e prepara cada pagamento para você fazer",
    about:
      "Reúne suas contas, lista o valor e o vencimento de cada uma e confere com o pedido ou o contrato: duplicadas, total errado e cobranças que ninguém combinou. Mantém uma lista por vencimento e avisa quando uma conta pede pagamento urgente para uma conta nova, um golpe comum. Quem faz cada pagamento é você.",
    when: "Quando as contas chegam de muitos lugares e você não quer pagar atrasado, nem duas vezes.",
    pairs: "Auxiliar contábil, para as pagas, e Previsor de fluxo de caixa, para o que vem.",
  },
  "invoicing-assistant": {
    name: "Assistente de cobrança",
    role: "Prepara suas notas e lembretes de pagamento e acompanha o que os clientes ainda devem a você",
    summary:
      "Prepara suas notas e lembretes de pagamento educados e acompanha o que os clientes ainda devem",
    about:
      "Prepara a cobrança do trabalho que você fez, numera tudo em ordem e mantém a lista do que foi enviado, pago e atrasado. Para uma cobrança atrasada, escreve um lembrete em três passos, do simpático ao firme, sem ameaçar. Marca os campos legais que não consegue preencher, para você ou seu contador, e você mesmo envia tudo.",
    when: "Quando você cobra clientes e correr atrás de pagamento toma tempo demais.",
    pairs: "Auxiliar contábil, para as pagas, e Previsor de fluxo de caixa, para os vencimentos.",
  },
  "tax-organizer": {
    name: "Organizador de impostos",
    role: "Reúne e organiza os documentos e números dos seus impostos durante o ano, para o contador ou a declaração começarem prontos",
    summary:
      "Reúne e organiza seus papéis e números de impostos durante o ano para a declaração começar pronta",
    about:
      "Reúne e separa os papéis de que você precisa para os impostos, mantém uma lista do que está completo e do que falta e avisa antes das datas que importam. Não decide o que é dedutível: lista os itens duvidosos como perguntas para o seu contador. Nunca declara, paga nem entra em site de imposto, e não é contador.",
    when: "Durante o ano e nas semanas antes de você ou o contador declararem.",
    pairs:
      "Auxiliar contábil, para os registros, e Assistente de cobrança, para o lado da receita.",
  },
};
