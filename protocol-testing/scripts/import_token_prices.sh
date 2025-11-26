#!/bin/bash
set -e

# Configuration
S3_BUCKET="repo.propellerheads-propellerheads"
TEMP_DIR="/tmp/token_price_import_$$"

usage() {
    cat << EOF
Usage: $0 --chain CHAIN [--db-url URL]

Required:
  --chain CHAIN    Chain name (ethereum, base, unichain)

Optional:
  --db-url URL     PostgreSQL connection URL 
                   (default: postgresql://postgres:mypassword@localhost:5431/tycho_indexer_0)

Example:
  $0 --chain ethereum
  $0 --chain ethereum --db-url postgresql://user:pass@host:5432/dbname
EOF
    exit 1
}

# Defaults
DB_URL="postgresql://postgres:mypassword@localhost:5431/tycho_indexer_0"

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --chain) CHAIN="$2"; shift 2 ;;
        --db-url) DB_URL="$2"; shift 2 ;;
        -h|--help) usage ;;
        *) echo "Unknown argument: $1"; usage ;;
    esac
done

if [[ -z "$CHAIN" ]]; then
    echo "Error: --chain is required"
    usage
fi

# Create temp directory
mkdir -p "$TEMP_DIR"
trap "rm -rf $TEMP_DIR" EXIT

# Find latest export
S3_PREFIX="s3://${S3_BUCKET}/token-prices/${CHAIN}/"
echo "Finding latest export for $CHAIN..."
LATEST_EXPORT=$(aws s3 ls "${S3_PREFIX}" | grep PRE | awk '{print $2}' | sed 's#/##' | sort -r | head -n 1)

if [[ -z "$LATEST_EXPORT" ]]; then
    echo "Error: No exports found for chain: $CHAIN"
    exit 1
fi

S3_EXPORT_PATH="${S3_PREFIX}${LATEST_EXPORT}/"
echo "Latest export: $LATEST_EXPORT"

# Download CSV
CSV_FILE="${TEMP_DIR}/token-prices.csv"
echo "Downloading from S3..."
aws s3 cp "${S3_EXPORT_PATH}token-prices.csv" "$CSV_FILE" --quiet

ROW_COUNT=$(($(wc -l < "$CSV_FILE") - 1))
echo "CSV contains $ROW_COUNT token prices"

# Run all operations in a single psql session
echo "Importing and updating prices..."
RESULT=$(psql "$DB_URL" -t <<EOF
CREATE TEMP TABLE temp_token_prices (
    address TEXT,
    price TEXT
);

\COPY temp_token_prices(address, price) FROM '$CSV_FILE' WITH CSV HEADER

WITH address_bytes AS (
    SELECT 
        decode(substring(address from 3), 'hex') as address_bin,
        price::float8 as price_float
    FROM temp_token_prices
),
token_lookup AS (
    SELECT 
        t.id as token_id,
        ab.price_float as price
    FROM address_bytes ab
    JOIN account a ON a.address = ab.address_bin
    JOIN token t ON t.account_id = a.id
)
INSERT INTO token_price (token_id, price)
SELECT 
    token_id, 
    price
FROM token_lookup
ON CONFLICT (token_id) 
DO UPDATE SET 
    price = EXCLUDED.price
RETURNING token_id;
EOF
)

AFFECTED_ROWS=$(echo "$RESULT" | grep -v '^$' | wc -l)
SKIPPED=$((ROW_COUNT - AFFECTED_ROWS))

echo "Done! Updated/Inserted: $AFFECTED_ROWS, Skipped: $SKIPPED"
echo "Tip: Run update_component_tvl.sh to recalculate TVL"