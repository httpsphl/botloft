# ADR 0001: bots em Claude Code headless, com chat

Status: aceito (2026-09-28). Substitui o desenho de runtime do M2–M4 (ConPTY, terminal e inbox por named pipe).

## Contexto

Até o M4 cada bot rodava o Claude Code interativo numa ConPTY. O app mostrava o terminal (xterm.js), e as mensagens entre bots entravam pelo inbox nativo do Claude Code, um named pipe.

Funcionou, mas o dono quer outra coisa: conversar com cada bot como num app de mensagens. As mensagens dele e do bot aparecem em balões, o que o bot fez aparece resumido, os pedidos de aprovação têm botões e dá para mandar imagens e arquivos.

Um terminal não dá isso. Da TUI só saem pixels, e o conteúdo estruturado (texto do bot, ferramentas usadas, pedidos de permissão) não está disponível.

O teste real no M4 também mostrou dois atritos do caminho antigo:

- **Autoridade do dono:** tudo que entra pelo inbox o Claude Code apresenta como vindo "de outra sessão", inclusive a mensagem do dono, e isso não vale como aprovação.
- **Replay:** reenviar a tela ao reabrir o terminal fazia o xterm responder de novo a consultas antigas do terminal.

## Decisão

Cada bot roda como Claude Code headless: `claude -p --input-format stream-json --output-format stream-json`, com stdin e stdout em pipes. Não há PTY, TUI nem inbox.

- **Tudo entra pelo stdin**, uma mensagem de usuário por linha JSON: o que o dono escreve, o que outros bots mandam e os avisos do daemon. A mensagem do dono vai como o texto dele, com a autoridade de quem digita. Mensagens de bots e avisos levam o envelope (spec 9.3).
- **Tudo sai pelo stdout como eventos.** O daemon transforma os eventos em itens de chat, grava no SQLite e manda ao app: texto da resposta, ferramentas com resultado, fim de turno e erros. O texto parcial vai ao vivo, sem gravar.
- **Permissões:** `--permission-prompt-tool` aponta para uma ferramenta do servidor MCP do próprio daemon. Ela segura a chamada até o dono responder no chat (Permitir ou Negar) ou até vencer o prazo.
- **Estados** saem do fluxo de eventos, sem hooks: turno em andamento, aprovação pendente, erro de limite ou de login.
- **Sessão:** o daemon guarda o `session_id` e reinicia com `--resume`.
- **Isolamento:** `--setting-sources project,local` e `--strict-mcp-config` deixam de fora os hooks, skills, agents, permissões e servidores MCP pessoais do dono. O bot vê só o que o Botloft gera.
- **Anexos** vão para a pasta do bot. Imagens também seguem inline, como blocos `image`.

## Verificado com Claude Code 2.1.284

- Um processo atende vários turnos. Mensagem escrita durante um turno entra na fila e vira o turno seguinte.
- O processo fica calado até a primeira mensagem e sai com 0 quando o stdin fecha.
- `--replay-user-messages` devolve cada mensagem com o `uuid` que o daemon pôs, quando o turno dela começa.
- Um bloco `image` na mensagem de entrada é aceito.
- `--permission-prompt-tool mcp__<srv>__<tool>` chama a ferramenta com `{tool_name, input, tool_use_id}`, espera a resposta (testado com 20 s) e segue `{"behavior":"allow","updatedInput":...}`.
- `--permission-prompts host` sem o aperto de mão do SDK nega tudo (`system/permission_denied`). O protocolo de controle do SDK não é documentado, então não é usado.
- `--setting-sources project,local` tirou os hooks e skills do usuário.
- Login pela assinatura funciona (`apiKeySource: none`).

## O que se perde

- **A TUI do Claude Code:** não há `/login`, seletor de `/model` nem `/mcp` interativos.
  - Login e troca de conta ficam fora do bot (`claude auth login` num terminal). O app diz isso quando o bot cai em `auth_error`.
  - Comandos com barra que o modo `-p` aceita continuam funcionando no chat.
- **O formato de entrada do `stream-json` não é documentado.** Ele foi verificado na prática, e a seção 19 da spec registra a versão testada.
- **O ConPTY, o terminal com replay, o inbox por named pipe e os hooks** do M2–M4 saem do código.

## Alternativa descartada

Manter a TUI e montar o chat lendo o transcript `.jsonl` da sessão. A documentação diz que esse formato é interno e muda entre versões. As aprovações e os anexos continuariam dependendo do terminal, e a mensagem do dono continuaria com autoridade de colega.
