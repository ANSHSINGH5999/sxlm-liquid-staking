import {
  Contract,
  TransactionBuilder,
  nativeToScVal,
  scValToNative,
  xdr,
  Account,
  Horizon,
  Operation,
  SorobanDataBuilder
} from '@stellar/stellar-sdk';
import { Server as SorobanServer, Api } from '@stellar/stellar-sdk/rpc';

export const CONFIG = {
  vault: 'CBT3MV2YU2FBQV7QNSAKGIWYRTQTKLCXBIZBKR2T3TRDWJKOCXQ53EFV',
  sxlmToken: 'CDTWBLUQAEXAQ6JWYZUS7ZTBFWCVBGZA5XYTTJ7C25QJX7PBTZNL6BDF',
  xlmNative: 'CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC',
  networkPassphrase: 'Test SDF Network ; September 2015',
  horizonUrl: 'https://horizon-testnet.stellar.org',
  sorobanUrl: 'https://soroban-testnet.stellar.org',
  decimals: 7
};

export const horizonServer = new Horizon.Server(CONFIG.horizonUrl);
export const sorobanServer = new SorobanServer(CONFIG.sorobanUrl);

export function stroopsToXlm(stroops: bigint | number | string): number {
  return Number(stroops) / Math.pow(10, CONFIG.decimals);
}

export function xlmToStroops(xlm: number): bigint {
  return BigInt(Math.floor(xlm * Math.pow(10, CONFIG.decimals)));
}

export async function getXlmBalance(address: string): Promise<number> {
  try {
    const account = await horizonServer.loadAccount(address);
    const xlmAsset = account.balances.find((b: { asset_type: string }) => b.asset_type === 'native');
    return xlmAsset ? parseFloat(xlmAsset.balance) : 0;
  } catch {
    return 0;
  }
}

export async function callContractView(
  contractId: string,
  method: string,
  args: xdr.ScVal[] = []
): Promise<xdr.ScVal | null> {
  try {
    const contract = new Contract(contractId);
    const sourceAccount = 'GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF';
    const account = new Account(sourceAccount, '0');

    const tx = new TransactionBuilder(account, {
      fee: '100',
      networkPassphrase: CONFIG.networkPassphrase,
    })
      .addOperation(contract.call(method, ...args))
      .setTimeout(30)
      .build();

    const simResult = await sorobanServer.simulateTransaction(tx);

    if (Api.isSimulationSuccess(simResult) && simResult.result) {
      return simResult.result.retval;
    }
    return null;
  } catch (error) {
    console.error(`View call ${method} error:`, error);
    return null;
  }
}

export async function getExchangeRate(): Promise<number> {
  const result = await callContractView(CONFIG.vault, 'get_exchange_rate', []);
  if (result) {
    const value = scValToNative(result);
    return Number(value) / Math.pow(10, CONFIG.decimals);
  }
  return 1.0;
}

export async function getTotalAssets(): Promise<number> {
  const result = await callContractView(CONFIG.vault, 'get_total_assets', []);
  if (result) {
    const value = scValToNative(result);
    return stroopsToXlm(value);
  }
  return 0;
}

export async function getSxlmBalance(address: string): Promise<number> {
  const result = await callContractView(CONFIG.sxlmToken, 'balance', [
    nativeToScVal(address, { type: 'address' })
  ]);
  if (result) {
    const value = scValToNative(result);
    return stroopsToXlm(value);
  }
  return 0;
}

export async function getTotalSupply(): Promise<number> {
  const result = await callContractView(CONFIG.sxlmToken, 'total_supply', []);
  if (result) {
    const value = scValToNative(result);
    return stroopsToXlm(value);
  }
  return 0;
}

