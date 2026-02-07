'use client';

import { useState, useEffect, useCallback } from 'react';
import dynamic from 'next/dynamic';
import {
  isConnected as freighterIsConnected,
  isAllowed,
  setAllowed,
  getAddress,
  getNetworkDetails,
  signTransaction
} from '@stellar/freighter-api';
import {
  CONFIG,
  getXlmBalance,
  getSxlmBalance,
  getExchangeRate,
  getTotalAssets,
  buildDepositTx,
  buildWithdrawTx,
  submitSignedTx,
  xlmToStroops,
  stroopsToXlm
} from '@/lib/stellar';

const GridMotion = dynamic(() => import('@/components/GridMotion'), { ssr: false });

export default function Home() {
  const [mounted, setMounted] = useState(false);
  const [isConnected, setIsConnected] = useState(false);
  const [userAddress, setUserAddress] = useState<string | null>(null);
  const [xlmBalance, setXlmBalance] = useState(0);
  const [sxlmBalance, setSxlmBalance] = useState(0);
  const [exchangeRate, setExchangeRate] = useState(1.0);
  const [tvl, setTvl] = useState(0);
  const [activeTab, setActiveTab] = useState<'stake' | 'unstake'>('stake');
  const [stakeAmount, setStakeAmount] = useState('');
  const [unstakeAmount, setUnstakeAmount] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [status, setStatus] = useState('');
  const [toast, setToast] = useState<{ message: string; type: 'success' | 'error' } | null>(null);
  const [apy, setApy] = useState(0);

  useEffect(() => setMounted(true), []);

  const showToast = (message: string, type: 'success' | 'error') => {
    setToast({ message, type });
    setTimeout(() => setToast(null), 4000);
  };

  const loadProtocolStats = useCallback(async () => {
    try {
      const [rate, assets] = await Promise.all([getExchangeRate(), getTotalAssets()]);
      setExchangeRate(rate);
      setTvl(assets);

      // Calculate APY based on rate growth
      // Rate started at 1.0, current rate shows total growth
      // For demo: assume growth happened over ~7 days, project to annual
      const totalGrowth = (rate - 1.0) / 1.0; // % growth from 1.0
      const daysElapsed = 7; // assume 7 days for demo
      const dailyRate = totalGrowth / daysElapsed;
      const projectedApy = dailyRate * 365 * 100; // annualized %
      setApy(Math.min(projectedApy, 999)); // cap at 999%
    } catch (e) {
      console.error('Stats error:', e);
    }
  }, []);

  const loadUserBalances = useCallback(async () => {
    if (!userAddress) return;
    try {
      const [xlm, sxlm] = await Promise.all([getXlmBalance(userAddress), getSxlmBalance(userAddress)]);
      setXlmBalance(xlm);
      setSxlmBalance(sxlm);
    } catch (e) {
      console.error('Balance error:', e);
    }
  }, [userAddress]);

  useEffect(() => {
    loadProtocolStats();
    const interval = setInterval(loadProtocolStats, 30000);
    return () => clearInterval(interval);
  }, [loadProtocolStats]);

  useEffect(() => {
    if (userAddress) loadUserBalances();
  }, [userAddress, loadUserBalances]);

  useEffect(() => {
    const check = async () => {
      try {
        if (await freighterIsConnected()) {
          if (await isAllowed()) {
            const { address } = await getAddress();
            const { networkPassphrase } = await getNetworkDetails();
            if (address && networkPassphrase === CONFIG.networkPassphrase) {
              setUserAddress(address);
              setIsConnected(true);
            }
          }
        }
      } catch (e) {
        console.log('Check error:', e);
      }
    };
    check();
  }, []);

  const connectWallet = async () => {
    try {
      setIsLoading(true);
      setStatus('Connecting...');

      if (!(await freighterIsConnected())) {
        showToast('Install Freighter wallet', 'error');
        window.open('https://www.freighter.app/', '_blank');
        return;
      }

      await setAllowed();
      const { networkPassphrase } = await getNetworkDetails();

      if (networkPassphrase !== CONFIG.networkPassphrase) {
        showToast('Switch to Testnet in Freighter', 'error');
        return;
      }

      const { address, error } = await getAddress();
      if (error) throw new Error(error);

      setUserAddress(address);
      setIsConnected(true);
      showToast('Connected', 'success');
    } catch (e) {
      showToast('Connection failed', 'error');
    } finally {
      setIsLoading(false);
      setStatus('');
    }
  };

  const handleStake = async () => {
    if (!userAddress) return;
    const amount = parseFloat(stakeAmount);
    if (!amount || amount < 0.1) return showToast('Min 0.1 XLM', 'error');
    if (amount > xlmBalance - 1) return showToast('Keep 1 XLM for fees', 'error');

    try {
      setIsLoading(true);
      setStatus('Building...');
      console.log('Building deposit tx for', userAddress, xlmToStroops(amount));
      const txXdr = await buildDepositTx(userAddress, xlmToStroops(amount));
      console.log('TX built');

      setStatus('Sign transaction...');
      const signResult = await signTransaction(txXdr, { networkPassphrase: CONFIG.networkPassphrase });
      console.log('Sign result:', signResult);
      if (!signResult.signedTxXdr) throw new Error('Transaction was cancelled or failed');

      setStatus('Submitting...');
      const result = await submitSignedTx(signResult.signedTxXdr);
      console.log('Submit result:', result);
      showToast(`Received ${result.result ? stroopsToXlm(result.result as bigint).toFixed(2) : '?'} sXLM`, 'success');
      setStakeAmount('');
      loadUserBalances();
      loadProtocolStats();
    } catch (e) {
      console.error('Stake error:', e);
      showToast(e instanceof Error ? e.message : 'Stake failed', 'error');
    } finally {
      setIsLoading(false);
      setStatus('');
    }
  };

  const handleUnstake = async () => {
    if (!userAddress) return;
    const amount = parseFloat(unstakeAmount);
    if (!amount || amount < 0.1) return showToast('Min 0.1 sXLM', 'error');
    if (amount > sxlmBalance) return showToast('Insufficient balance', 'error');

    try {
      setIsLoading(true);
      setStatus('Building...');
      console.log('Building withdraw tx for', userAddress, xlmToStroops(amount));
      const txXdr = await buildWithdrawTx(userAddress, xlmToStroops(amount));
      console.log('TX built');

      setStatus('Sign transaction...');
      const signResult = await signTransaction(txXdr, { networkPassphrase: CONFIG.networkPassphrase });
      console.log('Sign result:', signResult);
      if (!signResult.signedTxXdr) throw new Error('Transaction was cancelled or failed');

      setStatus('Submitting...');
      const result = await submitSignedTx(signResult.signedTxXdr);
      console.log('Submit result:', result);
      showToast(`Received ${result.result ? stroopsToXlm(result.result as bigint).toFixed(2) : '?'} XLM`, 'success');
      setUnstakeAmount('');
      loadUserBalances();
      loadProtocolStats();
    } catch (e) {
      console.error('Unstake error:', e);
      showToast(e instanceof Error ? e.message : 'Unstake failed', 'error');
    } finally {
      setIsLoading(false);
      setStatus('');
    }
  };

  const stakePreview = stakeAmount ? (parseFloat(stakeAmount) / exchangeRate).toFixed(4) : '0.00';
  const unstakePreview = unstakeAmount ? (parseFloat(unstakeAmount) * exchangeRate).toFixed(4) : '0.00';

  const gridItems = [
    'Liquid Staking', 'Stellar', 'sXLM', 'Yield', 'DeFi', 'Soroban', 'XLM',
    'Staking', 'Protocol', 'Vault', 'APY', 'TVL', 'Earn', 'Secure',
    'Liquid Staking', 'Stellar', 'sXLM', 'Yield', 'DeFi', 'Soroban', 'XLM',
    'Staking', 'Protocol', 'Vault', 'APY', 'TVL', 'Earn', 'Secure',
  ];

  if (!mounted) return null;

  return (
    <div className="min-h-screen bg-black text-white">
      {/* Background */}
      <div className="fixed inset-0">
        <GridMotion items={gridItems} gradientColor="black" />
      </div>
      <div className="fixed inset-0 bg-black/70" />

      {/* Content */}
      <div className="relative z-10">

        {/* Navbar */}
        <nav className="border-b border-white/10">
          <div style={{ maxWidth: 1000, margin: '0 auto', padding: '0 24px', height: 64, display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: 32 }}>
              <span style={{ fontSize: 20, fontWeight: 700 }}>sXLM</span>
              <div style={{ display: 'flex', alignItems: 'center', gap: 24 }}>
                <a href="#" style={{ fontSize: 14, color: 'white' }}>Stake</a>
                <a href={`https://stellar.expert/explorer/testnet/contract/${CONFIG.vault}`} target="_blank" style={{ fontSize: 14, color: 'rgba(255,255,255,0.5)' }}>Explorer</a>
              </div>
            </div>
            <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
              <span style={{ fontSize: 12, color: '#f97316', backgroundColor: 'rgba(249,115,22,0.1)', padding: '4px 8px', borderRadius: 4 }}>Testnet</span>
              <button
                onClick={connectWallet}
                style={{ backgroundColor: 'white', color: 'black', padding: '8px 16px', borderRadius: 8, fontSize: 14, fontWeight: 500, border: 'none', cursor: 'pointer' }}
              >
                {isLoading ? status : isConnected ? `${userAddress?.slice(0,4)}...${userAddress?.slice(-4)}` : 'Connect'}
              </button>
            </div>
          </div>
        </nav>

        {/* Main */}
        <div style={{ maxWidth: 420, margin: '0 auto', padding: '80px 16px' }}>

          {/* Stats */}
          <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: 40 }}>
            <div style={{ textAlign: 'center' }}>
              <div style={{ fontSize: 24, fontWeight: 700 }}>{tvl.toLocaleString()}</div>
              <div style={{ fontSize: 14, color: 'rgba(255,255,255,0.5)', marginTop: 4 }}>TVL</div>
            </div>
            <div style={{ textAlign: 'center' }}>
              <div style={{ fontSize: 24, fontWeight: 700, color: '#4ade80' }}>{apy > 0 ? `${apy.toFixed(1)}%` : '~10%'}</div>
              <div style={{ fontSize: 14, color: 'rgba(255,255,255,0.5)', marginTop: 4 }}>APY</div>
            </div>
            <div style={{ textAlign: 'center' }}>
              <div style={{ fontSize: 24, fontWeight: 700 }}>{exchangeRate.toFixed(4)}</div>
              <div style={{ fontSize: 14, color: 'rgba(255,255,255,0.5)', marginTop: 4 }}>Rate</div>
            </div>
          </div>

          {/* Card */}
          <div style={{ backgroundColor: '#171717', borderRadius: 16, padding: 24 }}>

            {/* Tabs */}
            <div style={{ display: 'flex', gap: 8, marginBottom: 24 }}>
              <button
                onClick={() => setActiveTab('stake')}
                style={{
                  flex: 1,
                  padding: '12px 0',
                  borderRadius: 8,
                  fontSize: 14,
                  fontWeight: 600,
                  border: 'none',
                  cursor: 'pointer',
                  backgroundColor: activeTab === 'stake' ? 'white' : 'transparent',
                  color: activeTab === 'stake' ? 'black' : 'rgba(255,255,255,0.5)'
                }}
              >
                Stake
              </button>
              <button
                onClick={() => setActiveTab('unstake')}
                style={{
                  flex: 1,
                  padding: '12px 0',
                  borderRadius: 8,
                  fontSize: 14,
                  fontWeight: 600,
                  border: 'none',
                  cursor: 'pointer',
                  backgroundColor: activeTab === 'unstake' ? 'white' : 'transparent',
                  color: activeTab === 'unstake' ? 'black' : 'rgba(255,255,255,0.5)'
                }}
              >
                Unstake
              </button>
            </div>

            {activeTab === 'stake' ? (
              <>
                {/* Input */}
                <div style={{ marginBottom: 16 }}>
                  <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: 14, color: 'rgba(255,255,255,0.5)', marginBottom: 8 }}>
                    <span>You stake</span>
                    <span>Balance: {xlmBalance.toFixed(2)} XLM</span>
                  </div>
                  <div style={{ backgroundColor: '#262626', borderRadius: 12, padding: 16, display: 'flex', alignItems: 'center', gap: 12 }}>
                    <input
                      type="number"
                      value={stakeAmount}
                      onChange={(e) => setStakeAmount(e.target.value)}
                      placeholder="0.00"
                      style={{ flex: 1, backgroundColor: 'transparent', border: 'none', outline: 'none', fontSize: 24, fontWeight: 500, color: 'white' }}
                    />
                    <button
                      onClick={() => setStakeAmount(Math.max(0, xlmBalance - 1).toFixed(2))}
                      style={{ fontSize: 12, color: '#a78bfa', fontWeight: 600, background: 'none', border: 'none', cursor: 'pointer' }}
                    >
                      MAX
                    </button>
                    <span style={{ fontSize: 14, color: 'rgba(255,255,255,0.5)' }}>XLM</span>
                  </div>
                </div>

                {/* Output */}
                <div style={{ marginBottom: 24 }}>
                  <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: 14, color: 'rgba(255,255,255,0.5)', marginBottom: 8 }}>
                    <span>You receive</span>
                    <span>Balance: {sxlmBalance.toFixed(2)} sXLM</span>
                  </div>
                  <div style={{ backgroundColor: '#262626', borderRadius: 12, padding: 16, display: 'flex', alignItems: 'center', gap: 12 }}>
                    <span style={{ flex: 1, fontSize: 24, fontWeight: 500 }}>{stakePreview}</span>
                    <span style={{ fontSize: 14, color: 'rgba(255,255,255,0.5)' }}>sXLM</span>
                  </div>
                </div>

                {/* Rate */}
                <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: 14, color: 'rgba(255,255,255,0.5)', marginBottom: 24 }}>
                  <span>Rate</span>
                  <span>1 XLM = {(1/exchangeRate).toFixed(4)} sXLM</span>
                </div>

                {/* Button */}
                <button
                  onClick={isConnected ? handleStake : connectWallet}
                  disabled={isLoading || (isConnected && !stakeAmount)}
                  style={{
                    width: '100%',
                    padding: '16px 0',
                    borderRadius: 12,
                    fontSize: 16,
                    fontWeight: 600,
                    border: 'none',
                    cursor: 'pointer',
                    backgroundColor: '#7c3aed',
                    color: 'white',
                    opacity: (isLoading || (isConnected && !stakeAmount)) ? 0.5 : 1
                  }}
                >
                  {isLoading ? status : isConnected ? 'Stake' : 'Connect Wallet'}
                </button>
              </>
            ) : (
              <>
                {/* Input */}
                <div style={{ marginBottom: 16 }}>
                  <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: 14, color: 'rgba(255,255,255,0.5)', marginBottom: 8 }}>
                    <span>You unstake</span>
                    <span>Balance: {sxlmBalance.toFixed(2)} sXLM</span>
                  </div>
                  <div style={{ backgroundColor: '#262626', borderRadius: 12, padding: 16, display: 'flex', alignItems: 'center', gap: 12 }}>
                    <input
                      type="number"
                      value={unstakeAmount}
                      onChange={(e) => setUnstakeAmount(e.target.value)}
                      placeholder="0.00"
                      style={{ flex: 1, backgroundColor: 'transparent', border: 'none', outline: 'none', fontSize: 24, fontWeight: 500, color: 'white' }}
                    />
                    <button
                      onClick={() => setUnstakeAmount(sxlmBalance.toFixed(2))}
                      style={{ fontSize: 12, color: '#a78bfa', fontWeight: 600, background: 'none', border: 'none', cursor: 'pointer' }}
                    >
                      MAX
                    </button>
                    <span style={{ fontSize: 14, color: 'rgba(255,255,255,0.5)' }}>sXLM</span>
                  </div>
                </div>

                {/* Output */}
                <div style={{ marginBottom: 24 }}>
                  <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: 14, color: 'rgba(255,255,255,0.5)', marginBottom: 8 }}>
                    <span>You receive</span>
                    <span>Balance: {xlmBalance.toFixed(2)} XLM</span>
                  </div>
                  <div style={{ backgroundColor: '#262626', borderRadius: 12, padding: 16, display: 'flex', alignItems: 'center', gap: 12 }}>
                    <span style={{ flex: 1, fontSize: 24, fontWeight: 500 }}>{unstakePreview}</span>
                    <span style={{ fontSize: 14, color: 'rgba(255,255,255,0.5)' }}>XLM</span>
                  </div>
                </div>

                {/* Rate */}
                <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: 14, color: 'rgba(255,255,255,0.5)', marginBottom: 24 }}>
                  <span>Rate</span>
                  <span>1 sXLM = {exchangeRate.toFixed(4)} XLM</span>
                </div>

                {/* Button */}
                <button
                  onClick={isConnected ? handleUnstake : connectWallet}
                  disabled={isLoading || (isConnected && !unstakeAmount)}
                  style={{
                    width: '100%',
                    padding: '16px 0',
                    borderRadius: 12,
                    fontSize: 16,
                    fontWeight: 600,
                    border: 'none',
                    cursor: 'pointer',
                    backgroundColor: '#7c3aed',
                    color: 'white',
                    opacity: (isLoading || (isConnected && !unstakeAmount)) ? 0.5 : 1
                  }}
                >
                  {isLoading ? status : isConnected ? 'Unstake' : 'Connect Wallet'}
                </button>
              </>
            )}
          </div>
        </div>
      </div>

      {/* Toast */}
      {toast && (
        <div style={{
          position: 'fixed',
          bottom: 24,
          left: '50%',
          transform: 'translateX(-50%)',
          zIndex: 50,
          padding: '12px 20px',
          borderRadius: 8,
          fontSize: 14,
          fontWeight: 500,
          backgroundColor: toast.type === 'success' ? '#22c55e' : '#ef4444',
          color: 'white'
        }}>
          {toast.message}
        </div>
      )}
    </div>
  );
}
