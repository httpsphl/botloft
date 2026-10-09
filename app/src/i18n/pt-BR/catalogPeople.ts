import type { people as en } from "../en/catalogPeople";

export const people: typeof en = {
  "training-designer": {
    name: "Designer de treinamentos",
    role: "Cria treinamentos para uma equipe: o que as pessoas precisam aprender, as aulas e os exercícios, e como saber se funcionou",
    summary:
      "Cria treinamentos para sua equipe, com aulas, exercícios e um jeito de saber se funcionou",
    about:
      "Parte do que as pessoas devem fazer de diferente depois, e não do assunto. Escreve objetivos que dá para observar, aulas curtas com uma prática cada, exercícios com respostas e notas para quem vai ensinar. Também diz como você vai saber que funcionou um mês depois, e avisa se treinamento não é o que está faltando.",
    when: "Quando uma equipe precisa aprender uma habilidade, ferramenta ou jeito de trabalhar, ou um novato precisa pegar o ritmo.",
    pairs: "Gestor de mudanças, para a implantação, e Designer de apresentações, para os slides.",
  },
  "change-manager": {
    name: "Gestor de mudanças",
    role: "Planeja como levar uma mudança a uma equipe: quem ela atinge, o que dizer e quando, como ouvir as dúvidas e ver se pegou",
    summary:
      "Planeja como levar uma mudança à sua equipe: quem ela atinge, o que dizer, quando, e como ouvir as dúvidas",
    about:
      "Pega uma mudança, como uma ferramenta ou um processo novo, e planeja como levá-la a quem ela atinge: quem fica sabendo primeiro e com quais palavras, as perguntas esperadas com respostas honestas, como as dúvidas voltam para você e o apoio depois. Não anuncia nada sozinho e nunca disfarça as partes difíceis.",
    when: "Antes de introduzir algo que vai mudar o jeito de as pessoas trabalharem.",
    pairs: "Designer de treinamentos, para o aprendizado, e Redator, para as mensagens.",
  },
  "performance-review-coach": {
    name: "Coach de avaliação de desempenho",
    role: "Ajuda gestores a preparar feedbacks justos e específicos e conversas de avaliação, a partir dos fatos e exemplos que trazem",
    summary:
      "Ajuda você a preparar feedbacks justos e específicos e conversas de avaliação, com os fatos que você traz",
    about:
      "Trabalha com os exemplos que você traz e escreve um feedback específico, sobre o trabalho e equilibrado, e depois confere as próprias palavras em busca de viés. Planeja a conversa e define metas que dá para checar. Salário, promoção e advertência continuam sendo decisões suas, e ele nunca procura a pessoa.",
    when: "Antes de uma avaliação, de uma conversa difícil ou de uma rodada de feedback.",
    pairs: "Redator, para polir o texto, e Coach de metas, para as metas do próximo período.",
  },
  "hr-policy-writer": {
    name: "Redator de políticas de RH",
    role: "Redige políticas de trabalho e um manual do colaborador em palavras simples, para um advogado ou consultor conferir",
    summary:
      "Redige políticas de trabalho e um manual da equipe em palavras simples, para um profissional conferir",
    about:
      "Redige políticas sobre folgas, trabalho remoto, despesas, conduta e denúncias, cada uma com objetivo, regra e passos, mais uma versão de dois minutos. Não afirma o que diz a lei: trechos que dependem da legislação trabalhista viram perguntas para um advogado ou consultor de RH do seu país. Nada do que escreve foi conferido juridicamente.",
    when: "Quando sua equipe cresceu e as regras só existem na cabeça das pessoas.",
    pairs: "Revisor de texto, para polir, e Gestor de mudanças, para implantar as políticas.",
  },
  "engagement-survey-analyst": {
    name: "Analista de pesquisa de clima",
    role: "Cria pesquisas curtas com a equipe e lê as respostas, protegendo o anonimato, e as transforma em poucas ações claras",
    summary:
      "Cria pesquisas curtas com a equipe, lê as respostas mantendo as pessoas anônimas e acha as ações",
    about:
      "Pergunta primeiro se você vai agir com as respostas, depois escreve uma pesquisa curta e neutra e diz claramente quem verá os resultados. Lê as respostas por tema e nunca mostra o resultado de um grupo tão pequeno que alguém possa ser reconhecido. Termina com três a cinco ações e uma mensagem para a equipe.",
    when: "Quando você quer saber como a equipe realmente se sente e está pronto para fazer algo a respeito.",
    pairs:
      "Analista de dados, para muitas respostas, e Redator, para a mensagem de volta à equipe.",
  },
  "resume-tailor": {
    name: "Adaptador de currículos",
    role: "Reescreve seu currículo para uma vaga específica, a partir da sua experiência real, e prepara você para a entrevista",
    summary:
      "Reescreve seu currículo para uma vaga a partir da sua experiência real e prepara você para a entrevista",
    about:
      "Lê o anúncio da vaga, casa cada exigência com algo que você realmente fez e reescreve seu currículo e uma carta curta para aquela vaga. Pede números e exemplos, prepara as perguntas prováveis da entrevista com respostas feitas das suas próprias histórias e nunca inventa um emprego, um diploma, uma habilidade ou um número.",
    when: "Sempre que você se candidatar a uma vaga que importa.",
    pairs:
      "Coach de carreira, para o quadro maior, e Revisor de texto, para revisar os textos finais.",
  },
  "career-coach": {
    name: "Coach de carreira",
    role: "Ajuda você a pensar no seu próximo passo no trabalho: o que quer, no que é bom, as opções e um plano pequeno",
    summary:
      "Ajuda você a pensar no próximo passo no trabalho e a transformá-lo num plano pequeno e possível",
    about:
      "Escuta primeiro e depois ajuda você a ver o que quer, no que é bom e duas ou três opções reais com o que cada uma custa. Transforma sua escolha em três passos pequenos para o próximo mês e ajuda a preparar uma conversa difícil, como pedir aumento. Não é terapeuta nem consultor, e a escolha é sua.",
    when: "Quando você está em dúvida sobre o próximo passo no trabalho, ou diante de uma escolha difícil.",
    pairs:
      "Adaptador de currículos, para a candidatura, e Pesquisador, para fatos sobre uma área ou uma empresa.",
  },
};
