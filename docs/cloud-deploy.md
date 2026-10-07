# Hospedar a nuvem do Botloft

Como pôr no ar o servidor da conta e das cópias (spec 27). Ele é um serviço pequeno, `botloft-cloud`, que guarda o e-mail de cada conta e as cópias seladas. O conteúdo das cópias ele não consegue abrir: quem sela é o dono, com a senha dele.

## O que fica onde

| O quê | Onde | Tamanho |
|---|---|---|
| Contas, aparelhos, lista de cópias, contagem dos freios | `cloud.db` (SQLite), no volume `/data` | pequeno (KB a poucos MB) |
| Os bytes das cópias | um bucket S3 (como o R2 da Cloudflare) ou a pasta `/data/copies` | uma cópia tem poucos MB; o teto é de 50 MB |
| A senha do SMTP e o segredo do bucket | arquivos em `config/` ou as variáveis `BOTLOFT_CLOUD_SMTP_PASSWORD` e `BOTLOFT_CLOUD_BUCKET_SECRET` (a variável vale mais que o arquivo), fora da imagem | — |

Se o disco da máquina é dividido com outros trabalhos, use o bucket: a VPS fica só com o `cloud.db` e com um arquivo temporário (até 50 MB) por envio em andamento.

## O que você precisa

- Uma VPS com Docker e Docker Compose (ou o binário, sem Docker: passo 6). O servidor só repassa bytes, então a máquina pode ser pequena.
- Um domínio, por exemplo `cloud.seudominio.com`, com o DNS apontando para a VPS.
- Uma conta de SMTP para mandar o link de entrada.
- Opcional, recomendado: um bucket no R2.

## 1. O proxy com TLS

O servidor só escuta em `127.0.0.1`; quem recebe a internet é um proxy com TLS. O `deploy/cloud/Caddyfile` é um exemplo: troque o endereço pelo seu, e o Caddy pega e renova o certificado sozinho.

- O `request_body { max_size 60MB }` é um pouco acima do teto de uma cópia (`max_copy_bytes`, 50 MiB). Com nginx, é `client_max_body_size 60m;`.
- O servidor usa o **último** valor de `X-Forwarded-For` como endereço de quem chama (`behind_proxy = true`). O Caddy põe esse cabeçalho; no nginx, `proxy_set_header X-Forwarded-For $remote_addr;`. Sem proxy, ponha `behind_proxy = false`.
- O Caddy não guarda log de acesso se você não ligar o `log`. Se ligar, o endereço de quem chama fica nele: mantenha por poucos dias (a spec diz 7).

## 2. O bucket no R2 (opcional)

Passos no painel da Cloudflare (confira os nomes na documentação deles, que muda):

1. R2 → criar um bucket, por exemplo `botloft-copies`.
2. R2 → gerenciar tokens de API → criar um token com permissão de leitura e escrita de objetos, só para esse bucket. Guarde o *Access Key ID* e o *Secret*.
3. O endereço é `https://<id da conta>.r2.cloudflarestorage.com` (o id da conta aparece no painel).
4. Ponha o segredo num arquivo só com ele, `config/bucket-secret`, com permissão `600`.

Depois, no `cloud.toml`: `storage = "bucket"` e a seção `[bucket]` (o exemplo está comentado no `deploy/cloud/cloud.example.toml`). As cópias já saem seladas pelo dono, então o bucket guarda bytes que ninguém abre.

## 3. O e-mail

O link de entrada precisa chegar à caixa de entrada, não ao spam. Use um provedor de SMTP que dê os registros de DNS do seu domínio (SPF e DKIM, e de preferência DMARC) e configure-os. Ponha o usuário, o servidor e o remetente na seção `[smtp]`, e a senha num arquivo só com ela (`config/smtp-password`, `600`).

Teste de verdade antes de divulgar: peça um link pelo app para um e-mail seu, em mais de um provedor (Gmail, Outlook), e olhe também o spam.

## 4. A configuração

```bash
cd deploy/cloud
mkdir -p config
cp cloud.example.toml config/cloud.toml
$EDITOR config/cloud.toml              # public_url, [smtp], e [bucket] se for usar
printf '%s' 'a-senha-do-smtp' > config/smtp-password
chmod 600 config/smtp-password
```

As chaves (todas explicadas no exemplo): `public_url` (https, vai nos links dos e-mails), `quota_bytes` (200 MiB por conta), `max_copy_bytes` (50 MiB), `keep` (5 cópias por conta), `behind_proxy`, `storage`, `[bucket]`, `[smtp]`. O servidor recusa um `public_url` que não seja https.

## 5. Subir com Docker

```bash
cd deploy/cloud
docker compose up -d --build
docker compose logs -f cloud
```

O contêiner roda sem privilégios, com o sistema de arquivos só de leitura (só `/data` escreve) e escuta em `127.0.0.1:8787` da máquina. O `HEALTHCHECK` olha `GET /health`.

## 5b. No Coolify (ou outra plataforma que guarda os segredos como variáveis)

