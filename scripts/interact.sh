#!/bin/bash

# sXLM Protocol v2 Interaction Scripts
# Network: Testnet

VAULT="CBT3MV2YU2FBQV7QNSAKGIWYRTQTKLCXBIZBKR2T3TRDWJKOCXQ53EFV"
SXLM_TOKEN="CDTWBLUQAEXAQ6JWYZUS7ZTBFWCVBGZA5XYTTJ7C25QJX7PBTZNL6BDF"
XLM_SAC="CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC"
NETWORK="testnet"
SOURCE="deployer"

echo "=== sXLM Protocol v2 CLI ==="
echo ""

case "$1" in
  "stats")
    echo "Fetching protocol stats..."
    echo ""
    echo "Exchange Rate (scaled by 10^7):"
    stellar contract invoke --id $VAULT --source $SOURCE --network $NETWORK -- get_exchange_rate
    echo ""
    echo "Total Assets (stroops):"
    stellar contract invoke --id $VAULT --source $SOURCE --network $NETWORK -- get_total_assets
    echo ""
    echo "Total Deposits (stroops):"
    stellar contract invoke --id $VAULT --source $SOURCE --network $NETWORK -- get_total_deposits
    echo ""
    echo "Yield Accrued (stroops):"
    stellar contract invoke --id $VAULT --source $SOURCE --network $NETWORK -- get_yield_accrued
    echo ""
    echo "Total sXLM Supply (stroops):"
    stellar contract invoke --id $SXLM_TOKEN --source $SOURCE --network $NETWORK -- total_supply
    echo ""
    echo "Is Paused:"
    stellar contract invoke --id $VAULT --source $SOURCE --network $NETWORK -- is_paused
    ;;

  "deposit")
    if [ -z "$2" ]; then
      echo "Usage: ./interact.sh deposit <amount_in_stroops>"
      echo "Example: ./interact.sh deposit 1000000000  (100 XLM)"
      echo "Note: 1 XLM = 10,000,000 stroops (7 decimals)"
      exit 1
    fi
    USER=$(stellar keys address $SOURCE)
    echo "Depositing $2 stroops from $USER..."
    stellar contract invoke \
      --id $VAULT \
      --source $SOURCE \
      --network $NETWORK \
      -- deposit \
      --user $USER \
      --amount $2
    ;;

  "deposit-xlm")
    if [ -z "$2" ]; then
      echo "Usage: ./interact.sh deposit-xlm <amount_in_xlm>"
      echo "Example: ./interact.sh deposit-xlm 100"
      exit 1
    fi
    USER=$(stellar keys address $SOURCE)
    STROOPS=$(echo "$2 * 10000000" | bc | cut -d. -f1)
    echo "Depositing $2 XLM ($STROOPS stroops) from $USER..."
    stellar contract invoke \
      --id $VAULT \
      --source $SOURCE \
      --network $NETWORK \
      -- deposit \
      --user $USER \
      --amount $STROOPS
    ;;

  "withdraw")
    if [ -z "$2" ]; then
      echo "Usage: ./interact.sh withdraw <sxlm_amount_in_stroops>"
      echo "Example: ./interact.sh withdraw 500000000  (50 sXLM)"
      exit 1
    fi
    USER=$(stellar keys address $SOURCE)
    echo "Withdrawing $2 sXLM stroops for $USER..."
    stellar contract invoke \
      --id $VAULT \
      --source $SOURCE \
      --network $NETWORK \
      -- withdraw \
      --user $USER \
      --sxlm_amount $2
    ;;

  "withdraw-xlm")
    if [ -z "$2" ]; then
      echo "Usage: ./interact.sh withdraw-xlm <sxlm_amount>"
      echo "Example: ./interact.sh withdraw-xlm 50"
      exit 1
    fi
    USER=$(stellar keys address $SOURCE)
    STROOPS=$(echo "$2 * 10000000" | bc | cut -d. -f1)
    echo "Withdrawing $2 sXLM ($STROOPS stroops) for $USER..."
    stellar contract invoke \
      --id $VAULT \
      --source $SOURCE \
      --network $NETWORK \
      -- withdraw \
      --user $USER \
      --sxlm_amount $STROOPS
    ;;

  "balance")
    USER=$(stellar keys address $SOURCE)
    echo "Checking balances for $USER..."
    echo ""
    echo "XLM Balance:"
    stellar contract invoke --id $XLM_SAC --source $SOURCE --network $NETWORK -- balance --id $USER
    echo ""
    echo "sXLM Balance:"
    stellar contract invoke --id $SXLM_TOKEN --source $SOURCE --network $NETWORK -- balance --id $USER
    ;;

  "preview-deposit")
    if [ -z "$2" ]; then
      echo "Usage: ./interact.sh preview-deposit <xlm_amount_in_stroops>"
      exit 1
    fi
    echo "Preview deposit of $2 stroops XLM:"
    stellar contract invoke --id $VAULT --source $SOURCE --network $NETWORK -- preview_deposit --xlm_amount $2
    ;;

  "preview-withdraw")
    if [ -z "$2" ]; then
      echo "Usage: ./interact.sh preview-withdraw <sxlm_amount_in_stroops>"
      exit 1
    fi
    echo "Preview withdraw of $2 stroops sXLM:"
    stellar contract invoke --id $VAULT --source $SOURCE --network $NETWORK -- preview_withdraw --sxlm_amount $2
    ;;

  "add-yield")
    if [ -z "$2" ]; then
      echo "Usage: ./interact.sh add-yield <amount_in_stroops>"
      echo "Example: ./interact.sh add-yield 100000000  (10 XLM yield)"
      exit 1
    fi
    echo "Adding $2 stroops yield (admin only)..."
    stellar contract invoke \
      --id $VAULT \
      --source $SOURCE \
      --network $NETWORK \
      -- add_yield \
      --amount $2
    ;;

  "pause")
    echo "Pausing vault (admin only)..."
    stellar contract invoke \
      --id $VAULT \
      --source $SOURCE \
      --network $NETWORK \
      -- pause
    ;;

  "unpause")
    echo "Unpausing vault (admin only)..."
    stellar contract invoke \
      --id $VAULT \
      --source $SOURCE \
      --network $NETWORK \
      -- unpause
    ;;

  "set-max-deposit")
    if [ -z "$2" ]; then
      echo "Usage: ./interact.sh set-max-deposit <max_amount_in_stroops>"
      exit 1
    fi
    echo "Setting max deposit to $2 stroops (admin only)..."
    stellar contract invoke \
      --id $VAULT \
      --source $SOURCE \
      --network $NETWORK \
      -- set_max_deposit \
      --max_deposit $2
    ;;

  "info")
    echo "Contract Addresses:"
    echo "  Vault:      $VAULT"
    echo "  sXLM Token: $SXLM_TOKEN"
    echo "  XLM SAC:    $XLM_SAC"
    echo ""
    echo "Explorer Links:"
    echo "  Vault: https://stellar.expert/explorer/testnet/contract/$VAULT"
    echo "  sXLM:  https://stellar.expert/explorer/testnet/contract/$SXLM_TOKEN"
    ;;

  *)
    echo "Usage: ./interact.sh <command> [args]"
    echo ""
    echo "Commands:"
    echo "  stats              - View protocol statistics"
    echo "  info               - Show contract addresses"
    echo "  balance            - Check your XLM and sXLM balances"
    echo "  deposit <stroops>  - Deposit XLM (in stroops)"
    echo "  deposit-xlm <xlm>  - Deposit XLM (in XLM)"
    echo "  withdraw <stroops> - Withdraw sXLM (in stroops)"
    echo "  withdraw-xlm <xlm> - Withdraw sXLM (in sXLM)"
    echo "  preview-deposit    - Preview deposit output"
    echo "  preview-withdraw   - Preview withdrawal output"
    echo "  add-yield <amount> - Add yield (admin only)"
    echo "  pause              - Pause vault (admin only)"
    echo "  unpause            - Unpause vault (admin only)"
    echo "  set-max-deposit    - Set max deposit (admin only)"
    echo ""
    echo "Note: 1 XLM = 10,000,000 stroops (7 decimals)"
    ;;
esac
