# ADR 0004: várias conversas no mesmo agente

Status: proposto (2026-11-12). Muda o princípio 3 da spec ("uma conversa por bot") e estende o ADR 0001.

## Contexto

Cada agente tem hoje uma conversa só: um processo do motor, um `bots.session_id`, um chat. O dono quer abrir várias conversas no mesmo agente, como abas, para pedir coisas diferentes sem misturar contexto e sem esperar uma acabar para começar outra. Hoje o único jeito é "Começar do zero", que esquece a conversa anterior.

O código não tem id de conversa em lugar nenhum (mapa lido em 2026-11-12). Estão ligados ao agente:

- o slot do supervisor (estado, geração, fila `waiting`, sono, limite, reinício);
- o token do MCP, que diz de quem é uma chamada de ferramenta;
- o chat, as entregas, as perguntas ao dono, as tarefas entre agentes e o "lido até";
- o contexto, a compactação, as tarefas em segundo plano e o botão Parar;
- as telas que mostram ou mandam mensagem: barra lateral, chat, ilha, celular e busca.

## Decisões do dono (2026-11-12)

1. **Ao mesmo tempo.** Cada conversa é um processo próprio do motor, na mesma pasta e com a mesma memória (`CLAUDE.md`). Dá para pedir duas coisas ao mesmo agente de uma vez.
2. **A principal recebe o que vem de fora.** Pedidos de outros agentes, rotinas, lembretes e avisos do daemon entram na conversa principal, como hoje. As outras conversas são do dono. A resposta a um pedido volta para a conversa que perguntou.
3. **Título feito por IA.** Depois da primeira resposta, um modelo pequeno dá um título curto à conversa. O dono pode renomear.

## Decisão

- **Conversa** é uma entidade nova (`conversations`, id `cnv_`), com a sessão do motor, o título e o "lido até". Todo agente tem exatamente uma **principal**, criada pela migration a partir do que ele já tem (o chat e o `session_id` atuais). Com uma conversa só, nada muda para o dono.
- **O supervisor passa a cuidar de conversas, não de agentes.** O estado do agente na tela é um resumo do das conversas dele: trabalhando se alguma trabalha, esperando você se alguma espera, dormindo quando todas dormem.
- **Cada processo tem o seu token.** Ele diz o agente e a conversa, e é assim que uma tarefa ou pergunta feita numa conversa leva a resposta de volta a ela.
- **O que é do agente continua do agente:** a pasta, a memória, as regras, as ferramentas, as permissões, o modelo, o esforço, o modo, o navegador e o desktop.
- **Primeira versão só no Claude Code.** Antigravity e Codex ficam com a principal: o `agy` grava o token num arquivo da pasta, que uma segunda conversa sobrescreveria.

## Consequências

- O princípio 3 vira: "Uma conversa principal por agente, e as outras que o dono abrir". Cada conversa continua sendo uma fila em ordem, no stdin do seu processo.
- Mais processos: uma conversa parada dorme depois do tempo de sono do agente (7.7), as outras dormem mesmo com o sono desligado, e cada agente tem no máximo 3 acordadas. O `max_running` (7.8) conta processos, não agentes.
- Duas conversas mexendo nos mesmos arquivos ao mesmo tempo podem se atrapalhar. É o mesmo risco de dois agentes da crew na pasta compartilhada; o dono decide o que pede a cada uma.
- A migration acrescenta `conversation_id` às tabelas do chat e das entregas, preenchido com a principal. A cópia leve do backup (14.2) continua sem chat e sem sessões.
