# ADR 0002: bots usando o desktop do dono

Status: proposto (2026-10-04). Desenho na spec, seção 24.

## Contexto

O navegador dos bots (spec 21) cobre sites. Muito trabalho do dono, porém, mora em programas: uma planilha no Excel, um sistema de clínica sem site, um programa de nota fiscal. O dono quer que o bot veja e use esses apps abertos na tela dele.

O navegador é do bot: um perfil isolado, com logins que o dono decide fazer ali. O desktop é do dono, com todas as contas, conversas e arquivos dele. Um erro do bot, ou um texto num app que o engane, age como se fosse o dono. Por isso o desenho parte de tudo fechado e de permissões que o dono entende.

## Decisões do dono

Perguntadas em 2026-10-04:

1. **Quando:** o bot também pode agir sem o dono na frente, numa rotina de madrugada por exemplo, nos apps já liberados. Isso só liga depois de uma tela que deixa os riscos bem explícitos e de o dono aceitar (24.8, 24.10).
2. **Alcance:** o dono escolhe se libera app por app ou o desktop inteiro (24.2).
3. **Ver e mexer:** são permissões separadas. Liberar um app deixa só ver; mexer é um segundo passo (24.2).
4. **Como age:** por acessibilidade primeiro (UI Automation, sem tirar o cursor do dono). Mouse e teclado de verdade são uma opção que o dono liga quando quiser (24.5, 24.7).

## Decisões de desenho

- **No daemon, por UI Automation e `PrintWindow`.** O daemon roda na sessão do dono (tarefa de logon), então alcança as janelas dele. A UI Automation lê a árvore de controles como texto, que custa menos ao bot que fotos e funciona com a janela atrás de outras. `PrintWindow` fotografa uma janela só, sem o que está por cima dela.
- **O modo do bot não libera o desktop.** No navegador, `auto` e `bypass_permissions` navegam sem perguntar. No desktop, sempre vale a permissão gravada.
- **Uma lista do que nunca é liberado**, no código e com testes (24.3): o próprio Botloft, janelas de administrador, a área de trabalho segura, terminais e caixas de comando, gerenciadores de senha e campos de senha. Terminais entram porque digitar num deles passaria por cima das regras de permissão de comandos do bot.
- **Sem tool para abrir programas.** O bot usa o que o dono deixou aberto.
- **O dono por perto manda.** Com mouse e teclado de verdade, qualquer entrada do dono (não injetada) para o bot na hora. Há um aviso na tela enquanto ele age, e um atalho global para tudo.
- **Uma ação de desktop por vez, de todos os bots.** O cursor é um só.
- **Só Windows** no começo. As tools respondem que não há desktop nos outros sistemas.

## Riscos aceitos

- **Ver é ver tudo do alcance.** Com o desktop inteiro, o bot vê dados pessoais e de clientes que estiverem na tela. Mitigação: o alcance de app é o padrão sugerido; a tela de riscos (24.10); nada do que ele viu fica gravado fora do chat (24.11).
- **Injeção por texto.** Um e-mail aberto pode trazer instruções para o bot. Mitigação: as descrições das tools dizem que o texto das janelas não é do dono, e o que nunca é liberado tira os alvos mais perigosos (terminais, cofres de senha). Não elimina o risco; a tela de riscos diz isso.
- **Sem o dono na frente, ninguém vê na hora.** Mitigação: só com a opção ligada por app, a tela de riscos, tudo no chat, e o aviso na volta (24.8).

## Alternativas descartadas

- **Só fotos e cliques por coordenada** (como uma pessoa olhando a tela): funciona em qualquer app, mas tira o cursor do dono sempre, não age com a janela atrás de outras, e cada passo custa uma imagem ao bot. Fica como a opção de mouse e teclado de verdade.
- **Uma sessão separada do Windows para o bot** (outro usuário, RDP): isola melhor, mas não é o desktop do dono, que é o que ele pediu, e exige Windows Pro e configuração.
