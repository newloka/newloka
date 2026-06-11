#!/bin/bash
# Backup script for New Loka databases

DB_PATH=${1:-newloka.db}
BACKUP_DIR=${2:-./backups}
DATE=$(date +%Y%m%d_%H%M%S)

mkdir -p "$BACKUP_DIR"

cp "$DB_PATH" "$BACKUP_DIR/newloka_$DATE.db"
echo "Backup created: $BACKUP_DIR/newloka_$DATE.db"

# Keep only last 30 backups
ls -t "$BACKUP_DIR"/newloka_*.db | tail -n +31 | xargs rm -f
