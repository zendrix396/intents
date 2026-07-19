"use client";
import { useEffect, useState } from "react";
import { Navbar } from "@/components/Navbar";
import { IntentForm } from "@/components/IntentForm";
import { NetworkStatus } from "@/components/NetworkStatus";
import { useConnection } from "@solana/wallet-adapter-react";
import { useWallet } from "@solana/wallet-adapter-react";
import { LAMPORTS_PER_SOL } from "@solana/web3.js";

export default function Home() {
  const { connection } = useConnection();
  const { publicKey } = useWallet();
  const [balance, setBalance] = useState<number | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    let cancelled = false;
    async function loadBalance() {
      if (!publicKey) {
        setBalance(null);
        return;
      }
      setLoading(true);
      try {
        const lamports = await connection.getBalance(publicKey);
        if (!cancelled) setBalance(lamports / LAMPORTS_PER_SOL);
      } finally {
        if (!cancelled) setLoading(false);
      }
    }
    loadBalance();
    return () => {
      cancelled = true;
    };
  }, [connection, publicKey]);
  return (
    <div>
      <div className="min-h-screen w-full relative bg-black text-white">
        {/* background effect */}
        <div
          className="absolute inset-0 z-0"
          style={{
            background: `
          radial-gradient(ellipse 70% 55% at 50% 50%, rgba(255, 20, 147, 0.15), transparent 50%),
            radial-gradient(ellipse 160% 130% at 10% 10%, rgba(0, 255, 255, 0.12), transparent 60%),
            radial-gradient(ellipse 160% 130% at 90% 90%, rgba(138, 43, 226, 0.18), transparent 65%),
            radial-gradient(ellipse 110% 50% at 80% 30%, rgba(255, 215, 0, 0.08), transparent 40%),
            #000000
          `,
          }}
        />
        {/* Content */}
        <div className="relative z-10">
          <Navbar />
          <div className="flex flex-col items-center gap-6 p-8 rounded-2xl border border-white/10 bg-white/5 backdrop-blur mx-auto mt-24 max-w-lg">
            {publicKey ? (
              <>
                <div className="w-full">
                  <h1 className="text-2xl font-semibold tracking-tight mb-1">Swap</h1>
                  <p className="text-white/50 text-xs mb-4">Trade tokens at the best rate across DEXs</p>
                </div>

                {/* Wallet Info */}
                <div className="w-full flex items-center justify-between bg-white/5 rounded-lg px-4 py-2.5 border border-white/10">
                  <div className="text-white/60 text-xs">
                    <span className="text-white/40">Connected: </span>
                    <span className="font-mono text-white/80">
                      {publicKey.toBase58().slice(0, 4)}...{publicKey.toBase58().slice(-4)}
                    </span>
                  </div>
                  <div className="text-white/60 text-xs">
                    {loading ? "..." : `${balance?.toFixed(4) ?? "0.0000"} SOL`}
                  </div>
                </div>

                {/* Network Status */}
                <NetworkStatus />

                {/* Intent Form */}
                <IntentForm />
              </>
            ) : (
              <>
                <h1 className="text-2xl font-semibold tracking-tight">Connect your wallet</h1>
                <p className="text-white/70 text-sm">Start by connecting a Solana wallet to continue.</p>
              </>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