Quando a máquina já roda Coolify, ele ocupa as portas 80 e 443 e dá o TLS: não use o Caddy. O `deploy/cloud/docker-compose.coolify.yml` serve para isso, sem publicar porta:

1. Leve a imagem à máquina (sem registro): `docker save botloft-cloud | gzip > botloft-cloud.tar.gz`, copie, e `gunzip -c botloft-cloud.tar.gz | docker load` lá.
2. No Coolify: novo recurso, "Docker Compose", e cole o arquivo.
3. Em *Domains* do serviço `cloud`, ponha o endereço com a porta do contêiner: `https://botloft.seudominio.com:8787`.
4. Nas variáveis do recurso, preencha `BOTLOFT_CLOUD_SMTP_PASSWORD` (a chave de API do Resend) e `BOTLOFT_CLOUD_BUCKET_SECRET` (o segredo do token do R2).
5. Ajuste o `BOTLOFT_CLOUD_CONFIG` do compose (a configuração inteira, como texto, sem segredo): `public_url`, o `endpoint`, o `access_key_id` e o `from`.
6. Deploy. O `/health` e o `/v1/me` (passo 7) dizem se subiu.

O compose usa `pull_policy: never`: a imagem tem que estar na máquina. Nenhum arquivo precisa ser criado na máquina: a configuração vem na variável `BOTLOFT_CLOUD_CONFIG` (quando ela existe, o servidor não lê o `cloud.toml`).

## 6. Sem Docker (systemd)

Compile (`cargo build --release -p botloft-cloud`) ou copie o binário, e siga os comandos no topo do `deploy/cloud/botloft-cloud.service`. Nesse modo, no `cloud.toml`: `listen = "127.0.0.1:8787"` e `data_dir = "/var/lib/botloft-cloud"`.

## 7. Conferir

```bash
curl -fsS https://cloud.seudominio.com/health        # ok
curl -si  https://cloud.seudominio.com/v1/me | head -1   # HTTP/2 401
```

O `401` é o certo: sem entrar, nada abre.

## 8. Apontar o app

No `config.toml` do Botloft de quem vai usar (a pasta de dados, `%LOCALAPPDATA%\Botloft`):

```toml
[cloud]
url = "https://cloud.seudominio.com"
```

Reinicie o Botloft. Em Configurações → Cópia de segurança, o bloco "Conta e cópias na nuvem" passa a pedir o e-mail. Sem esse endereço, o bloco só diz que as cópias na nuvem ainda não estão configuradas.

## 9. Backup do `cloud.db`

O `cloud.db` tem as contas e a lista de cópias; sem ele, os bytes no bucket não têm dono. O comando `botloft-cloud backup --to <arquivo>` faz uma cópia consistente com o servidor no ar. O `deploy/cloud/backup-db.sh` faz isso por Docker, copia o arquivo para `backups/` e o manda com o `rclone` para um destino seu (por exemplo outro bucket do R2: `rclone config`, tipo `s3`, provedor `Cloudflare`, o mesmo endereço):

```bash
cd deploy/cloud
BOTLOFT_BACKUP_REMOTE=r2:botloft-backups ./backup-db.sh
# todo dia às 3h17:
# 17 3 * * *  cd /opt/botloft/deploy/cloud && BOTLOFT_BACKUP_REMOTE=r2:botloft-backups ./backup-db.sh
```

Para voltar de uma cópia: pare o servidor, ponha o arquivo como `/data/cloud.db` no volume e suba de novo.

```bash
docker compose stop cloud
docker run --rm -v cloud_cloud-data:/data -v "$PWD/backups:/b:ro" debian:bookworm-slim \
    sh -c 'cp /b/cloud-AAAA-MM-DD-HHMM.db /data/cloud.db && chown 999 /data/cloud.db'
docker compose start cloud
```

O volume se chama `cloud_cloud-data` porque o Compose usa o nome da pasta (`cloud`) como prefixo; confirme com `docker volume ls`. O `chown 999` é para o usuário `botloft` da imagem.

## 10. Atualizar

```bash
git pull
cd deploy/cloud && docker compose up -d --build
```

O banco se atualiza sozinho ao subir (as migrações são numeradas e rodam uma vez). Faça um backup (passo 9) antes de uma versão nova.

## Limites conhecidos

- **Uma instância só.** O banco é um arquivo SQLite: não suba dois servidores sobre o mesmo `cloud.db`.
- **Bytes sem dono.** Se o servidor cair entre guardar os bytes de uma cópia e gravar a linha dela no banco, sobra um objeto no bucket que nenhuma conta lista. Ainda não há faxina automática; o custo é desprezível, e os objetos se reconhecem por não terem linha em `copies` (a chave é `<id da conta>/<id da cópia>`).
- **Sem envio retomável.** Uma cópia que falha no meio recomeça do início.
- **O endereço do app.** Enquanto não houver um endereço padrão no app, cada Botloft precisa do `[cloud] url` (passo 8).
