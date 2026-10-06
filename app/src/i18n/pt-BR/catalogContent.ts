import type { contentAndLearning as en } from "../en/catalogContent";

export const contentAndLearning: typeof en = {
  editor: {
    name: "Revisor de texto",
    role: "Revisa e edita textos: corrige erros, enxuga a escrita e mantém a voz do autor",
    summary: "Corrige erros e enxuga a sua escrita, mantendo o jeito que soa como você",
    about:
      "Corrige ortografia, gramática e frases desajeitadas e corta o que sobra, no nível que você pedir: só os erros, o jeito de dizer ou também a estrutura. Mostra cada mudança com o motivo e nunca altera um fato, um número nem uma citação.",
    when: "Antes de publicar ou enviar qualquer coisa que importa.",
    pairs:
      "Checador de fatos, para verificar afirmações, e Redator, quando uma parte precisa ser refeita.",
  },
  ghostwriter: {
    name: "Escritor fantasma",
    role: "Escreve na sua voz: posts, discursos, minibiografias e textos mais longos que você publica como seus",
    summary:
      "Escreve na sua voz o que você vai publicar com o seu nome, a partir das suas ideias e histórias",
    about:
      "Aprende como você soa, tira de você as histórias e opiniões com perguntas e escreve rascunhos na sua voz. Marca cada lacuna que preencheu para você confirmar, nunca inventa experiências e não escreve trabalhos que você precisa entregar como seus, como uma prova ou uma tese.",
    when: "Quando você tem o que dizer, mas não tem tempo nem as palavras.",
    pairs: "Pesquisador, para os fatos, e Revisor de texto, para polir.",
  },
  "fact-checker": {
    name: "Checador de fatos",
    role: "Confere afirmações em fontes confiáveis e diz o quanto tem certeza, ou que não deu para saber",
    summary:
      "Confere se uma afirmação é verdadeira em fontes confiáveis e diz o quanto tem certeza",
    about:
      "Pega as afirmações de um texto e confere cada uma nas melhores fontes que achar, preferindo o documento original. Dá um veredito em palavras simples, com o link e o quanto tem certeza, e diz com honestidade quando não deu para saber.",
    when: "Antes de publicar, e sempre que algo que você leu parece bom demais para ser verdade.",
    pairs: "Revisor de texto e Redator, que aplicam as correções.",
  },
  "academic-researcher": {
    name: "Pesquisador acadêmico",
    role: "Encontra e resume fontes acadêmicas sobre um tema, com citações reais e um relato honesto das evidências",
    summary:
      "Encontra e resume trabalhos acadêmicos sobre um tema, com citações reais e uma visão honesta das evidências",
    about:
      "Encontra artigos, livros e relatórios sobre a sua pergunta, abre cada um antes de citar e resume a conclusão, o método e os limites. Nunca inventa uma referência, diz quando só leu o resumo e não escreve trabalhos que você precisa entregar como seus.",
    when: "Quando você precisa saber o que a pesquisa realmente diz sobre um tema.",
    pairs: "Analista de dados, para os números de um estudo, e Redator, para um resumo legível.",
  },
  tutor: {
    name: "Tutor",
    role: "Ensina um assunto passo a passo, no seu nível, e confere se você realmente entendeu",
    summary:
      "Ensina um assunto a você passo a passo, no seu nível, e confere se você realmente entendeu",
    about:
      "Descobre o que você já sabe e ensina uma ideia de cada vez, com exemplos do seu mundo. Pede para você explicar de volta ou fazer um exercício pequeno, dá dicas antes das respostas e corrige com gentileza. Não faz o seu trabalho valendo nota por você.",
    when: "Quando você quer aprender um assunto direito, ou ajudar alguém que está estudando.",
    pairs: "Pesquisador, para fontes, e Redator, para resumos de estudo.",
  },
  "language-teacher": {
    name: "Professor de idiomas",
    role: "Ensina um idioma por meio de conversa, correções gentis, vocabulário e exercícios curtos",
    summary:
      "Ajuda você a aprender um idioma conversando, com correções gentis, vocabulário e exercícios curtos",
    about:
      "Conversa com você no nível certo no idioma que você está aprendendo, corrige os erros que mais importam com a regra em uma linha, ensina palavras no contexto e termina cada sessão com um resumo e um exercício pequeno. Trabalha com texto, então não ouve a sua pronúncia.",
    when: "Quando você quer praticar um idioma todo dia, sem vergonha.",
    pairs: "Tradutor, para conferir uma frase difícil, e Redator, para material de leitura.",
  },
};
