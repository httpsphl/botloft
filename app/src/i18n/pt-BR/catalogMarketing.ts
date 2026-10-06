import type { marketing as en } from "../en/catalogMarketing";

export const marketing: typeof en = {
  "seo-specialist": {
    name: "Especialista em SEO",
    role: "Ajuda um site a aparecer nas buscas: descobre o que as pessoas procuram, corrige páginas e planeja conteúdo",
    summary: "Descobre o que as pessoas procuram e melhora as suas páginas para serem encontradas",
    about:
      "Descobre as palavras que as pessoas realmente usam para procurar o que você vende, confere as suas páginas atrás do que as atrapalha e sugere mudanças em títulos, textos e estrutura. Só usa métodos honestos, e os resultados levam semanas.",
    when: "Quando o seu site está no ar, mas as pessoas não estão encontrando.",
    pairs: "Redator, para escrever as páginas, e Desenvolvedor, para corrigir problemas técnicos.",
  },
  "email-marketer": {
    name: "Especialista em e-mail marketing",
    role: "Planeja e escreve campanhas e sequências de e-mail, com objetivo claro e respeito ao consentimento",
    summary:
      "Escreve e-mails e sequências que são abertos e respondidos, com consentimento e saída fácil",
    about:
      "Planeja e escreve e-mails e sequências automáticas (boas-vindas, acompanhamento, reconquista, novidades) com opções de assunto e uma ação clara. Nunca envia nada e lembra você de que as leis sobre e-mail valem.",
    when: "Quando você tem uma lista de pessoas que aceitaram ouvir de você e quer usá-la bem.",
    pairs: "Designer, para o visual, e Analista de dados, para ler os resultados.",
  },
  "content-strategist": {
    name: "Estrategista de conteúdo",
    role: "Planeja o que publicar, para quem e por quê, para o conteúdo apoiar um objetivo do negócio",
    summary:
      "Planeja o que publicar, para quem e por quê, com um calendário ligado aos seus objetivos",
    about:
      "Escolhe poucos temas sobre os quais você fala com conhecimento de verdade e monta um calendário que você consegue manter: cada peça com seu objetivo, formato e a pergunta que responde. Depois escreve um resumo curto para cada uma.",
    when: "Quando você publica muito, mas não sabe se leva a algum lugar.",
    pairs: "Redator e Roteirista de vídeo, que fazem as peças, e Pesquisador, para os fatos.",
  },
  copywriter: {
    name: "Redator publicitário",
    role: "Escreve textos persuasivos e honestos para anúncios, páginas de venda e páginas de produto",
    summary: "Escreve títulos e textos que fazem as pessoas agirem, sem exagerar",
    about:
      "Escreve títulos, anúncios, páginas de venda e descrições de produto com várias opções, cada uma baseada no que importa para quem compra. Mantém afirmações que você consegue provar e nunca inventa depoimentos nem números.",
    when: "Quando uma página ou um anúncio não está fazendo as pessoas agirem.",
    pairs: "Especialista em SEO, para as palavras de busca, e Designer, para o visual.",
  },
  "ads-manager": {
    name: "Gestor de anúncios",
    role: "Planeja campanhas de anúncios pagos, escreve os anúncios e lê os resultados, mas nunca gasta dinheiro sozinho",
    summary:
      "Planeja os seus anúncios, escreve e lê os resultados, e não gasta um centavo sem você",
    about:
      "Planeja um começo pequeno para os seus anúncios pagos, escreve para cada plataforma e, quando você traz os números, diz o que parar, manter e testar. Ele nunca cria nem paga um anúncio: você faz isso, com o plano dele.",
    when: "Quando você quer experimentar anúncios pagos sem desperdiçar dinheiro.",
    pairs: "Redator publicitário, para os textos, e Analista de dados, para os números.",
  },
  "video-scriptwriter": {
    name: "Roteirista de vídeo",
    role: "Escreve roteiros e listas de cenas para vídeos curtos e longos",
    summary:
      "Escreve roteiros com um começo forte, estrutura clara e uma lista de cenas para gravar",
    about:
      "Escreve roteiros em duas colunas, o que é dito e o que é visto, com os primeiros segundos fortes e uma lista do que você precisa para gravar. Ele não faz o vídeo.",
    when: "Para um vídeo curto, uma demonstração de produto ou um tutorial.",
    pairs:
      "Estrategista de conteúdo, para o plano em volta do vídeo, e Redator publicitário, para a chamada final.",
  },
  "newsletter-editor": {
    name: "Editor de newsletter",
    role: "Planeja e escreve uma newsletter recorrente: temas, edições e uma voz consistente",
    summary:
      "Planeja e escreve a sua newsletter edição por edição, com uma voz que os leitores reconhecem",
    about:
      "Define o que a sua newsletter promete, mantém um plano das próximas edições e escreve cada uma com uma voz constante. Confere fatos, cita as fontes e nunca envia uma edição sozinho.",
    when: "Quando você quer uma newsletter, mas sem a página em branco toda semana.",
    pairs: "Pesquisador, para os fatos, e Designer, para as imagens.",
  },
  "community-manager": {
    name: "Gestor de comunidade",
    role: "Rascunha respostas, dá boas-vindas e avisa dos problemas na sua comunidade, seguindo as regras que você definir",
    summary: "Prepara respostas e moderação para a sua comunidade, seguindo as suas regras",
    about:
      "Rascunha respostas e boas-vindas na voz da sua comunidade e avisa o que precisa de você: reclamações, assédio, segurança ou questões legais. Nunca publica, apaga nem bane ninguém por conta própria.",
    when: "Quando o seu grupo ou página cresce mais rápido do que você consegue responder.",
    pairs:
      "Atendimento ao cliente, para problemas de clientes, e Estrategista de conteúdo, para ideias de conversa.",
  },
  "ecommerce-manager": {
    name: "Gestor de e-commerce",
    role: "Cuida de uma loja online: páginas de produto, catálogo, promoções e o que consertar primeiro",
    summary:
      "Melhora a sua loja online: páginas de produto, catálogo, promoções e o que consertar primeiro",
    about:
      "Percorre as suas páginas de produto como um comprador, reescreve títulos e descrições, olha o que vende e o que não vende e planeja promoções que a sua margem aguenta. Nunca mexe na loja em si.",
    when: "Quando você tem uma loja online e quer vender mais com ela.",
    pairs: "Redator publicitário, para os textos, e Especialista em SEO, para as buscas.",
  },
  "brand-strategist": {
    name: "Estrategista de marca",
    role: "Define o que uma marca representa, como ela fala e no que se diferencia das outras",
    summary: "Define quem é a sua marca, como ela fala e o que a torna diferente",
    about:
      "Pergunta sobre a sua história, olha como negócios parecidos se apresentam e escreve um guia curto: a quem você serve, a sua promessa, os seus valores, a sua voz com exemplos e o que faz você diferente.",
    when: "Quando você está começando, ou quando o que você diz soa igual ao de todo mundo.",
    pairs: "Designer, Redator publicitário e Estrategista de conteúdo, que seguem o guia.",
  },
  "competitor-analyst": {
    name: "Analista de concorrência",
    role: "Estuda os concorrentes a partir de informações públicas e mostra onde você está e onde pode ganhar",
    summary: "Estuda os seus concorrentes em fontes públicas e mostra onde você pode se destacar",
    about:
      "Lê os sites, preços, ofertas e avaliações dos seus concorrentes, compara com você no que os clientes valorizam e sugere movimentos. Tudo vem com fonte e data, e só usa informação pública.",
    when: "Antes de definir preços, lançar algo ou mudar como você se apresenta.",
    pairs: "Analista de preços e Estrategista de marca, que usam o que ele encontra.",
  },
  "growth-marketer": {
    name: "Marketing de crescimento",
    role: "Desenha pequenos experimentos para conseguir mais clientes e confere quais funcionaram",
    summary: "Faz pequenos experimentos medidos para descobrir o que traz mais clientes",
    about:
      "Encontra o maior vazamento entre um desconhecido e um cliente e propõe um pequeno experimento de cada vez, com o que medir e quanto esperar. Diz com honestidade quando um resultado é só ruído.",
    when: "Quando você quer mais clientes e não sabe o que tentar primeiro.",
    pairs:
      "Analista de dados, para os números, e Gestor de anúncios ou Especialista em e-mail marketing, para rodar um teste.",
  },
  "conversion-optimizer": {
    name: "Otimizador de conversão",
    role: "Descobre por que os visitantes não compram nem se cadastram e propõe correções para testar",
    summary: "Descobre por que os visitantes saem sem comprar e sugere mudanças para testar",
    about:
      "Percorre a sua página como um desconhecido no celular, encontra o que faz as pessoas duvidarem ou desistirem e propõe mudanças específicas, ordenadas pelo efeito esperado, cada uma para ser testada. Sem truques como contagem regressiva falsa.",
    when: "Quando você recebe visitas, mas poucas vendas ou cadastros.",
    pairs:
      "Redator publicitário e Designer, que fazem as mudanças, e Analista de dados, para ler os testes.",
  },
  "pr-writer": {
    name: "Assessor de imprensa",
    role: "Escreve comunicados, sugestões de pauta e notas que são precisos e têm valor de notícia",
    summary:
      "Escreve comunicados e pautas que os jornalistas conseguem usar, e que são verdadeiros",
    about:
      "Escreve comunicados à imprensa, sugestões de pauta para jornalistas e notas, e avisa quando não há uma notícia de verdade. Cada fato vem de você ou de uma fonte citada, e ele não envia nada a ninguém.",
    when: "Para um lançamento, um prêmio, um evento ou um momento delicado que você precisa explicar.",
    pairs: "Pesquisador, para encontrar veículos, e Redator, para polir.",
  },
  "reputation-manager": {
    name: "Gestor de reputação",
    role: "Lê avaliações e menções, rascunha respostas honestas e percebe padrões para corrigir",
    summary:
      "Lê as suas avaliações, rascunha respostas honestas e mostra o que os clientes repetem",
    about:
      "Organiza as suas avaliações, rascunha uma resposta para cada uma que precisa e mostra o que os clientes mais elogiam ou criticam, para você corrigir a causa. Nunca discute, nunca escreve avaliação falsa e nunca publica sozinho.",
    when: "Quando as avaliações se acumulam e você não tem tempo de responder bem.",
    pairs: "Atendimento ao cliente e Gerente de operações, para resolver problemas reais.",
  },
  "proposal-writer": {
    name: "Redator de propostas",
    role: "Escreve propostas e orçamentos claros, que combinam com o que o cliente pediu",
    summary:
      "Escreve propostas e orçamentos que respondem ao que o cliente pediu e são fáceis de aceitar",
    about:
      "Escreve o problema do cliente de volta para ele, lista exatamente o que está e o que não está incluído, organiza as etapas, o preço e as condições, e facilita o aceite. Ele pede os números a você em vez de inventar.",
    when: "Quando um cliente pede um orçamento e você quer responder bem e rápido.",
    pairs: "Analista de preços, para o preço, e Treinador de vendas, para a conversa.",
  },
  "sales-coach": {
    name: "Treinador de vendas",
    role: "Prepara para conversas de venda: perguntas, objeções e retornos, com prática",
    summary:
      "Ajuda você a se preparar para ligações de venda, lidar com objeções e fazer bons retornos",
    about:
      "Prepara você para uma conversa de venda com as perguntas certas e respostas honestas às objeções que você espera, faz o papel do cliente para você treinar e escreve o retorno. Nunca ajuda a pressionar ninguém.",
    when: "Antes de uma ligação importante, ou quando você perde sempre o mesmo tipo de venda.",
    pairs: "Pesquisador, para conhecer o cliente, e Redator de propostas.",
  },
  "pricing-analyst": {
    name: "Analista de preços",
    role: "Ajuda a definir preços a partir dos custos, dos concorrentes e do valor para o cliente, e testa opções",
    summary:
      "Ajuda você a definir preços a partir dos seus custos, do mercado e do valor para o cliente",
    about:
      "Calcula o seu custo e a sua margem de verdade, compara o que os outros cobram e o que incluem, propõe uma faixa de preço com os motivos e um pequeno teste para fazer. Quem define o preço é sempre você.",
    when: "Quando você está lançando algo, ou desconfia de que cobra pouco.",
    pairs:
      "Analista de concorrência, para o mercado, e Analista de dados, para os números de venda.",
  },
};
