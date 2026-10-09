import type { games as en } from "../en/catalogGames";

export const games: typeof en = {
  "game-designer": {
    name: "Designer de jogos",
    role: "Transforma uma ideia de jogo em regras claras e num ciclo divertido de repetir: o objetivo, as escolhas, o retorno e uma primeira versão pequena para testar",
    summary:
      "Transforma sua ideia de jogo em regras claras e num ciclo divertido, com uma primeira versão pequena para testar",
    about:
      "Parte da experiência que você quer que o jogador tenha, acha o ciclo que ele repete e escreve as regras em frases curtas e exatas, conferindo se há brechas. Planeja a menor versão jogável e um teste, e quando algo não é divertido oferece duas mudanças para tentar. O jogo continua sendo seu.",
    when: "Quando você tem uma ideia de jogo, digital ou de mesa, e quer que ela vire algo jogável.",
    pairs:
      "Designer de narrativa, para a história, e Designer de economia de jogos, para os números.",
  },
  "game-narrative-designer": {
    name: "Designer de narrativa",
    role: "Cria a história, os personagens e o mundo de um jogo, e escreve os textos para combinar com o jeito de jogar",
    summary:
      "Cria a história, os personagens e o mundo do seu jogo e escreve os textos para combinar com o jogo",
    about:
      "Monta o mundo numa página, cria personagens com um desejo, uma falha e uma voz e planeja a história em torno das escolhas do jogador e da duração do jogo. Escreve os textos em falas curtas com o contexto de cada uma, para um desenvolvedor encaixar. Tudo é original, e ele diz para qual idade servem os temas sensíveis.",
    when: "Quando seu jogo precisa de um mundo, uma história, missões ou personagens que as pessoas lembrem.",
    pairs: "Designer de jogos, para as regras, e Tradutor, para outros idiomas.",
  },
  "level-designer": {
    name: "Designer de fases",
    role: "Desenha as fases, mapas ou cenários de um jogo para que ensinem, desafiem e surpreendam na ordem certa",
    summary:
      "Desenha as fases, mapas ou cenários do seu jogo para que ensinem, desafiem e surpreendam na ordem certa",
    about:
      "Planeja a ordem em que o jogador aprende cada mecânica, uma novidade por vez, e descreve cada fase com um mapa em texto: o começo, o objetivo, o caminho, os desafios e as recompensas. Confere se há lugares onde o jogador pode travar ou pular o objetivo da fase e escreve uma lista de teste para cada uma.",
    when: "Quando você tem as regras de um jogo e precisa das fases ou cenários que as usam.",
    pairs:
      "Designer de jogos, para conferir as regras, e Designer de narrativa, para os momentos da história.",
  },
  "game-economy-designer": {
    name: "Designer de economia de jogos",
    role: "Equilibra os números de um jogo: custos, recompensas, progresso e preços, para ele ficar justo e interessante, sem enganar o jogador para gastar",
    summary:
      "Equilibra os números do seu jogo: custos, recompensas e progresso, de forma justa e sem truques",
    about:
      "Desenha os fluxos de moedas, itens e tempo, põe os números numa planilha com fórmulas e simula jogadores em ritmos diferentes. Acha recursos que se acumulam e caminhos que são sempre os melhores. Se o seu jogo ganha dinheiro, propõe formas justas, e nunca custos escondidos, contagens falsas, vantagem por pagamento nem prêmios aleatórios pagos para crianças.",
    when: "Quando seu jogo tem moedas, itens ou progresso e os números parecem errados, ou você pretende ganhar dinheiro com ele.",
    pairs: "Designer de jogos, para os objetivos, e Analista de dados, para as simulações.",
  },
  "playtest-analyst": {
    name: "Analista de testes de jogo",
    role: "Planeja testes com jogadores e transforma o que eles fizeram e disseram em achados claros e nas mudanças que mais valem a pena",
    summary:
      "Planeja testes com jogadores e transforma o que eles fizeram e disseram em achados e mudanças que valem a pena",
    about:
      "Planeja a sessão para você não explicar nem defender o jogo e depois transforma as suas anotações em achados: o problema, quantos jogadores o encontraram, onde e por quê. Confia mais no que os jogadores fizeram do que no que disseram, ordena as mudanças pelo efeito em relação ao esforço e sugere o próximo teste. Nunca fala com os jogadores.",
    when: "Depois que pessoas testaram seu jogo e você não sabe o que mudar primeiro.",
    pairs: "Designer de jogos, que age sobre os achados, e Designer de fases.",
  },
  "game-audio-director": {
    name: "Redator de briefings de áudio",
    role: "Planeja o som e a música de um jogo e escreve briefings claros para quem os faz: o clima, a lista de sons e como tocam",
    summary:
      "Planeja o som e a música do seu jogo e escreve briefings claros para quem vai fazê-los",
    about:
      "Planeja a música e a lista de sons numa tabela: quando cada um toca, como deve soar e o que o jogador precisa entender com ele. Planeja a mixagem e avisos visuais para quem não ouve bem, e escreve um briefing por peça para um compositor ou designer de som começar. Não faz o áudio e lembra você de conferir os direitos.",
    when: "Quando seu jogo já é jogável e precisa de som, ou você vai contratar um compositor.",
    pairs:
      "Designer de jogos, para os momentos que pedem som, e Desenvolvedor, para como os sons são disparados.",
  },
  "tabletop-game-master": {
    name: "Mestre de jogos de mesa",
    role: "Ajuda a conduzir um RPG de mesa: prepara aventuras, personagens e respostas sobre regras, e mantém a história andando na mesa",
    summary:
      "Ajuda a conduzir um RPG de mesa: aventuras, personagens, respostas sobre regras e ideias para a mesa",
    about:
      "Pergunta o seu sistema, o seu grupo e o tom, e prepara uma aventura curta com um gancho, cenas com uma escolha cada e vários finais, além de personagens rápidos. Responde dúvidas de regras com o livro que você tem e diz quando não tem certeza. Durante o jogo dá três opções quando você trava e pergunta ao grupo sobre limites.",
    when: "Quando você conduz ou joga um RPG e quer ajuda para preparar ou improvisar.",
    pairs: "Designer de narrativa, para um enredo mais fundo, e Designer, para mapas e materiais.",
  },
};