export async function buildDepositTx(
  userAddress: string,
  amount: bigint
): Promise<string> {
  const contract = new Contract(CONFIG.vault);
  const sourceAccount = await sorobanServer.getAccount(userAddress);

  // Build initial transaction
  const tx = new TransactionBuilder(sourceAccount, {
    fee: '10000000',
    networkPassphrase: CONFIG.networkPassphrase,
  })
    .addOperation(
      contract.call(
        'deposit',
        nativeToScVal(userAddress, { type: 'address' }),
        nativeToScVal(amount, { type: 'i128' })
      )
    )
    .setTimeout(300)
    .build();

  // Simulate
  const simResult = await sorobanServer.simulateTransaction(tx);

  if (!Api.isSimulationSuccess(simResult)) {
    throw new Error('Simulation failed: ' + JSON.stringify(simResult));
  }

  // Extract data from simulation
  const simSuccess = simResult as Api.SimulateTransactionSuccessResponse;

  // Get auth and soroban data
  const auth = simSuccess.result?.auth || [];
  const sorobanData = simSuccess.transactionData;

  // Calculate new fee
  const minFee = simSuccess.minResourceFee ? BigInt(simSuccess.minResourceFee) : BigInt(0);
  const newFee = (BigInt(tx.fee) + minFee).toString();

  // Rebuild with new operation that includes auth
  const newAccount = await sorobanServer.getAccount(userAddress);

  const invokeOp = Operation.invokeHostFunction({
    func: (tx.operations[0] as Operation.InvokeHostFunction).func,
    auth: auth
  });

  const finalTx = new TransactionBuilder(newAccount, {
    fee: newFee,
    networkPassphrase: CONFIG.networkPassphrase,
  })
    .addOperation(invokeOp)
    .setTimeout(300);

  if (sorobanData) {
    finalTx.setSorobanData(sorobanData.build());
  }

  return finalTx.build().toXDR();
}

export async function buildWithdrawTx(
  userAddress: string,
  sxlmAmount: bigint
): Promise<string> {
  const contract = new Contract(CONFIG.vault);
  const sourceAccount = await sorobanServer.getAccount(userAddress);

  const tx = new TransactionBuilder(sourceAccount, {
    fee: '10000000',
    networkPassphrase: CONFIG.networkPassphrase,
  })
    .addOperation(
      contract.call(
        'withdraw',
        nativeToScVal(userAddress, { type: 'address' }),
        nativeToScVal(sxlmAmount, { type: 'i128' })
      )
    )
    .setTimeout(300)
    .build();

  const simResult = await sorobanServer.simulateTransaction(tx);

  if (!Api.isSimulationSuccess(simResult)) {
    throw new Error('Simulation failed: ' + JSON.stringify(simResult));
  }

  const simSuccess = simResult as Api.SimulateTransactionSuccessResponse;
  const auth = simSuccess.result?.auth || [];
  const sorobanData = simSuccess.transactionData;
  const minFee = simSuccess.minResourceFee ? BigInt(simSuccess.minResourceFee) : BigInt(0);
  const newFee = (BigInt(tx.fee) + minFee).toString();

  const newAccount = await sorobanServer.getAccount(userAddress);

  const invokeOp = Operation.invokeHostFunction({
    func: (tx.operations[0] as Operation.InvokeHostFunction).func,
    auth: auth
  });

  const finalTx = new TransactionBuilder(newAccount, {
    fee: newFee,
    networkPassphrase: CONFIG.networkPassphrase,
  })
    .addOperation(invokeOp)
    .setTimeout(300);

  if (sorobanData) {
    finalTx.setSorobanData(sorobanData.build());
  }

  return finalTx.build().toXDR();
}

export async function submitSignedTx(signedXdr: string): Promise<{ hash: string; result: unknown }> {
  const signedTx = TransactionBuilder.fromXDR(signedXdr, CONFIG.networkPassphrase);
  const sendResult = await sorobanServer.sendTransaction(signedTx);

  if (sendResult.status === 'ERROR') {
    throw new Error('Submit failed: ' + JSON.stringify(sendResult));
  }

  let attempts = 0;
  while (attempts < 30) {
    const getResult = await sorobanServer.getTransaction(sendResult.hash);

    if (getResult.status === 'SUCCESS') {
      return {
        hash: sendResult.hash,
        result: getResult.returnValue ? scValToNative(getResult.returnValue) : null
      };
    } else if (getResult.status === 'FAILED') {
      throw new Error('Transaction failed on-chain');
    }

    await new Promise(r => setTimeout(r, 1000));
    attempts++;
  }

  throw new Error('Transaction timeout');
}
