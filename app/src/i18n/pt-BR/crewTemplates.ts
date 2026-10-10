import type { crewTemplates as en } from "../en/crewTemplates";

export const crewTemplates: typeof en = {
  title: "Começar com um modelo de equipe",
  intro: "Escolha um modelo de equipe pronto para trabalhar, ou comece com uma equipe vazia.",
  scratch: "Começar com uma equipe vazia",
  scratchHint: "Só o chefe. Você adiciona os agentes.",
  bots: (count: number) => (count === 1 ? "1 agente" : `${count} agentes`),
  adds: "Este modelo traz",
  change: "Escolher outro modelo",
  back: "Voltar",
  partial: (failed: number) =>
    failed === 1
      ? "A equipe foi criada, mas 1 agente não pôde ser adicionado. Você pode adicioná-lo pela Agência de agentes."
      : `A equipe foi criada, mas ${failed} agentes não puderam ser adicionados. Você pode adicioná-los pela Agência de agentes.`,
  loadFailed: "Não foi possível carregar os modelos",
  items: {
    "content-studio": {
      name: "Estúdio de conteúdo",
      summary: "Planeja, escreve, revisa e publica conteúdo para um site, uma marca ou um canal.",
      goal: "Toque um pequeno estúdio de conteúdo: planeje o que publicar, mande escrever e revisar e mantenha as redes sociais andando. A equipe já está aqui, então passe o trabalho como tarefas e me diga o que precisar.",
    },
    "software-team": {
      name: "Equipe de software",
      summary: "Projeta, constrói, testa, revisa e documenta um software.",
      goal: "Construa e mantenha o meu software: planeje o trabalho, divida entre os desenvolvedores, faça testar e revisar cada mudança e mantenha a documentação verdadeira. A equipe já está aqui, então passe o trabalho como tarefas.",
    },
    "online-store": {
      name: "Loja online",
      summary: "Cuida dos anúncios, dos clientes, das devoluções e da contabilidade de uma loja.",
      goal: "Ajude a tocar a minha loja online: mantenha os produtos e os anúncios em dia, atenda clientes e devoluções pela minha política e mantenha a contabilidade. A equipe já está aqui, então passe o trabalho como tarefas.",
    },
    "growth-team": {
      name: "Equipe de crescimento",
      summary: "Atrai novos clientes com anúncios e e-mail e confere se os números estão certos.",
      goal: "Faça a minha base de clientes crescer: planeje testes pequenos e cuidadosos de anúncios e e-mail, garanta que os resultados sejam medidos direito e me diga o que parar, manter e tentar. A equipe já está aqui, então passe o trabalho como tarefas.",
    },
    "research-desk": {
      name: "Mesa de pesquisa",
      summary: "Encontra, confere e explica o que se sabe sobre um assunto, com fontes reais.",
      goal: "Responda às minhas perguntas com pesquisa em que eu possa confiar: ache fontes, confira os fatos, faça as contas e escreva relatórios claros. A equipe já está aqui, então passe o trabalho como tarefas.",
    },
    "back-office": {
      name: "Retaguarda de um pequeno negócio",
      summary: "Mantém em ordem a contabilidade, as contas, as cobranças, o caixa e os contratos.",
      goal: "Mantenha em ordem a papelada do meu negócio: a contabilidade, as contas, as cobranças, a previsão de caixa, os impostos e as datas dos contratos. Nada é pago nem enviado sem mim. A equipe já está aqui, então passe o trabalho como tarefas.",
    },
    "personal-office": {
      name: "Escritório pessoal",
      summary: "Cuida das suas tarefas, viagens, contas, impostos, metas e reuniões.",
      goal: "Seja o meu escritório pessoal: mantenha em ordem as minhas tarefas, viagens, contas e impostos, acompanhe as minhas metas e registre as minhas reuniões. Nada é enviado nem pago sem mim. A equipe já está aqui, então passe o trabalho como tarefas.",
    },
    "course-studio": {
      name: "Estúdio de cursos",
      summary: "Transforma o que você sabe num curso online, com vídeos, slides e divulgação.",
      goal: "Transforme o que eu sei num curso online: planeje, escreva as aulas, faça os roteiros e os slides e prepare a divulgação. A equipe já está aqui, então passe o trabalho como tarefas.",
    },
    "game-studio": {
      name: "Estúdio de jogos",
      summary: "Cria um jogo: as regras, a história, as fases, os números e os testes.",
      goal: "Crie o meu jogo: dê forma às regras, escreva a história, planeje as fases, equilibre os números e aprenda com os testes. A equipe já está aqui, então passe o trabalho como tarefas.",
    },
    "people-team": {
      name: "Equipe de pessoas",
      summary: "Contrata, acolhe, treina e ouve uma equipe, sem decidir por você.",
      goal: "Ajude a cuidar da minha equipe: contratação, integração, treinamento, políticas, feedback e pesquisas. As decisões sobre pessoas são minhas. A equipe já está aqui, então passe o trabalho como tarefas.",
    },
  },
};
