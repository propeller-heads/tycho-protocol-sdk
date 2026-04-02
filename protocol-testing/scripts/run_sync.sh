#!/usr/bin/env bash
# run_sync.sh — Clean sync script for Supernova V3 indexer
# 
# This script handles the historical partition gap that causes duplicate key
# errors when syncing February 2026 data into a fresh database.
#
# Usage:
#   export SUBSTREAMS_API_TOKEN="your_token"
#   bash protocol-testing/scripts/run_sync.sh
#
set -euo pipefail

COMPOSE="docker compose -f protocol-testing/docker-compose.yaml"
DB_NAME="tycho_indexer_0"
PSQL="$COMPOSE exec -T db psql -U postgres -d $DB_NAME"

echo "======================================================"
echo " Supernova V3 Clean Sync"
echo "======================================================"

# --- 1. Wipe everything ---
echo ""
echo "[1/5] Wiping containers and volumes..."
$COMPOSE down -v --remove-orphans
echo "      Done."

# --- 2. Start DB only, wait for healthy ---
echo ""
echo "[2/5] Starting database..."
$COMPOSE up -d db
echo "      Waiting for DB to be ready..."
until $COMPOSE exec -T db pg_isready -U postgres > /dev/null 2>&1; do
    sleep 1
done
echo "      DB is ready."

# --- 3. Run Tycho migrations (start indexer without token → exits with Unauthenticated) ---
echo ""
echo "[3/5] Running Tycho migrations..."
# Start indexer with EMPTY token so it runs migrations then exits cleanly
SUBSTREAMS_API_TOKEN="" $COMPOSE up indexer 2>&1 | head -30 || true
echo "      Migrations complete."

# --- 4. Pre-create February 2026 partitions (NOW that protocol_state table exists) ---
echo ""
echo "[4/5] Pre-creating historical partitions for Feb-Mar 2026..."
$PSQL -v ON_ERROR_STOP=1 << 'EOSQL'
DO $$
DECLARE
    d DATE := '2026-02-01';
    created INT := 0;
BEGIN
    WHILE d < '2026-06-01' LOOP
        EXECUTE format(
            'CREATE TABLE IF NOT EXISTS protocol_state_p%s PARTITION OF protocol_state FOR VALUES FROM (%L) TO (%L)',
            to_char(d, 'YYYYMMDD'), d::timestamptz, (d + 1)::timestamptz
        );
        EXECUTE format(
            'CREATE TABLE IF NOT EXISTS component_balance_p%s PARTITION OF component_balance FOR VALUES FROM (%L) TO (%L)',
            to_char(d, 'YYYYMMDD'), d::timestamptz, (d + 1)::timestamptz
        );
        created := created + 1;
        d := d + 1;
    END LOOP;
    RAISE NOTICE 'Created % daily partition pairs (protocol_state + component_balance)', created;
END $$;
SELECT COUNT(*) AS feb_partitions_ready FROM pg_tables WHERE tablename LIKE 'protocol_state_p202602%';
EOSQL
echo "      Partitions created."

# --- 5. Start indexer with real token ---
echo ""
echo "[5/5] Starting indexer with SUBSTREAMS_API_TOKEN..."
if [ -z "${SUBSTREAMS_API_TOKEN:-}" ]; then
    echo "ERROR: SUBSTREAMS_API_TOKEN is not set!"
    echo "Please run: export SUBSTREAMS_API_TOKEN='your_token'"
    exit 1
fi

$COMPOSE up -d indexer
echo ""
echo "======================================================"
echo " Indexer started! Following logs..."
echo " (Press Ctrl+C to stop following, indexer keeps running)"
echo "======================================================"
$COMPOSE logs -f indexer
