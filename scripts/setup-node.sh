#!/bin/bash
# Setup a new New Loka node

set -e

TIER=${1:-t0}
NODE_NAME=${2:-local-node}
DB_PATH=${3:-newloka.db}

echo "Setting up New Loka node: $NODE_NAME (tier: $TIER)"

if ! command -v newloka &> /dev/null; then
    echo "Error: newloka CLI not found. Run 'cargo install --path newloka_cli' first."
    exit 1
fi

# Initialize node
newloka init --password "$(openssl rand -base64 32)" --node "$NODE_NAME" --tier "$TIER" --db "$DB_PATH"

echo "Node setup complete. Database: $DB_PATH"
