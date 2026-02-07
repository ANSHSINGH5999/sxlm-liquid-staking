#!/bin/bash

# Auto-yield script - simulates 0.1% daily yield
# Run this with cron: 0 0 * * * /path/to/auto_yield.sh

VAULT_ID="CBT3MV2YU2FBQV7QNSAKGIWYRTQTKLCXBIZBKR2T3TRDWJKOCXQ53EFV"

# Get current total assets
TOTAL=$(stellar contract invoke --id $VAULT_ID --source-account GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF --network testnet -- get_total_assets 2>&1 | grep -o '"[0-9]*"' | tr -d '"')

# Calculate 0.1% yield (daily for ~36% APY)
YIELD=$((TOTAL / 1000))

if [ "$YIELD" -gt 0 ]; then
    echo "Adding yield: $YIELD stroops"
    stellar contract invoke \
        --id $VAULT_ID \
        --source-account deployer \
        --network testnet \
        --send=yes \
        -- add_yield --amount $YIELD
fi
