#!/usr/bin/env bash
# A copy of the accounts' database (cloud.db), kept off the machine. Run it
# from deploy/cloud, from cron once a day, for example:
#
#   17 3 * * *  cd /opt/botloft/deploy/cloud && ./backup-db.sh
#
# It uses `botloft-cloud backup`, which is safe while the server runs, then
# sends the file with rclone to a remote you set up (`rclone config`; a
# Cloudflare R2 bucket is one, see docs/cloud-deploy.md), and keeps the last
# 14 copies on the machine. The copies of the accounts are not in this file:
# they are in the bucket.
set -euo pipefail

REMOTE="${BOTLOFT_BACKUP_REMOTE:-}"          # for example r2:botloft-backups
KEEP_DAYS="${BOTLOFT_BACKUP_KEEP_DAYS:-14}"
HERE="$(cd "$(dirname "$0")" && pwd)"
STAMP="$(date +%Y-%m-%d-%H%M)"
NAME="cloud-$STAMP.db"

mkdir -p "$HERE/backups"
docker compose -f "$HERE/docker-compose.yml" exec -T cloud \
    botloft-cloud backup --config /config/cloud.toml --to "/data/backups/$NAME"
docker compose -f "$HERE/docker-compose.yml" cp "cloud:/data/backups/$NAME" "$HERE/backups/$NAME"
docker compose -f "$HERE/docker-compose.yml" exec -T cloud rm -f "/data/backups/$NAME"
chmod 600 "$HERE/backups/$NAME"

if [ -n "$REMOTE" ]; then
    rclone copy "$HERE/backups/$NAME" "$REMOTE"
else
    echo "BOTLOFT_BACKUP_REMOTE is not set: the copy stays only in $HERE/backups" >&2
fi

find "$HERE/backups" -name 'cloud-*.db' -mtime +"$KEEP_DAYS" -delete
echo "backup written: $NAME"
