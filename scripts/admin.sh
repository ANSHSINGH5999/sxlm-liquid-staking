#!/bin/bash

VAULT="CBT3MV2YU2FBQV7QNSAKGIWYRTQTKLCXBIZBKR2T3TRDWJKOCXQ53EFV"
SXLM="CDTWBLUQAEXAQ6JWYZUS7ZTBFWCVBGZA5XYTTJ7C25QJX7PBTZNL6BDF"
READ_ACCOUNT="GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF"

show_status() {
    echo ""
    echo "=== sXLM Protocol Status ==="
    echo ""

    RATE=$(stellar contract invoke --id $VAULT --source-account $READ_ACCOUNT --network testnet -- get_exchange_rate 2>&1 | grep -oE '[0-9]+')
    ASSETS=$(stellar contract invoke --id $VAULT --source-account $READ_ACCOUNT --network testnet -- get_total_assets 2>&1 | grep -oE '[0-9]+')
    SUPPLY=$(stellar contract invoke --id $SXLM --source-account $READ_ACCOUNT --network testnet -- total_supply 2>&1 | grep -oE '[0-9]+')

    echo "Exchange Rate: $(echo "scale=4; $RATE / 10000000" | bc)"
    echo "Total Assets:  $(echo "scale=2; $ASSETS / 10000000" | bc) XLM"
    echo "sXLM Supply:   $(echo "scale=2; $SUPPLY / 10000000" | bc) sXLM"
    echo ""
}

add_yield() {
    AMOUNT=$1
    if [ -z "$AMOUNT" ]; then
        echo "Usage: ./admin.sh yield <amount_in_xlm>"
        echo "Example: ./admin.sh yield 10"
        exit 1
    fi

    STROOPS=$((AMOUNT * 10000000))
    echo "Adding $AMOUNT XLM yield ($STROOPS stroops)..."

    stellar contract invoke \
        --id $VAULT \
        --source-account deployer \
        --network testnet \
        --send=yes \
        -- add_yield --amount $STROOPS 2>&1 | grep -E "(Success|yield)"

    show_status
}

case "$1" in
    status)
        show_status
        ;;
    yield)
        add_yield $2
        ;;
    help|*)
        echo ""
        echo "sXLM Admin Commands:"
        echo "--------------------"
        echo "./admin.sh status        - Show current protocol status"
        echo "./admin.sh yield <xlm>   - Add yield (e.g., ./admin.sh yield 10)"
        echo ""
        ;;
esac
