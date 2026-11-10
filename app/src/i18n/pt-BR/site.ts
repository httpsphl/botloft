// O site público (spec 39).

import type { SiteMessages } from "../en/site";
import { siteDemos } from "./siteDemos";

export const site: SiteMessages = {
  meta: {
    title: "Botloft: seus agentes de IA, sempre ligados, trabalhando juntos",
    description:
      "Um app gratuito para o computador que mantém uma equipe de agentes de IA trabalhando, sobre o Claude Code. Eles passam trabalho uns para os outros, continuam depois que você fecha a janela e pedem antes de agir.",
  },
  skip: "Pular para o conteúdo",
  nav: {
    label: "Seções",
    features: "Recursos",
    how: "Como começar",
    privacy: "Privacidade",
    faq: "Perguntas",
    github: "GitHub",
    language: "Idioma",
  },
  download: {
    windows: "Baixar para Windows",
    macos: "Baixar para macOS",
    linux: "Baixar para Linux",
    phone: "Baixe no seu computador",
    short: "Baixar",
    all: "Todos os downloads",
    windowsFile: "Windows 10 ou 11",
    macosFile: "macOS, Apple Silicon (prévia)",
    debFile: "Linux .deb (prévia)",
    appImageFile: "Linux AppImage (prévia)",
  },
  hero: {
    badge: "Gratuito · Windows, Linux e macOS",
    title: "Seus agentes de IA, sempre ligados, trabalhando juntos.",
    lead: "O Botloft mantém uma equipe de agentes trabalhando no seu computador, cada um sobre o Claude Code. Eles passam trabalho uns para os outros, continuam depois que você fecha a janela e pedem a você antes de agir.",
    github: "Código no GitHub",
    needs: "Você precisa do Claude Code, com a sua própria conta.",
    needsLink: "Como instalar",
    demo: "A equipe trabalhando no Botloft: o Scout pede para usar um site, acha as fontes e passa para o Writer",
  },
  facts: {
    label: "Em resumo",
    items: [
      "Roda no seu computador",
      "Continua quando você fecha",
      "Pede antes de agir",
      "Claude Code, Antigravity, Codex",
      "Gratuito, com o código aberto ao público",
    ],
  },
  cards: {
    title: "Uma equipe que trabalha enquanto você faz outra coisa",
    lead: "Cada agente tem sua pasta, sua memória e sua conversa. Você fala com cada um como num app de mensagens e decide o que eles podem fazer.",
    browser: {
      title: "Veja eles trabalhando",
      body: "Acompanhe ao vivo cada página que um agente abre, e assuma o controle para entrar numa conta por ele.",
    },
    ask: {
      title: "Você decide o que eles podem fazer",
      body: "Um comando, uma edição de arquivo, um site novo: um cartão pergunta antes, a não ser que você já tenha deixado.",
    },
    handoff: {
      title: "Eles passam trabalho uns para os outros",
      body: "Os agentes trocam mensagens e tarefas. Cada uma é guardada antes de sair e reenviada até chegar.",
    },
    routines: {
      title: "Rotinas com horário",
      body: "Nas manhãs de dia útil, a cada duas horas ou no horário que quiser. Ou um atalho que roda com um clique.",
    },
    phone: {
      title: "Pelo celular",
      body: "Aprove, responda e converse de qualquer lugar, com tudo lacrado de ponta a ponta entre o celular e o computador.",
    },
    maker: {
      title: "O Criador monta para você",
      body: "Peça equipes, agentes e rotinas com as suas palavras. Você vê o plano inteiro antes de qualquer coisa ser criada, e Desfazer volta atrás.",
    },
    sleep: {
      title: "Sempre ligados, sem atrapalhar",
      body: "Os agentes continuam depois que você fecha o Botloft. Os parados dormem, sem gastar memória, e acordam com a mesma conversa.",
    },
  },
  demos: siteDemos,
  steps: {
    title: "Funcionando em três passos",
    items: {
      engine: {
        title: "Instale o Claude Code",
        body: "Instale e entre uma vez com a sua própria conta. Seus agentes usam o seu plano.",
        link: "Instalar o Claude Code",
      },
      app: {
        title: "Baixe o Botloft",
        body: "Abra e ele se configura sozinho: não há nada para ajustar. Ele também se atualiza sozinho.",
      },
      crew: {
        title: "Crie sua primeira equipe",
        body: "Diga para que ela serve e conheça os agentes que ela sugere.",
      },
    },
  },
  engines: {
    title: "Escolha o motor",
    lead: "Cada agente roda no motor de IA para programação que você escolher, instalado no seu computador e com a sua própria conta.",
    claude: {
      title: "Claude Code",
      tag: "Experiência completa",
      body: "Todos os recursos: pede antes de cada ferramenta, com “sempre permitir”, o navegador ao vivo, as rotinas e quanto resta do seu plano.",
    },
    others: {
      title: "Antigravity e Codex",
      tag: "Experimentais",
      body: "Desligados até você ligar nas Configurações. Conversam, recebem tarefas e falam com a equipe; alguns recursos ainda não foram conferidos com eles.",
    },
  },
  privacy: {
    title: "Seu, no seu computador",
    lead: "O Botloft roda no seu computador. A conta é opcional, e nada sai sem você pedir.",
    items: {
      local: {
        title: "Sem telemetria",
        body: "Sem conta, o Botloft usa a rede só para as atualizações e para o trabalho dos seus agentes.",
      },
      sealed: {
        title: "Cópias lacradas",
        body: "A cópia opcional na nuvem vai lacrada com uma senha que nunca sai do seu computador. O servidor não consegue abri-la.",
      },
      keys: {
        title: "Chaves fora da conversa",
        body: "Quando um agente precisa de uma senha ou de um token, você digita num campo escondido. Ela nunca entra na conversa.",
      },
      code: {
        title: "Código aberto ao público",
        body: "Leia cada linha no GitHub. Cada versão também passa a ser Apache 2.0 dois anos depois de sair.",
      },
    },
  },
  faq: {
    title: "Perguntas",
    items: {
      cost: {
        q: "O Botloft custa alguma coisa?",
        a: "Não. O Botloft é gratuito, em casa ou no trabalho. Os agentes rodam no motor que você escolher, com a sua própria conta, então usam os limites do seu plano; com o Claude Code, o Botloft mostra quanto resta.",
      },
      window: {
        q: "Preciso deixar a janela aberta?",
        a: "Não. Os agentes rodam em segundo plano, mesmo depois que você fecha o Botloft, e começam quando você entra no computador.",
      },
      systems: {
        q: "Em quais computadores ele roda?",
        a: "Windows 10 e 11 (64 bits). Linux (64 bits, .deb ou AppImage) e macOS com Apple Silicon rodam como prévia; Macs com Intel ainda não são suportados.",
      },
      warning: {
        q: "Por que o computador me avisa quando instalo?",
        a: "O instalador do Windows ainda não tem assinatura de código, então o SmartScreen não conhece quem o publica: escolha “Mais informações” e depois “Executar assim mesmo”. A Apple também ainda não autenticou o app do macOS: permita uma vez em Ajustes do Sistema, Privacidade e Segurança. Cada versão é montada pelo GitHub a partir do código público, e as atualizações são assinadas e conferidas antes de instalar.",
      },
      anthropic: {
        q: "O Botloft é feito pela Anthropic?",
        a: "Não. O Botloft é um projeto independente. Ele usa o Claude Code, o Antigravity e o Codex instalados no seu computador e não tem ligação nem endosso da Anthropic, do Google ou da OpenAI.",
      },
    },
  },
  closing: {
    title: "Dê aos seus agentes um lugar para trabalhar.",
    body: "Gratuito, e pronto em poucos minutos.",
  },
  footer: {
    links: "Links",
    tagline: "Seus agentes de IA, sempre ligados, trabalhando juntos.",
    releases: "Versões",
    license: "Licença (FSL-1.1-ALv2)",
    signing: "Política de assinatura de código",
    security: "Segurança",
    account: "O que a conta guarda",
    notAffiliated: "Sem ligação com a Anthropic, o Google ou a OpenAI.",
  },
};
