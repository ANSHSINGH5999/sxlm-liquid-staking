#!/bin/bash

# Setup cron job for automated yield (runs every 5 minutes)

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
CRON_CMD="*/5 * * * * cd $SCRIPT_DIR/.. && ./scripts/add-yield-cron.sh >> /tmp/sxlm-yield.log 2>&1"

echo "Setting up automated yield cron job..."
echo ""

# Create the cron script
cat > "$SCRIPT_DIR/add-yield-cron.sh" << 'EOF'
#!/bin/bash
cd "$(dirname "$0")/.."

VAULT="CBT3MV2YU2FBQV7QNSAKGIWYRTQTKLCXBIZBKR2T3TRDWJKOCXQ53EFV"
READ_ACCOUNT="GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF"

ASSETS=$(stellar contract invoke --id $VAULT --source-account $READ_ACCOUNT --network testnet -- get_total_assets 2>&1 | grep -oE '[0-9]+' | head -1)

if [ -n "$ASSETS" ] && [ "$ASSETS" -gt 0 ]; then
    YIELD=$((ASSETS / 1000))
    if [ "$YIELD" -gt 0 ]; then
        stellar contract invoke --id $VAULT --source-account deployer --network testnet --send=yes -- add_yield --amount $YIELD
        echo "$(date): Added yield $YIELD stroops"
    fi
fi
EOF

chmod +x "$SCRIPT_DIR/add-yield-cron.sh"

echo "Cron command to add (run 'crontab -e'):"
echo ""
echo "$CRON_CMD"
echo ""
echo "Or run manually: ./scripts/yield-bot.sh"
