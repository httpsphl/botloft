import type { base as en } from "../en/catalogBase";

export const base: typeof en = {
  developer: {
    name: "Desenvolvedor",
    role: "Escreve e muda código, roda os testes e explica o que mudou",
    summary: "Cria e conserta software, confere o próprio trabalho e conta o que fez",
    about:
      "Escreve funções novas, corrige erros e arruma o código do seu projeto. Roda os testes a cada mudança e conta, em palavras simples, o que fez e o que ainda falta.",
    when: "Quando você tem um site, um app ou um script para criar ou consertar.",
    pairs:
      "Revisor de código e Testador, que conferem o trabalho dele, e Designer, que passa as telas para ele montar.",
  },
  "code-reviewer": {
    name: "Revisor de código",
    role: "Revisa o que os outros bots mudaram e aponta erros, riscos e complicação à toa",
    summary: "Lê as mudanças com olhar crítico e diz o que está errado ou arriscado",
    about:
      "Lê as mudanças que outro bot fez e lista o que pode quebrar, o que é arriscado e o que está mais complicado do que precisa. Ele nunca edita o código, só relata, e a decisão é sua. Pensa mais fundo que a maioria dos bots, então gasta um pouco mais do seu plano.",
    when: "Depois que um Desenvolvedor muda algo importante: um pagamento, um login, qualquer coisa difícil de desfazer.",
    pairs: "Desenvolvedor, de quem ele revisa o trabalho, e Testador.",
  },
  "qa-tester": {
    name: "Testador",
    role: "Testa o que foi feito, tenta quebrar e conta como repetir cada problema",
    summary: "Usa o trabalho como uma pessoa usaria e anota como repetir cada problema",
    about:
      "Experimenta o que foi feito do jeito que uma pessoa de verdade faria, e depois do jeito de quem não tem cuidado, e relata cada problema com os passos para repetir. Ele não conserta, ele encontra.",
    when: "Antes de mostrar ou publicar algo, ou quando as pessoas dizem que quebrou.",
    pairs: "Desenvolvedor, que conserta o que ele acha, e Revisor de código.",
  },
  designer: {
    name: "Designer",
    role: "Desenha telas e páginas em HTML que você vê ao vivo e pode pedir para mudar",
    summary: "Desenha páginas e telas que você vê tomando forma e depois refina com você",
    about:
      "Desenha páginas e telas em HTML. Você vê cada uma tomando forma na área de design enquanto ele trabalha, e pede mudanças como pediria a uma pessoa.",
    when: "Para uma página de venda, uma tela de app, um cardápio, um cartão ou qualquer ideia visual que você queira ver antes de construir.",
    pairs:
      "Desenvolvedor, que pode construir o que ele desenha, e Redator, para as palavras da página.",
  },
  writer: {
    name: "Redator",
    role: "Escreve artigos, e-mails e páginas no seu tom de voz",
    summary: "Escreve textos claros que soam como você e servem a quem vai ler",
    about:
      "Escreve artigos, e-mails, textos de produto e páginas no seu tom de voz. Dê a ele um exemplo do seu jeito de escrever e ele acompanha.",
    when: "Quando as palavras precisam ser claras e soar como você.",
    pairs: "Pesquisador, para os fatos, e Tradutor, para outros idiomas.",
  },
  "social-media": {
    name: "Social media",
    role: "Planeja conteúdo e escreve posts para cada rede, mas nunca publica sozinho",
    summary: "Cria ideias, calendário e posts para você aprovar",
    about:
      "Planeja o que postar e escreve os posts para cada rede. Nunca publica sozinho: você aprova cada post.",
    when: "Quando você quer presença constante sem começar do zero toda vez.",
    pairs: "Designer, para as imagens, e Redator, para textos mais longos.",
  },
  translator: {
    name: "Tradutor",
    role: "Traduz e adapta textos entre idiomas, mantendo o tom e avisando o que se perde",
    summary: "Traduz para soar natural e avisa onde o sentido muda",
    about:
      "Traduz textos e adapta para quem vai ler, mantendo o tom e avisando onde o sentido se perde.",
    when: "Para um site em outro idioma, um e-mail a um cliente no exterior ou um documento que você não consegue ler.",
    pairs: "Redator, para refazer um texto que precisa de mais do que tradução.",
  },
  researcher: {
    name: "Pesquisador",
    role: "Pesquisa uma pergunta na web, confere as fontes e conta o que achou e o que não achou",
    summary: "Procura na web, confere as fontes e dá uma resposta curta com links",
    about:
      "Procura na web com o próprio navegador, que você pode acompanhar, compara fontes e dá uma resposta curta com os links. Ele diz o que não conseguiu achar.",
    when: "Para preços, concorrentes, regras, passo a passo ou qualquer dúvida em que você gastaria uma hora pesquisando.",
    pairs: "Redator e Analista de dados, que usam o que ele acha.",
  },
  "data-analyst": {
    name: "Analista de dados",
    role: "Lê planilhas e dados, faz as contas, mostra como chegou lá e explica o que significa",
    summary: "Transforma planilhas e números em respostas, mostra a conta e faz gráficos",
    about:
      "Lê planilhas e dados, faz as contas, mostra como chegou a cada resultado e explica o que significa, com gráficos que você vê.",
    when: "Quando você tem números (vendas, gastos, resultados) e quer respostas, não tabelas.",
    pairs: "Pesquisador, para fatos de fora dos seus dados.",
  },
  "sales-prospector": {
    name: "Prospector de vendas",
    role: "Encontra e avalia possíveis clientes e rascunha as mensagens, mas não envia nada sem você",
    summary: "Encontra pessoas e empresas que combinam, organiza e rascunha as primeiras mensagens",
    about:
      "Encontra pessoas e empresas que combinam com o que você vende, explica o porquê e rascunha a primeira mensagem para cada uma. Não envia nada sem a sua aprovação. Funciona melhor com uma ferramenta conectada à rede onde você procura clientes.",
    when: "Quando você precisa de uma lista de bons contatos e de um começo para cada conversa.",
    pairs: "Redator, para polir as mensagens, e Pesquisador, para investigar uma empresa.",
  },
  "customer-support": {
    name: "Atendimento ao cliente",
    role: "Responde dúvidas de clientes com o material que você deu e passa adiante o que não sabe",
    summary:
      "Responde dúvidas com gentileza usando o seu material e passa para você o que não sabe",
    about:
      "Responde dúvidas de clientes com o material que você dá e passa para você os casos que não resolve. Rascunha as respostas e, no começo, espera a sua aprovação.",
    when: "Quando as mesmas perguntas não param de chegar e você quer ajuda para responder bem.",
    pairs: "Desenvolvedor, para problemas técnicos, e Redator, para o jeito de dizer.",
  },
  "personal-assistant": {
    name: "Assistente pessoal",
    role: "Mantém as suas tarefas em ordem, resume o que você precisa ler e rascunha e-mails",
    summary: "Organiza as suas tarefas, resume o que você precisa ler e rascunha seus e-mails",
    about:
      "Mantém as suas tarefas em ordem, resume o que você precisa ler, rascunha e-mails e lembra do que importa. Agenda e e-mail só funcionam se você conectar uma ferramenta para isso.",
    when: "Quando o seu dia está cheio de pequenas coisas que se acumulam.",
    pairs: "Pesquisador e Redator, para trabalhos maiores.",
  },
};
