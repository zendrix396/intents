"use client";

import { useState } from "react";
import { useWallet } from "@solana/wallet-adapter-react";
import { getQuote, executeSwap, SwapIntent, JupiterOrderResponse, ExecuteResponse } from "@/app/lib/api";

const TOKEN_OPTIONS = [
  { mint: "So11111111111111111111111111111111111111112", symbol: "SOL", decimals: 9 },
  { mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", symbol: "USDC", decimals: 6 },
  { mint: "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB", symbol: "USDT", decimals: 6 },
  { mint: "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So", symbol: "mSOL", decimals: 9 },
  { mint: "7vfCXTUXx5WJV5JADk17DUJ4ksgau7utNKj4b963voxs", symbol: "wSOL", decimals: 9 },
];

const SLIPPAGE_OPTIONS = [
  { value: 10, label: "0.1%" },
  { value: 50, label: "0.5%" },
  { value: 100, label: "1%" },
  { value: 200, label: "2%" },
];

interface IntentFormProps {
  onQuoteReceived?: (quote: JupiterOrderResponse, intent: SwapIntent) => void;
  onSwapExecuted?: (signature: string) => void;
}

export function IntentForm({ onQuoteReceived, onSwapExecuted }: IntentFormProps) {
  const { publicKey } = useWallet();
  const [inputToken, setInputToken] = useState(TOKEN_OPTIONS[0]);
  const [outputToken, setOutputToken] = useState(TOKEN_OPTIONS[1]);
  const [amount, setAmount] = useState("");
  const [slippageBps, setSlippageBps] = useState(50);
  const [quote, setQuote] = useState<JupiterOrderResponse | null>(null);
  const [loading, setLoading] = useState(false);
  const [executing, setExecuting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [txSignature, setTxSignature] = useState<string | null>(null);
  const [execDetails, setExecDetails] = useState<ExecuteResponse | null>(null);

  const handleGetQuote = async () => {
    if (!publicKey || !amount || parseFloat(amount) <= 0) return;

    setError(null);
    setQuote(null);
    setTxSignature(null);
    setLoading(true);

    try {
      const rawAmount = Math.floor(parseFloat(amount) * Math.pow(10, inputToken.decimals));
      const intent: SwapIntent = {
        inputMint: inputToken.mint,
        outputMint: outputToken.mint,
        amount: rawAmount,
        taker: publicKey.toBase58(),
        slippageBps,
      };

      const result = await getQuote(intent);
      setQuote(result);
      onQuoteReceived?.(result, intent);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to get quote");
    } finally {
      setLoading(false);
    }
  };

  const handleExecute = async () => {
    if (!publicKey || !quote || !amount) return;

    setExecuting(true);
    setError(null);

    try {
      const rawAmount = Math.floor(parseFloat(amount) * Math.pow(10, inputToken.decimals));
      const intent: SwapIntent = {
        inputMint: inputToken.mint,
        outputMint: outputToken.mint,
        amount: rawAmount,
        taker: publicKey.toBase58(),
        slippageBps,
      };

      const result = await executeSwap(intent);
      setTxSignature(result.signature);
      setExecDetails(result);
      onSwapExecuted?.(result.signature);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to execute swap");
    } finally {
      setExecuting(false);
    }
  };

  const swapTokens = () => {
    setInputToken(outputToken);
    setOutputToken(inputToken);
    setQuote(null);
    setTxSignature(null);
  };

  const formatAmount = (raw: string, decimals: number) => {
    const num = parseFloat(raw) / Math.pow(10, decimals);
    return num.toFixed(decimals > 6 ? 6 : decimals);
  };

  return (
    <div className="space-y-4">
      {/* Input Token */}
      <div>
        <label className="block text-white/60 text-xs mb-1.5">You Pay</label>
        <div className="flex gap-2">
          <select
            value={inputToken.mint}
            onChange={(e) => {
              const tok = TOKEN_OPTIONS.find((t) => t.mint === e.target.value);
              if (tok) setInputToken(tok);
              setQuote(null);
            }}
            className="bg-white/5 border border-white/10 rounded-lg px-3 py-2.5 text-white text-sm focus:outline-none focus:border-purple-500/50"
          >
            {TOKEN_OPTIONS.map((t) => (
              <option key={t.mint} value={t.mint} className="bg-gray-900">
                {t.symbol}
              </option>
            ))}
          </select>
          <input
            type="number"
            value={amount}
            onChange={(e) => {
              setAmount(e.target.value);
              setQuote(null);
            }}
            placeholder="0.0"
            min="0"
            step="any"
            className="flex-1 bg-white/5 border border-white/10 rounded-lg px-3 py-2.5 text-white text-sm placeholder:text-white/30 focus:outline-none focus:border-purple-500/50 [appearance:textfield] [&::-webkit-outer-spin-button]:appearance-none [&::-webkit-inner-spin-button]:appearance-none"
          />
        </div>
      </div>

      {/* Swap Button */}
      <div className="flex justify-center">
        <button
          onClick={swapTokens}
          className="w-8 h-8 rounded-full bg-white/10 hover:bg-white/20 flex items-center justify-center transition-colors"
          aria-label="Swap tokens"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" className="text-white/70">
            <path d="M7 16V4m0 0L3 8m4-4l4 4M17 8v12m0 0l4-4m-4 4l-4-4" />
          </svg>
        </button>
      </div>

      {/* Output Token */}
      <div>
        <label className="block text-white/60 text-xs mb-1.5">You Receive</label>
        <div className="flex gap-2">
          <select
            value={outputToken.mint}
            onChange={(e) => {
              const tok = TOKEN_OPTIONS.find((t) => t.mint === e.target.value);
              if (tok) setOutputToken(tok);
              setQuote(null);
            }}
            className="bg-white/5 border border-white/10 rounded-lg px-3 py-2.5 text-white text-sm focus:outline-none focus:border-purple-500/50"
          >
            {TOKEN_OPTIONS.map((t) => (
              <option key={t.mint} value={t.mint} className="bg-gray-900">
                {t.symbol}
              </option>
            ))}
          </select>
          <div className="flex-1 bg-white/5 border border-white/10 rounded-lg px-3 py-2.5 text-white/50 text-sm">
            {quote && !quote.errorMessage
              ? formatAmount(quote.outAmount, outputToken.decimals)
              : "—"}
          </div>
        </div>
      </div>

      {/* Slippage Tolerance */}
      <div>
        <label className="block text-white/60 text-xs mb-1.5">Slippage Tolerance</label>
        <div className="flex gap-1.5">
          {SLIPPAGE_OPTIONS.map((opt) => (
            <button
              key={opt.value}
              onClick={() => setSlippageBps(opt.value)}
              className={`flex-1 px-2 py-1.5 rounded-md text-xs font-medium transition-colors ${
                slippageBps === opt.value
                  ? "bg-purple-600 text-white"
                  : "bg-white/5 text-white/50 hover:bg-white/10 hover:text-white/70"
              }`}
            >
              {opt.label}
            </button>
          ))}
        </div>
      </div>

      {/* Error */}
      {error && (
        <div className="bg-red-500/10 border border-red-500/20 rounded-lg px-3 py-2 text-red-400 text-xs">
          {error}
        </div>
      )}

      {/* Quote Info */}
      {quote && !quote.errorMessage && (
        <div className="bg-white/5 border border-white/10 rounded-lg px-3 py-2 space-y-1">
          <div className="flex justify-between text-xs">
            <span className="text-white/50">Rate</span>
            <span className="text-white/80">
              1 {inputToken.symbol} ≈{" "}
              {(
                parseFloat(quote.outAmount) /
                Math.pow(10, outputToken.decimals) /
                (parseFloat(quote.inAmount) / Math.pow(10, inputToken.decimals))
              ).toFixed(6)}{" "}
              {outputToken.symbol}
            </span>
          </div>
          <div className="flex justify-between text-xs">
            <span className="text-white/50">Min Received</span>
            <span className="text-green-400">
              {formatAmount(
                String(Math.floor(parseFloat(quote.outAmount) * (1 - slippageBps / 10000))),
                outputToken.decimals
              )}{" "}
              {outputToken.symbol}
            </span>
          </div>
          <div className="flex justify-between text-xs">
            <span className="text-white/50">Slippage</span>
            <span className="text-white/70">{slippageBps / 100}%</span>
          </div>
        </div>
      )}

      {/* Actions */}
      <div className="flex gap-2">
        <button
          onClick={handleGetQuote}
          disabled={!publicKey || !amount || loading || executing}
          className="flex-1 bg-purple-600 hover:bg-purple-500 disabled:bg-white/10 disabled:text-white/30 text-white font-medium py-2.5 rounded-lg text-sm transition-colors"
        >
          {loading ? "Getting Quote..." : "Get Quote"}
        </button>
        {quote && !quote.errorMessage && (
          <button
            onClick={handleExecute}
            disabled={executing}
            className="flex-1 bg-green-600 hover:bg-green-500 disabled:bg-white/10 disabled:text-white/30 text-white font-medium py-2.5 rounded-lg text-sm transition-colors"
          >
            {executing ? "Executing..." : "Execute Swap"}
          </button>
        )}
      </div>

      {/* Success */}
      {txSignature && (
        <div className="bg-green-500/10 border border-green-500/20 rounded-lg px-3 py-2 text-green-400 text-xs space-y-1">
          <div className="font-medium">Swap executed successfully!</div>
          {execDetails && (
            <div className="flex flex-col gap-0.5 text-green-300/80">
              <div className="flex justify-between">
                <span>Priority Fee</span>
                <span>{execDetails.priorityFee.toLocaleString()} {execDetails.priorityFeeUnit.replace("micro_lamports_per_cu", "μLam/CU")}</span>
              </div>
              <div className="flex justify-between">
                <span>Execution Time</span>
                <span>{execDetails.executionTimeMs.toLocaleString()}ms</span>
              </div>
              {execDetails.unitsConsumed != null && (
                <div className="flex justify-between">
                  <span>Compute Units</span>
                  <span>{execDetails.unitsConsumed.toLocaleString()}</span>
                </div>
              )}
              <div className="flex justify-between">
                <span>Received</span>
                <span>{(parseFloat(execDetails.outAmount) / Math.pow(10, outputToken.decimals)).toFixed(outputToken.decimals > 6 ? 6 : outputToken.decimals)} {outputToken.symbol}</span>
              </div>
            </div>
          )}
          <a
            href={`https://explorer.solana.com/tx/${txSignature}?cluster=devnet`}
            target="_blank"
            rel="noopener noreferrer"
            className="underline text-green-300 break-all"
          >
            View on Explorer
          </a>
        </div>
      )}
    </div>
  );
}
