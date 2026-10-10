import type { design as en } from "../en/catalogDesign";

export const design: typeof en = {
  "brand-guardian": {
    name: "Guardião da marca",
    role: "Mantém tudo fiel à marca: escreve um guia curto com o seu material e confere as peças com ele",
    summary: "Escreve um guia curto da marca com o seu material e confere se tudo segue o guia",
    about:
      "Reúne seu logo, cores, fontes e exemplos num guia de uma página, marcando o que viu e o que sugere. Depois confere páginas, posts e apresentações com o guia e lista exatamente onde cada um foge dele. Nunca inventa um fato da marca nem muda uma regra sem perguntar.",
    when: "Quando você tem uma marca e várias pessoas ou agentes criam coisas para ela.",
    pairs: "Designer, que aplica os ajustes visuais, e Redator, que aplica os ajustes de voz.",
  },
  "ui-designer": {
    name: "Designer de interface",
    role: "Monta o sistema visual de uma interface: cores, fontes, espaçamento e componentes, numa página de estilo",
    summary:
      "Monta cores, fontes, espaçamento e botões de uma interface e mostra tudo numa página de estilo",
    about:
      "Escolhe uma paleta pequena, uma escala de fontes e um passo de espaçamento, e constrói botões, campos, cartões e avisos com todos os estados numa página de estilo que você vê. Cada tela seguinte reaproveita essas peças, e o produto parece uma coisa só.",
    when: "No começo de um produto, ou quando suas telas deixaram de parecer da mesma família.",
    pairs:
      "Designer, que faz as páginas, Desenvolvedor front-end, que as constrói, e Revisor de acessibilidade.",
  },
  "ux-architect": {
    name: "Arquiteto de experiência",
    role: "Planeja como um produto se organiza e como as pessoas passam por ele: mapas, fluxos e esboços simples",
    summary:
      "Planeja como um produto se organiza e como as pessoas o percorrem, com fluxos e esboços simples",
    about:
      "Parte de quem são as pessoas e do que querem fazer, agrupa o conteúdo do jeito que elas pensam e desenha cada fluxo, com erros, telas vazias e o voltar. Os esboços são telas cinza e simples, para você julgar a estrutura antes de qualquer enfeite. Lista os lugares em que as pessoas podem se perder.",
    when: "Antes de desenhar ou construir um produto, um site ou uma novidade grande.",
    pairs:
      "Pesquisador de usuários, para testar as hipóteses, e Designer de interface, que dá aparência aos esboços.",
  },
  "accessibility-reviewer": {
    name: "Revisor de acessibilidade",
    role: "Confere se páginas e telas servem a pessoas com habilidades diferentes e explica cada ajuste em palavras simples",
    summary:
      "Confere se suas páginas funcionam para pessoas com habilidades diferentes e explica cada ajuste",
    about:
      "Lê o código de uma página e confere o que dá para ver nele: contraste, descrição de imagens, rótulos de campos, títulos, uso pelo teclado, foco visível e informação dada só pela cor. Cada problema traz quem ele atinge e o ajuste exato. Diz o que não consegue conferir, como um leitor de tela de verdade, e nunca chama uma página de certificada.",
    when: "Antes de lançar uma página e depois de uma mudança grande nela.",
    pairs:
      "Desenvolvedor front-end, que aplica os ajustes, e Designer de interface, que já planeja isso desde o começo.",
  },
  "presentation-designer": {
    name: "Designer de apresentações",
    role: "Transforma sua mensagem numa apresentação clara: a história, uma ideia por slide e um visual limpo, em telas",
    summary:
      "Transforma o que você quer dizer numa apresentação clara, uma ideia por slide, que você vê nascer",
    about:
      "Escreve a história primeiro e depois desenha cada slide como uma tela que você vê aparecer na área de design: uma ideia, um título que diz o ponto, texto grande e no máximo uma imagem ou gráfico. Os detalhes vão para as notas de fala. Os slides são telas, não um arquivo de PowerPoint.",
    when: "Quando você precisa apresentar, vender uma ideia ou ensinar algo e quer clareza.",
    pairs:
      "Pesquisador, para os fatos, Redator, para as palavras, e Designer de infográficos, para os gráficos.",
  },
  "infographic-designer": {
    name: "Designer de infográficos",
    role: "Transforma números e ideias em gráficos e infográficos claros, desenhados como telas, sem entortar os dados",
    summary: "Transforma números e ideias em gráficos e infográficos claros, sem entortar os dados",
    about:
      "Confere de onde vêm os números, escolhe a forma que deixa a mensagem clara e a desenha como tela: escalas honestas, rótulos diretos, poucas cores, a fonte e a data na imagem. Diz o que a imagem não mostra e nunca inventa um número para tapar uma lacuna.",
    when: "Quando uma tabela ou um relatório é difícil de ler e uma imagem resolveria melhor.",
    pairs:
      "Analista de dados, para os números, e Designer de apresentações, para pôr tudo numa apresentação.",
  },
  "image-prompt-writer": {
    name: "Redator de pedidos de imagem",
    role: "Escreve descrições claras e consistentes para você colar num gerador de imagem ou vídeo e as melhora com os resultados",
    summary:
      "Escreve descrições claras e consistentes para o gerador de imagem ou vídeo que você usa e as melhora",
    about:
      "Não faz as imagens: escreve as descrições que você cola na ferramenta que usa, com o assunto, o cenário, o estilo, a luz e o enquadramento, em duas ou três versões. Para um conjunto que precisa ter o mesmo visual, escreve um parágrafo de estilo para repetir. Mostre o resultado e ele muda só a parte que causou o problema.",
    when: "Quando você usa um gerador de imagem ou vídeo e os resultados saem aleatórios ou diferentes entre si.",
    pairs: "Guardião da marca, para manter o estilo na marca, e Social media, para os posts.",
  },
  "design-critic": {
    name: "Crítico de design",
    role: "Analisa telas e páginas com olhar novo e entrega uma lista curta e ordenada do que melhorar, e por quê",
    summary: "Olha suas telas com olhar novo e diz, em ordem, o que melhorar e por quê",
    about:
      "Lê o objetivo e os arquivos da tela e diz primeiro as três coisas que mais importam: o que o olho vê primeiro, se a ação principal está clara, o espaçamento, as fontes, o contraste e como funciona numa tela pequena. Separa o que é problema claro do que é gosto, diz o que manter e não refaz a tela a menos que você peça.",
    when: "Quando uma tela parece estranha e você não sabe dizer por quê, ou antes de publicar.",
    pairs: "Designer, que faz as mudanças, e Revisor de acessibilidade, para o acesso.",
  },
};
