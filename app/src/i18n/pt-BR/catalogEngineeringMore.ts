import type { engineeringMore as en } from "../en/catalogEngineeringMore";

export const engineeringMore: typeof en = {
  "technical-writer": {
    name: "Redator técnico",
    role: "Escreve documentação clara a partir do código e da configuração reais: guias, referências e passo a passo que batem com o que o sistema faz",
    summary:
      "Escreve documentação clara a partir do seu código e da sua configuração reais, que continua verdadeira",
    about:
      "Lê o seu código e roda numa cópia os comandos que documenta, para que os passos e a saída sejam reais. Escreve para um leitor de cada vez: a tarefa mais comum primeiro, passos numerados, o que você deve ver depois de cada um e o que fazer quando falha. Diz o que testou e o que só leu.",
    when: "Quando seu projeto não tem README, a documentação está velha ou alguém novo vai assumir.",
    pairs:
      "Guia de código, para o panorama geral, e Revisor de código, para conferir as afirmações técnicas.",
  },
  "sre-engineer": {
    name: "Engenheiro de confiabilidade",
    role: "Deixa seu serviço confiável: define o que é bom em números, monta alertas que importam e escreve os passos para quando quebrar",
    summary:
      "Deixa seu serviço confiável: metas claras, alertas que importam e passos para quando quebrar",
    about:
      "Transforma o que seus usuários sentem em poucas metas em números, mapeia onde o serviço pode falhar e propõe alertas que significam que alguém precisa agir agora, cada um com a primeira coisa a conferir. Escreve guias curtos para as falhas comuns e uma revisão sem culpados depois de um incidente. Nunca reinicia nem muda nada em produção sozinho.",
    when: "Quando seu serviço cai com frequência demais, ou você recebe alertas que ignora.",
    pairs:
      "Engenheiro DevOps, para a infraestrutura, e Desenvolvedor back-end, para as correções no código.",
  },
  "release-engineer": {
    name: "Engenheiro de lançamentos",
    role: "Planeja e prepara lançamentos: números de versão, notas de mudança, conferências antes de publicar e um jeito de voltar, sem publicar por você",
    summary:
      "Planeja e prepara seus lançamentos, com notas, lista de conferência e caminho de volta, e nunca publica sozinho",
    about:
      "Lê o que mudou desde o último lançamento, agrupa pelo que os usuários percebem e propõe o número da versão e as notas em palavras simples. Escreve uma lista de conferência, os passos para publicar e os passos para desfazer, e diz qual deles é difícil de desfazer. A etiqueta, o envio e o anúncio são seus, e ele nunca mexe nas suas chaves de assinatura.",
    when: "Antes de todo lançamento de um app, uma biblioteca ou um site.",
    pairs: "Testador, para rodar a lista, e Engenheiro DevOps, para a esteira.",
  },
  "codebase-guide": {
    name: "Guia de código",
    role: "Explica um código desconhecido: o panorama geral, onde cada coisa está, como uma requisição percorre o sistema e por onde começar a mudar",
    summary:
      "Explica um código desconhecido: o panorama geral, onde cada coisa está e por onde começar",
    about:
      "Adapta a visita ao que você precisa fazer, segue um caminho real do começo ao fim por arquivo e função e aponta as convenções, as áreas de risco e os testes. Diz do que tem certeza e o que deduziu, e termina com uma primeira tarefa pequena. Só lê; não muda nada se você não pedir.",
    when: "Quando você herda código, entra num projeto ou precisa mudar algo que não escreveu.",
    pairs: "Redator técnico, para virar documentação, e Arquiteto de software.",
  },
  "prompt-engineer": {
    name: "Redator de instruções para IA",
    role: "Escreve e melhora as instruções que você dá a modelos de IA e as testa com exemplos reais antes de você confiar nelas",
    summary:
      "Escreve e melhora instruções para modelos de IA e as testa com exemplos reais antes de você confiar",
    about:
      "Pede exemplos reais, inclusive difíceis, escreve a instrução com objetivo claro, regras, formato da resposta e exemplos, e a testa mudando uma coisa por vez. Você recebe uma tabela de resultados de cada versão e os pontos fracos que restam. Avisa quando a instrução vai ler um texto de fora que pode dar ordens.",
    when: "Quando um recurso de IA no seu app ou fluxo dá respostas em que você não pode confiar.",
    pairs: "Desenvolvedor back-end, para ligar tudo, e Testador, para tentar mais casos.",
  },
  "rapid-prototyper": {
    name: "Criador de protótipos",
    role: "Monta rápido uma versão que funciona de uma ideia para testar com pessoas de verdade, do jeito mais simples, e diz o que é de mentira",
    summary:
      "Monta rápido uma versão da sua ideia para testar com pessoas de verdade e diz o que é de mentira",
    about:
      "Parte da pergunta que o protótipo precisa responder e constrói só isso, com dados inventados e sem contas reais. Escreve um teste curto para quem vai experimentar e uma nota dizendo o que é de mentira e o que uma versão de verdade precisa, para ninguém publicar o protótipo por engano.",
    when: "Quando você tem uma ideia e quer saber se as pessoas se importam antes de construir direito.",
    pairs: "Designer, para a aparência, e Pesquisador de usuários, para conduzir o teste.",
  },
  "localization-engineer": {
    name: "Engenheiro de localização",
    role: "Prepara o software para vários idiomas e países: tira o texto do código, trata datas, números e plurais e confere o resultado",
    summary: "Prepara seu software para vários idiomas e países e confere se nada quebra",
    about:
      "Tira o texto do seu código e põe em arquivos de tradução, com uma frase por chave e marcadores em vez de pedaços colados. Trata plurais, datas, números e moedas pela região do usuário, confere o layout com palavras longas e cria um teste que falha quando falta um texto num idioma. Entrega os textos novos a um tradutor.",
    when: "Quando você quer seu app ou site num segundo idioma, ou num país com formatos diferentes.",
    pairs: "Tradutor, para os textos, e Desenvolvedor front-end, para o layout.",
  },
  "refactoring-engineer": {
    name: "Engenheiro de limpeza de código",
    role: "Arruma o código em passos pequenos e seguros, sem mudar o que ele faz, provando cada passo com testes",
    summary:
      "Arruma seu código em passos pequenos e seguros, sem mudar o que ele faz, com testes em cada passo",
    about:
      "Parte de um motivo, garante que existam testes antes de mexer e muda uma coisa de cada vez, cada uma num commit pequeno. Não mistura arrumação com mudança de comportamento e nunca altera um teste só para a arrumação passar. Lista o que viu e deixou quieto.",
    when: "Quando o código está difícil de mudar ou entender e você quer melhorá-lo sem quebrar.",
    pairs: "Revisor de código, para conferir cada passo, e Testador, para tentar o comportamento.",
  },
};
