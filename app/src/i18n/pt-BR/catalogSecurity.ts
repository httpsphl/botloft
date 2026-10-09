import type { security as en } from "../en/catalogSecurity";

export const security: typeof en = {
  "threat-modeler": {
    name: "Modelador de ameaças",
    role: "Mapeia o que pode dar errado num sistema antes de ele ser feito ou mudado: o que proteger, quem pode atacar, como e o que fazer",
    summary:
      "Mapeia o que pode dar errado no seu sistema e o que fazer, antes de ele ser feito ou mudado",
    about:
      "Desenha o seu sistema como uma lista curta de partes e fluxos, marca onde a confiança muda e pergunta o que pode dar errado em cada lugar. Ordena as ameaças por probabilidade e gravidade, em palavras simples, e propõe defesas com o custo, separando o que consertar antes do lançamento do que é bom ter.",
    when: "Antes de construir ou mudar algo que guarda dados valiosos ou faz algo importante.",
    pairs:
      "Revisor de segurança, para conferir o código depois de pronto, e Arquiteto de software.",
  },
  "incident-responder": {
    name: "Coordenador de incidentes",
    role: "Ajuda você num incidente de segurança: conter, entender o que houve, consertar, avisar quem precisa e aprender",
    summary:
      "Ajuda você num incidente de segurança, passo a passo: conter, entender, consertar e aprender",
    about:
      "Quando uma chave vaza, uma conta é tomada ou algo parece errado, dá o próximo passo, mantém a linha do tempo e diz quais passos são difíceis de desfazer. Pede que você guarde as provas, ajuda a ver até onde foi, lista quem pode precisar ser avisado e escreve uma revisão sem culpados no fim. As ações são suas.",
    when: "Assim que você suspeitar que algo deu errado com a segurança.",
    pairs:
      "Auditor de senhas e chaves, para achar outras chaves expostas, e Desenvolvedor back-end, para consertar a causa.",
  },
  "secrets-auditor": {
    name: "Auditor de senhas e chaves",
    role: "Acha senhas, chaves e tokens esquecidos no seu código, nos arquivos e nas configurações e planeja como trocá-los e protegê-los, sem repetir os valores",
    summary:
      "Acha senhas, chaves e tokens esquecidos no código e nos arquivos e planeja como trocá-los e protegê-los",
    about:
      "Procura no seu código, no histórico, nas configurações e nos documentos por senhas, chaves e tokens, e diz onde está cada um e de que tipo é, nunca o valor. Trata cada achado de verdade como vazado, dá a ordem para trocá-lo e planeja como manter segredos fora do código daqui para frente. Nunca usa um segredo que encontra.",
    when: "Antes de tornar um projeto público, compartilhá-lo ou entregá-lo.",
    pairs: "Engenheiro DevOps, para o armazenamento seguro, e Revisor de segurança.",
  },
  "privacy-engineer": {
    name: "Engenheiro de privacidade",
    role: "Confere quais dados pessoais seu produto coleta e guarda, se precisa deles e como protegê-los e respeitar as escolhas das pessoas",
    summary:
      "Confere quais dados pessoais seu produto coleta e guarda e como protegê-los e respeitar as escolhas",
    about:
      "Mapeia cada tipo de dado pessoal que você coleta, para onde vai e por quanto tempo fica, e pergunta se você precisa dele. Confere quem pode lê-lo, se aparece nos registros e se as pessoas conseguem ver, corrigir ou apagar os dados. Lista as leis que podem valer como perguntas para um advogado e nunca diz que algo está em conformidade.",
    when: "Antes de lançar, de coletar um dado novo, ou quando um cliente pergunta como você trata os dados dele.",
    pairs:
      "Redator de políticas de RH, para avisos em linguagem simples, e Desenvolvedor back-end, para as mudanças.",
  },
  "compliance-checklist": {
    name: "Assistente de conformidade",
    role: "Transforma uma norma de segurança ou privacidade numa lista simples, mostra o que você já faz e o que falta e reúne as provas",
    summary:
      "Transforma uma norma de segurança ou privacidade numa lista simples e mostra o que você faz e o que falta",
    about:
      "Pega a norma ou o questionário de cliente que você precisa cumprir, transforma cada exigência numa pergunta simples e confere com o que você mostra. Só marca um item como feito com prova, planeja as lacunas por esforço e efeito e rascunha respostas honestas para um questionário. Não é auditor e nada do que escreve é certificação.",
    when: "Quando um cliente, um contrato ou um mercado pede que você comprove sua segurança ou privacidade.",
    pairs: "Engenheiro de privacidade e Auditor de senhas e chaves, para achados específicos.",
  },
  "ai-code-auditor": {
    name: "Auditor de código feito por IA",
    role: "Revisa código escrito por uma IA atrás dos erros que ela costuma cometer: funções inventadas, segurança fraca, conferências que faltam e código que ninguém entende",
    summary:
      "Revisa código escrito por uma IA atrás dos erros que ela costuma cometer, antes de você confiar nele",
    about:
      "Confere se as bibliotecas e funções que o código usa existem de verdade, procura os buracos básicos de segurança que esse tipo de código costuma pular, testa os casos de borda e vê se os testes testam alguma coisa. Também avisa se o código está mais complicado do que precisa. Você recebe um veredito: seguro, seguro depois destes ajustes, ou não use.",
    when: "Antes de um código escrito por uma IA chegar a usuários ou dados de verdade.",
    pairs: "Revisor de código, para uma segunda olhada, e Revisor de segurança.",
  },
  "cloud-config-reviewer": {
    name: "Revisor de configuração na nuvem",
    role: "Revisa as configurações da sua nuvem e dos seus servidores atrás de escolhas arriscadas: portas abertas, acesso demais, registros e cópias que faltam",
    summary:
      "Revisa as configurações da sua nuvem e dos servidores atrás de portas abertas, acesso demais e cópias que faltam",
    about:
      "Lê as configurações que você exporta e acha os riscos de sempre: armazenamento aberto para o mundo, contas com acesso demais, sem segundo fator, sem cópias de segurança ou com cópias nunca testadas. Cada achado vem com a prova e a menor mudança segura, e ele avisa das mudanças que podem trancar você do lado de fora. Nunca entra na sua conta.",
    when: "Depois de montar uma conta de nuvem ou um servidor, e uma vez por ano daí em diante.",
    pairs: "Engenheiro DevOps, que aplica as mudanças, e Auditor de senhas e chaves.",
  },
};
