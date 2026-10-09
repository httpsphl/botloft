import type { legal as en } from "../en/catalogLegal";

export const legal: typeof en = {
  "contract-reader": {
    name: "Leitor de contratos",
    role: "Lê um contrato em palavras simples: o que cada lado deve fazer, as datas e o dinheiro, as cláusulas arriscadas e as perguntas para um advogado",
    summary:
      "Lê um contrato em palavras simples e aponta as cláusulas arriscadas e o que perguntar a um advogado",
    about:
      "Lê o contrato inteiro e entrega uma página: quem deve fazer o quê, o dinheiro, as datas e como termina. Aponta as cláusulas que merecem atenção, como renovação automática, multas ou renúncia de direitos, e marca cada uma como comum, incomum ou que vale negociar. Nunca diz que uma cláusula é válida ou exigível. Não é advogado e termina com as perguntas para levar a um.",
    when: "Antes de assinar um contrato, ou quando você quer entender um que já assinou.",
    pairs: "Controle de prazos de contratos, para as datas, e Assistente de pesquisa jurídica.",
  },
  "terms-drafter": {
    name: "Redator de termos e privacidade",
    role: "Redige os termos de uso e o aviso de privacidade de um site ou app em palavras simples, a partir de como o negócio realmente funciona, para um advogado conferir",
    summary:
      "Redige os termos e o aviso de privacidade do seu site em palavras simples, para um advogado conferir",
    about:
      "Pergunta como o seu negócio realmente funciona e que dados ele coleta, e então redige os termos de uso, o aviso de privacidade e a política de troca e entrega em frases curtas e simples. Os trechos que dependem da lei viram perguntas para um advogado, e ele lista as promessas que você vai precisar cumprir. Nunca publica nada no seu site.",
    when: "Quando você lança um site, um app ou uma loja e precisa dos textos que vão junto.",
    pairs: "Engenheiro de privacidade, para o mapa de dados, e Revisor de texto, para polir.",
  },
  "legal-research-assistant": {
    name: "Assistente de pesquisa jurídica",
    role: "Encontra e explica as leis, normas e decisões sobre uma questão em fontes oficiais, com citações exatas, e diz o que não conseguiu verificar",
    summary:
      "Encontra as leis e decisões sobre uma questão em fontes oficiais, com citações exatas e limites honestos",
    about:
      "Precisa do lugar, da data e dos fatos, e então procura primeiro em fontes oficiais, abre cada fonte antes de citá-la e entrega um memorando curto com referências exatas. Separa o que a lei diz do que os tribunais dizem sobre ela e do que está em aberto. Nunca inventa uma lei nem um caso, e diz quando não conseguiu verificar um.",
    when: "Quando você precisa saber o que a lei diz sobre uma questão, antes de falar com um advogado.",
    pairs:
      "Checador de fatos, para verificar uma citação, e Revisor de texto, para polir o memorando.",
  },
  "legal-intake-assistant": {
    name: "Assistente de triagem de clientes",
    role: "Ajuda um pequeno escritório ou consultor a reunir os fatos de um novo cliente: um questionário claro, um resumo organizado do caso e os documentos que faltam",
    summary:
      "Ajuda um pequeno escritório a reunir os fatos de um novo cliente: questionário claro, resumo organizado e o que falta",
    about:
      "Prepara um formulário simples para o tipo de caso e depois transforma as respostas do cliente numa linha do tempo e num resumo de uma página, com fatos separados de opiniões. Lista os documentos que faltam e põe o que é urgente no topo. Nunca diz ao cliente quais são os direitos dele nem se ele tem um caso, e nunca entra em contato com ele.",
    when: "No primeiro passo com um novo cliente, antes de o advogado analisar o caso.",
    pairs: "Assistente de horas e cobrança, e Leitor de contratos.",
  },
  "legal-billing-assistant": {
    name: "Assistente de horas e cobrança",
    role: "Transforma o seu registro de trabalho em lançamentos de horas claros e rascunhos de fatura para um escritório ou consultor, e mostra o que ainda não foi cobrado",
    summary:
      "Transforma o seu registro de trabalho em lançamentos de horas e rascunhos de fatura, e mostra o que falta cobrar",
    about:
      "Transforma suas anotações e agenda em lançamentos de horas com descrições que um cliente entenderia e que não revelam mais do que precisam. Mostra o que ainda não foi cobrado por cliente, aponta lançamentos que parecem errados sem alterá-los e prepara o rascunho da fatura com os lançamentos que você aprovar. Nunca arredonda para cima nem exagera o trabalho.",
    when: "Quando você cobra por hora e o tempo escapa antes de você faturar.",
    pairs: "Assistente de cobrança, para acompanhar pagamentos, e Auxiliar contábil.",
  },
  "contract-deadline-tracker": {
    name: "Controle de prazos de contratos",
    role: "Acompanha as datas dos seus contratos: renovações, avisos prévios, pagamentos e prazos, e avisa com bastante antecedência",
    summary:
      "Acompanha as datas dos seus contratos, como renovações e avisos prévios, e avisa com antecedência",
    about:
      "Lê os seus contratos e lista cada data e obrigação com a cláusula de onde vem, calculando a data real e dizendo em palavras simples como foi contada. Põe as armadilhas primeiro, como renovações automáticas com aviso prévio, e avisa cedo. Se uma cláusula não está clara, diz isso em vez de chutar. Nunca envia um aviso nem encerra um contrato.",
    when: "Quando você tem vários contratos e tem medo de perder uma renovação ou um aviso.",
    pairs:
      "Leitor de contratos, para explicar uma cláusula, e Redator, para rascunhar um aviso que você envia.",
  },
  "complaint-letter-drafter": {
    name: "Redator de cartas de reclamação",
    role: "Redige cartas claras, firmes e educadas para uma reclamação ou disputa com uma empresa ou um proprietário, a partir dos fatos e dos papéis que você tem",
    summary:
      "Redige cartas de reclamação claras, firmes e educadas para uma disputa com uma empresa ou proprietário",
    about:
      "Organiza os fatos com você, monta uma linha do tempo conferida com os seus papéis e redige uma carta curta, educada e firme: o que aconteceu, o que você pede e um prazo razoável para resposta. Não ameaça nem exagera. Lista os anexos e os lugares para tentar depois, como perguntas para o seu país. A carta, quem envia é você.",
    when: "Quando uma empresa, um proprietário ou um vendedor não agiu direito com você.",
    pairs: "Leitor de contratos, para as cláusulas, e Revisor de texto, para revisar.",
  },
};
