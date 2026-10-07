# Instalando o Botloft no Mac

Leva uns 10 minutos. São 3 passos: instalar o Claude Code, instalar o Botloft e liberar o app na primeira abertura.

## Antes de começar

- **Mac com chip Apple (M1, M2, M3, M4...).** Para conferir: menu  > **Sobre Este Mac** > "Chip". Se aparecer "Intel", avise: ainda não há versão para Mac Intel.
- **Uma conta Claude paga** (Pro, Max, Team ou Enterprise) ou uma chave de API da Anthropic. Os bots usam o Claude Code, que faz login com essa conta.

## Passo 1: instalar o Claude Code

Os bots do Botloft rodam em cima do Claude Code, então ele precisa estar no Mac.

1. Abra o **Terminal** (aperte `Cmd + Espaço`, digite "Terminal", Enter).
2. Cole o comando abaixo e aperte Enter:

   ```bash
   curl -fsSL https://claude.ai/install.sh | bash
   ```

3. Quando terminar, **feche o Terminal e abra de novo**, e confira:

   ```bash
   claude --version
   ```

   Deve aparecer um número de versão. Se der "command not found", rode isto e tente de novo:

   ```bash
   echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc && source ~/.zshrc
   ```

4. Faça o login, uma vez só:

   ```bash
   claude
   ```

   Ele abre o navegador para entrar na conta Claude. Depois de entrar e ver a tela inicial, digite `/exit` para sair.

(Guia oficial, se precisar: https://code.claude.com/docs/en/setup)

## Passo 2: instalar o Botloft

1. Abra https://github.com/httpsphl/botloft/releases/latest
2. Em **Assets**, baixe o arquivo que termina em **`_aarch64.dmg`**.
3. Abra o `.dmg` e arraste o **Botloft** para a pasta **Aplicativos**.

## Passo 3: liberar o app (só na primeira vez)

O app ainda não é notarizado pela Apple, então o macOS bloqueia a primeira abertura. É esperado, não é defeito.

1. Abra o **Botloft** pela pasta Aplicativos. O macOS vai dizer que não pôde verificar o app. Clique em **OK** / **Concluído**.
2. Abra **Ajustes do Sistema > Privacidade e Segurança**.
3. Role até o aviso sobre o "Botloft" e clique em **Abrir Mesmo Assim**. Confirme com a senha ou o Touch ID.
4. Abra o Botloft de novo. Dali em diante ele abre normalmente.

Se o botão "Abrir Mesmo Assim" não aparecer, o Terminal resolve:

```bash
xattr -dr com.apple.quarantine /Applications/Botloft.app
```

## Primeiro uso

1. O Botloft se prepara sozinho; não há nada para configurar.
2. Crie sua primeira equipe (crew) e diga para que ela serve.
3. Se aparecer o aviso **"Bots can't start"**, o Claude Code não foi encontrado: refaça o Passo 1 (principalmente o login) e espere até 30 segundos que o Botloft percebe sozinho.

## Se algo der errado

Mande para mim: a mensagem exata (ou um print) e o resultado destes dois comandos:

```bash
claude --version
which claude
```

## Para desinstalar

```bash
~/"Library/Application Support/Botloft/bin/botloftd" service uninstall
```

Depois arraste o Botloft da pasta Aplicativos para o Lixo.
