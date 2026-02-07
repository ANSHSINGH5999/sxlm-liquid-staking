#!/bin/bash

# sXLM Automated Yield Bot
# Runs every 60 seconds, adds yield based on TVL

VAULT="CBT3MV2YU2FBQV7QNSAKGIWYRTQTKLCXBIZBKR2T3TRDWJKOCXQ53EFV"
READ_ACCOUNT="GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF"

# Yield rate: 0.1% per cycle (simulating APY)
YIELD_RATE=1  # 0.1% = 1/1000

INTERVAL=60  # seconds between yield additions

echo "🤖 sXLM Yield Bot Started"
echo "   Vault: $VAULT"
echo "   Interval: ${INTERVAL}s"
echo "   Yield Rate: 0.1% per cycle"
echo ""
echo "Press Ctrl+C to stop"
echo "================================"

while true; do
    # Get current total assets
    ASSETS=$(stellar contract invoke \
        --id $VAULT \
        --source-account $READ_ACCOUNT \
        --network testnet \
        -- get_total_assets 2>&1 | grep -oE '[0-9]+' | head -1)

    if [ -z "$ASSETS" ] || [ "$ASSETS" -eq 0 ]; then
        echo "$(date '+%H:%M:%S') | No assets in vault, waiting..."
        sleep $INTERVAL
        continue
    fi

    # Calculate yield (0.1% of TVL)
    YIELD=$((ASSETS / 1000))

    if [ "$YIELD" -gt 0 ]; then
        # Add yield to vault
        RESULT=$(stellar contract invoke \
            --id $VAULT \
            --source-account deployer \
            --network testnet \
            --send=yes \
            -- add_yield --amount $YIELD 2>&1)

        # Get new exchange rate
        RATE=$(stellar contract invoke \
            --id $VAULT \
            --source-account $READ_ACCOUNT \
            --network testnet \
            -- get_exchange_rate 2>&1 | grep -oE '[0-9]+' | head -1)

        RATE_DISPLAY=$(echo "scale=4; $RATE / 10000000" | bc)
        YIELD_XLM=$(echo "scale=4; $YIELD / 10000000" | bc)

        echo "$(date '+%H:%M:%S') | Added $YIELD_XLM XLM | Rate: $RATE_DISPLAY"
    fi

    sleep $INTERVAL
done
