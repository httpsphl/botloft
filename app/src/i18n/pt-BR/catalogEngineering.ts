import type { engineering as en } from "../en/catalogEngineering";

export const engineering: typeof en = {
  "software-architect": {
    name: "Arquiteto de software",
    role: "Desenha como um sistema se encaixa e registra as decisões, as trocas e os riscos",
    summary: "Desenha como as partes de um sistema se encaixam e escreve o porquê",
    about:
      "Decide com você como um software deve ser estruturado, escolhe o desenho mais simples que atende ao que você precisa e escreve cada decisão importante com os motivos, os custos e os riscos. Ele desenha; não escreve o código do produto.",
    when: "Antes de construir algo grande, ou quando um sistema ficou difícil de mudar.",
    pairs:
      "Desenvolvedor, Desenvolvedor front-end, back-end e mobile, que constroem a partir do desenho, e Revisor de segurança.",
  },
  "frontend-developer": {
    name: "Desenvolvedor front-end",
    role: "Constrói e conserta a parte de um site ou app que as pessoas veem e usam, rápida no celular e utilizável por todos",
    summary: "Constrói as telas que as pessoas usam, rápidas no celular e utilizáveis por todos",
    about:
      "Constrói e conserta páginas, formulários e menus, pensando primeiro no celular, no teclado, nos leitores de tela e nas conexões lentas. Confere o trabalho num navegador de verdade e roda os testes do projeto.",
    when: "Quando um site ou app precisa de uma tela nova, ou o que existe está lento ou confuso.",
    pairs: "Designer, que desenha as telas, e Desenvolvedor back-end, que fornece os dados.",
  },
  "backend-developer": {
    name: "Desenvolvedor back-end",
    role: "Constrói e conserta o lado do servidor: APIs, tratamento de dados, tarefas em segundo plano e seus testes",
    summary:
      "Constrói a parte que fica atrás da tela: APIs, dados e tarefas em segundo plano, com testes",
    about:
      "Constrói e conserta o que roda atrás da tela: servidores, APIs, regras sobre dados e tarefas, com testes e com atenção ao que acontece quando algo falha. Nunca roda mudanças em dados de verdade sem perguntar a você.",
    when: "Quando o seu produto precisa guardar, processar ou compartilhar dados, ou se conectar a outro serviço.",
    pairs:
      "Engenheiro de banco de dados, para os dados, e Desenvolvedor front-end, que usa as APIs.",
  },
  "mobile-developer": {
    name: "Desenvolvedor mobile",
    role: "Constrói e conserta apps de celular e confere em telas, redes e aparelhos realistas",
    summary:
      "Constrói e conserta apps de celular, pensando em telas pequenas, redes lentas e bateria",
    about:
      "Constrói e conserta apps de celular pensando em telas pequenas, redes ruins, interrupções e permissões. Diz com honestidade o que não conseguiu testar sem um aparelho de verdade, e nunca mexe nas suas contas de loja nem nas chaves de assinatura.",
    when: "Quando você quer um app para celular, ou o que você tem está lento ou fecha sozinho.",
    pairs: "Designer, para as telas, Desenvolvedor back-end, para a API, e Testador.",
  },
  "database-engineer": {
    name: "Engenheiro de banco de dados",
    role: "Desenha tabelas, escreve e acelera consultas e planeja mudanças seguras em um banco de dados",
    summary:
      "Desenha o seu banco de dados, deixa as consultas rápidas e muda dados sem perder nada",
    about:
      "Desenha como os seus dados são guardados, deixa consultas lentas rápidas medindo antes e planeja cada mudança para poder ser desfeita, com cópia de segurança antes. Testa numa cópia e nunca roda nada destrutivo em dados de verdade.",
    when: "Quando um banco de dados está lento, bagunçado ou precisa mudar sem perder nada.",
    pairs:
      "Desenvolvedor back-end, que aplica as mudanças, e Engenheiro de dados, que carrega os dados.",
  },
  "devops-engineer": {
    name: "Engenheiro DevOps",
    role: "Monta compilações, testes, publicações e monitoramento, e mantém as esteiras e os servidores saudáveis",
    summary: "Automatiza compilar, testar e publicar, e mantém tudo rodando e vigiado",
    about:
      "Deixa publicar software seguro e sem drama: compilações e testes automáticos, publicações repetíveis, registros e alertas, e um jeito de voltar atrás em cada versão. Pergunta antes de mexer em qualquer sistema de verdade e nunca pede para você colar senhas no chat.",
    when: "Quando publicar é manual e assustador, ou algo vive quebrando em produção.",
    pairs:
      "Desenvolvedor back-end, para o que o serviço precisa, e Revisor de segurança, para acessos e segredos.",
  },
  "security-reviewer": {
    name: "Revisor de segurança",
    role: "Revisa o seu próprio código e a sua configuração atrás de falhas de segurança e explica como corrigir",
    summary:
      "Procura falhas de segurança no seu código e na sua configuração e explica como corrigir",
    about:
      "Revisa o seu código e a sua configuração atrás de falhas, explica cada uma em palavras simples com a gravidade e a menor correção. Trabalha na defesa, só no que é seu, e nunca repete um segredo que encontrar. Pensa mais fundo que a maioria dos agentes, então gasta um pouco mais do seu plano.",
    when: "Antes de lançar, depois de uma mudança grande, ou quando você lida com dados de outras pessoas.",
    pairs: "Desenvolvedor e Engenheiro DevOps, que aplicam as correções, e Revisor de código.",
  },
  "data-engineer": {
    name: "Engenheiro de dados",
    role: "Constrói esteiras confiáveis que movem, limpam e guardam dados para os analistas poderem confiar neles",
    summary: "Constrói os canos que levam dados limpos e confiáveis até onde você os analisa",
    about:
      "Constrói os caminhos que os dados percorrem do lugar onde nascem até onde são analisados: coleta, limpa e confere em cada passo e para quando algo parece errado, em vez de passar números ruins adiante. Mantém os seus originais intactos.",
    when: "Quando os seus números estão em muitos lugares ou você não confia nos seus relatórios.",
    pairs:
      "Engenheiro de banco de dados, para onde guardar, e Analista de dados, que usa o resultado.",
  },
};
