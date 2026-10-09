import type { health as en } from "../en/catalogHealth";

export const health: typeof en = {
  "appointment-prep-assistant": {
    name: "Assistente de consultas",
    role: "Ajuda você a aproveitar uma consulta médica: um histórico claro, um diário de sintomas, suas perguntas em ordem e uma anotação simples do que foi dito",
    summary:
      "Ajuda você a aproveitar uma consulta médica: seu histórico, suas perguntas e uma anotação simples depois",
    about:
      "Transforma o que vem acontecendo num resumo de uma página que o profissional lê em um minuto, com uma linha do tempo, seus remédios como estão escritos na embalagem e as perguntas a fazer por ordem de importância. Depois da consulta, ajuda você a anotar o que foi dito e nada além disso. Não adivinha o que um sintoma significa. Não é médico.",
    when: "Antes de qualquer consulta com um médico ou outro profissional de saúde, para você ou para alguém de quem você cuida.",
    pairs:
      "Organizador de registros de saúde, para os seus papéis, e Organizador do cuidado em família.",
  },
  "elder-care-companion": {
    name: "Organizador do cuidado em família",
    role: "Ajuda uma família a cuidar de um parente idoso ou doente: consultas, quem faz o quê, papéis, perguntas para a equipe de cuidado e os limites de quem cuida",
    summary:
      "Ajuda uma família a cuidar de um parente idoso ou doente: consultas, tarefas, papéis e perguntas para a equipe",
    about:
      "Mantém um calendário de consultas e renovações, um plano semanal de quem faz o quê e um resumo de uma página para dividir com a equipe de cuidado. Lista os remédios exatamente como estão escritos e nunca diz para você mudar um: essas perguntas vão para o médico ou o farmacêutico. Também cuida de você, que cuida, e ajuda nas conversas difíceis da família.",
    when: "Quando você cuida de um pai, uma mãe ou outro parente e as tarefas estão se acumulando.",
    pairs:
      "Assistente de consultas, para cada consulta, e Ajudante de contas médicas, para os custos.",
  },
  "clinic-front-desk-assistant": {
    name: "Assistente de recepção de clínica",
    role: "Ajuda uma clínica ou consultório pequeno a responder pacientes: horários, como marcar, preparo para as consultas e respostas educadas, sem dar conselho médico",
    summary:
      "Ajuda uma clínica pequena a responder pacientes: horários, agendamento, preparo e respostas educadas, sem conselho médico",
    about:
      "Mantém uma folha com seus horários, valores, regras de agendamento e preparo de cada consulta e rascunha as respostas aos pacientes a partir dela. Tudo o que fala de sintomas, remédios, resultados ou urgência fica separado para o profissional, com uma resposta curta dizendo que alguém vai responder e que, se for urgente, é para ligar para a emergência. Nunca envia uma mensagem sozinho.",
    when: "Quando os pacientes fazem as mesmas perguntas práticas o dia inteiro.",
    pairs:
      "Tradutor, para pacientes de outro idioma, e Assistente de cobrança, para os pagamentos.",
  },
  "medical-bill-helper": {
    name: "Ajudante de contas médicas",
    role: "Ajuda você a entender contas médicas e extratos do plano de saúde, achar erros e escrever um recurso ou uma pergunta claros para o plano ou o prestador",
    summary:
      "Ajuda você a entender contas médicas e extratos do plano, achar erros e escrever o recurso",
    about:
      "Explica cada linha de uma conta e do extrato do plano em palavras simples, numa tabela, e procura os erros de sempre, como um serviço cobrado duas vezes ou um pedido negado por um motivo que parece errado. Rascunha o roteiro de ligação ou a carta e mantém um registro de quem disse o quê. Nunca paga, liga nem entra em um portal por você, e não julga o tratamento.",
    when: "Quando uma conta médica ou um extrato do plano não faz sentido, ou um pedido foi negado.",
    pairs:
      "Assistente de contas, para o pagamento, e Redator de cartas de reclamação, para uma carta formal.",
  },
  "wellness-habit-coach": {
    name: "Coach de hábitos do dia a dia",
    role: "Ajuda você a criar hábitos pequenos e constantes de sono, movimento, alimentação e estresse, no seu ritmo, sem promessas médicas",
    summary:
      "Ajuda você a criar hábitos pequenos e constantes de sono, movimento, alimentação e estresse, no seu ritmo",
    about:
      "Parte do que você quer e faz cada hábito bem pequeno e ligado a algo que você já faz, com um plano para os dias ruins e uma revisão semanal simples. Nunca envergonha uma falha. Não dá dieta para doença, meta de calorias, conselho de suplemento nem tratamento de saúde mental: para isso manda você ao profissional certo, e se uma meta ficar nociva, para de orientá-la.",
    when: "Quando você quer dormir melhor, se mexer mais ou lidar com o estresse, um passo pequeno de cada vez.",
    pairs: "Coach de metas, para metas mais longas, e Assistente de consultas.",
  },
  "health-evidence-summarizer": {
    name: "Resumidor de evidências de saúde",
    role: "Encontra e resume o que a pesquisa diz sobre uma questão de saúde, com limites honestos e fontes reais, para levar a um profissional",
    summary:
      "Encontra e resume o que a pesquisa diz sobre uma questão de saúde, com fontes reais e limites honestos",
    about:
      "Escreve a sua pergunta com precisão, procura primeiro revisões de muitos estudos e diretrizes reconhecidas, abre cada fonte antes de citá-la e explica em palavras simples o tamanho do efeito, o quanto os pesquisadores têm certeza, os danos além dos benefícios e o que ainda não se sabe. Nunca inventa um estudo. É informação geral, não conselho para o seu caso, e termina com perguntas para um profissional.",
    when: "Quando você quer entender o que se sabe sobre um tratamento ou um exame antes de falar com o médico.",
    pairs: "Checador de fatos, para verificar uma citação, e Revisor de texto, para polir.",
  },
  "health-records-organizer": {
    name: "Organizador de registros de saúde",
    role: "Reúne e organiza seus papéis de saúde numa linha do tempo: consultas, exames, remédios e vacinas, sem interpretá-los",
    summary:
      "Reúne seus papéis de saúde numa linha do tempo clara: consultas, exames, remédios e vacinas",
    about:
      "Reúne seus laudos, resultados, receitas e carteira de vacinas numa só linha do tempo, copiando cada valor exatamente como está escrito, e monta um resumo de uma página para levar a qualquer consulta. Marca o que é difícil de ler ou inconsistente como pergunta para o profissional. Nunca diz que um resultado é bom, ruim ou normal, e nunca pede registros nem entra em um portal.",
    when: "Quando seus papéis de saúde estão espalhados e você precisa deles em ordem para uma consulta.",
    pairs: "Assistente de consultas, que usa o resumo numa consulta.",
  },
};
