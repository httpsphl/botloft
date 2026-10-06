import type { product as en } from "../en/catalogProduct";

export const product: typeof en = {
  "product-manager": {
    name: "Gerente de produto",
    role: "Transforma ideias e retornos em um plano claro: o que construir, em que ordem e por quê",
    summary: "Decide o que construir primeiro e deixa por escrito para os outros construírem",
    about:
      "Olha as suas ideias, os retornos que você recebeu e os números, e ajuda a escolher o que vale construir agora. Escreve cada escolha como um resumo curto, que o resto da equipe usa para construir.",
    when: "Quando você tem mais ideias do que tempo e precisa escolher.",
    pairs:
      "Analista de requisitos, para detalhar cada item, e Desenvolvedor e Designer, que constroem.",
  },
  "project-manager": {
    name: "Gerente de projetos",
    role: "Planeja um projeto, acompanha quem faz o quê até quando e avisa o que está atrasado ou travado",
    summary:
      "Divide um projeto em etapas, acompanha o andamento e avisa cedo quando algo escorrega",
    about:
      "Transforma um objetivo em um plano com etapas, responsáveis e datas, mantém tudo atualizado e avisa cedo quando algo atrasa ou trava.",
    when: "Para qualquer coisa com várias etapas e pessoas: um lançamento, uma reforma, uma mudança, uma campanha.",
    pairs: "Facilitador ágil, para o ritmo do trabalho, e Secretário de reuniões.",
  },
  "business-analyst": {
    name: "Analista de requisitos",
    role: "Transforma um pedido vago em requisitos claros, perguntas e critérios para saber quando está pronto",
    summary: "Faz as perguntas certas e escreve exatamente o que precisa ser construído",
    about:
      "Pega um pedido vago como 'preciso de um jeito de acompanhar pedidos' e o transforma em requisitos precisos, com as exceções e o jeito de todo mundo saber que está certo.",
    when: "Antes de construir qualquer coisa que ainda não está clara.",
    pairs:
      "Gerente de produto, que define as prioridades, e Desenvolvedor e Testador, que usam os requisitos.",
  },
  "ux-researcher": {
    name: "Pesquisador de usuários",
    role: "Planeja como descobrir o que os usuários precisam e transforma conversas e retornos em conclusões",
    summary: "Descobre o que os seus usuários realmente precisam, pelo que dizem e pelo que fazem",
    about:
      "Planeja entrevistas, questionários e testes, e transforma as anotações e respostas que você traz em conclusões com evidências. Ele não conversa com ninguém sozinho: prepara e analisa.",
    when: "Antes de uma decisão importante sobre um produto, uma página ou um serviço.",
    pairs:
      "Gerente de produto e Designer, que usam as conclusões, e Analista de dados, para os números dos questionários.",
  },
  "agile-facilitator": {
    name: "Facilitador ágil",
    role: "Conduz planejamento, conversas rápidas de acompanhamento e retrospectivas, e mantém o trabalho da equipe andando",
    summary:
      "Mantém o trabalho da equipe andando em passos pequenos, com acompanhamento e retrospectivas honestas",
    about:
      "Mantém um quadro simples do que vem a seguir, do que está em andamento e do que está pronto, pede atualizações rápidas aos bots e conduz uma retrospectiva ao fim de cada rodada para mudar uma coisa.",
    when: "Quando a equipe tem muitos bots e o trabalho vive travando ou sendo esquecido.",
    pairs: "Gerente de projetos, para o plano, e Secretário de reuniões.",
  },
  "goals-coach": {
    name: "Coach de metas",
    role: "Ajuda a definir metas e medidas claras e confere o progresso em relação a elas",
    summary: "Transforma o que você quer em metas que dá para medir e confere como você está indo",
    about:
      "Ajuda você a dizer o que quer de um jeito que dá para conferir, escolhe poucas medidas que mostram o progresso e olha os números com você com regularidade, sendo honesto quando você está fora do rumo.",
    when: "Quando você quer fazer algo crescer e não tem certeza de que está funcionando.",
    pairs: "Analista de dados, que traz os números, e Gerente de projetos.",
  },
  "meeting-secretary": {
    name: "Secretário de reuniões",
    role: "Prepara pautas, faz anotações e transforma reuniões em decisões e tarefas",
    summary: "Prepara a reunião, escreve o que aconteceu e transforma em decisões e coisas a fazer",
    about:
      "Prepara pautas e, a partir das anotações ou da transcrição que você der, escreve as decisões, as tarefas e as perguntas em aberto. Ele não entra em chamadas e não envia nada sozinho.",
    when: "Quando as reuniões terminam sem um registro claro de quem vai fazer o quê.",
    pairs: "Gerente de projetos e Facilitador ágil, que acompanham as tarefas.",
  },
  "process-analyst": {
    name: "Analista de processos",
    role: "Mapeia como o trabalho é feito hoje, encontra desperdício e escreve um jeito melhor em passos claros",
    summary:
      "Desenha como o trabalho realmente acontece, acha o que faz perder tempo e escreve um jeito mais simples",
    about:
      "Mapeia um processo como ele realmente funciona, encontra passos repetidos, esperas e erros, e escreve uma versão mais simples como uma lista que uma pessoa nova conseguiria seguir.",
    when: "Quando o mesmo trabalho é lento, tem muitos erros ou só existe na cabeça de alguém.",
    pairs: "Gerente de operações e Analista de dados, que ajudam a medir.",
  },
  "operations-manager": {
    name: "Gerente de operações",
    role: "Mantém o dia a dia funcionando: rotinas, listas de conferência, fornecedores e o que vive escorregando",
    summary:
      "Mantém o dia a dia funcionando: rotinas, listas de conferência e as coisas que vivem escorregando",
    about:
      "Acompanha o que precisa acontecer todo dia, toda semana e todo mês, transforma o que está na sua cabeça em listas de conferência, vigia renovações e estoque e avisa antes de algo ser esquecido. Pode propor rotinas que rodam em horários marcados.",
    when: "Quando tocar o negócio está consumindo o tempo de que você precisa para crescer.",
    pairs: "Analista de processos, que melhora as rotinas, e Assistente pessoal.",
  },
  recruiter: {
    name: "Recrutador",
    role: "Escreve vagas, organiza candidaturas por critérios claros e prepara entrevistas",
    summary:
      "Escreve a vaga, organiza as candidaturas por critérios claros e prepara boas perguntas de entrevista",
    about:
      "Escreve a vaga, define como é um bom candidato, compara as candidaturas que você der com esses critérios e prepara as mesmas perguntas de entrevista para todos. Ele recomenda, você decide, e ele nunca entra em contato com candidatos.",
    when: "Quando você está contratando e quer um processo justo e organizado.",
    pairs: "Redator, para polir a vaga, e Pesquisador, para olhar o mercado.",
  },
  "onboarding-coach": {
    name: "Coach de integração",
    role: "Prepara as primeiras semanas de uma pessoa nova: o que aprender, quem conhecer e o que fazer primeiro",
    summary:
      "Planeja as primeiras semanas de uma pessoa nova para ela ficar útil logo e se sentir bem-vinda",
    about:
      "Planeja o primeiro dia, a primeira semana e o primeiro mês de quem chega, escreve a mensagem de boas-vindas e um guia de como as coisas funcionam, e lista o que precisa estar pronto antes de a pessoa começar.",
    when: "Quando alguém está entrando e você quer que fique útil rápido.",
    pairs: "Analista de processos, que mapeia o trabalho, e Redator, para polir o guia.",
  },
  "customer-success": {
    name: "Sucesso do cliente",
    role: "Ajuda os clientes a tirar valor: acompanha, percebe quem está em risco e planeja o próximo passo",
    summary:
      "Ajuda os clientes a tirar valor de verdade, percebe quem está insatisfeito e sugere o que fazer",
    about:
      "Mantém uma tabela simples dos seus clientes, percebe os que estão em risco (menos uso, pagamento atrasado, reclamações) e rascunha uma mensagem curta e calorosa para cada um que precisa de atenção. Não envia nada sem a sua aprovação.",
    when: "Quando você tem clientes recorrentes e quer que menos deles vão embora.",
    pairs: "Atendimento ao cliente, para as dúvidas, e Analista de dados, para os números de uso.",
  },
  "event-planner": {
    name: "Organizador de eventos",
    role: "Planeja um evento do começo ao fim: orçamento, programação, fornecedores e lista de conferência",
    summary: "Planeja o seu evento do orçamento ao último item da lista, para nada ser esquecido",
    about:
      "Planeja eventos de qualquer tamanho: orçamento por item, um cronograma contado de trás para a frente a partir da data, opções de fornecedores para comparar, a programação do dia e uma lista para o dia seguinte. Não reserva nem convida ninguém.",
    when: "Para um lançamento, uma festa, uma oficina ou uma reunião de família.",
    pairs: "Pesquisador, para comparar fornecedores, e Redator, para os convites.",
  },
  "travel-planner": {
    name: "Planejador de viagens",
    role: "Planeja uma viagem: roteiro, hospedagem, programação por dia e orçamento, com opções e o que reservar",
    summary:
      "Planeja a sua viagem com opções, roteiro dia a dia e orçamento, e diz o que reservar primeiro",
    about:
      "Pesquisa rotas, hospedagens e passeios, propõe opções, monta um plano dia a dia com tempos realistas e um orçamento, e lista o que reservar e até quando. Nunca reserva e nunca digita seus documentos ou cartões em lugar nenhum.",
    when: "Para férias, uma viagem a trabalho ou uma visita à família.",
    pairs:
      "Pesquisador, para olhar um lugar mais a fundo, e Analista de dados, para comparar custos.",
  },
};
