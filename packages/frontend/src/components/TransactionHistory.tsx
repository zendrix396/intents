"use client";

import { useEffect, useState } from "react";

interface SwapRecord {
  id: string;
  inputSymbol: string;
  outputSymbol: string;
  inputAmount: string;
  outputAmount: string;
  signature: string;
  timestamp: number;
}

const STORAGE_KEY = "intent-swap-history";
const MAX_RECORDS = 20;

function loadHistory(): SwapRecord[] {
  if (typeof window === "undefined") return [];
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    return raw ? JSON.parse(raw) : [];
  } catch {
    return [];
  }
}

function saveHistory(records: SwapRecord[]) {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(records));
  } catch {}
}

export function addSwapToHistory(record: Omit<SwapRecord, "id" | "timestamp">) {
  const entry: SwapRecord = {
    ...record,
    id: crypto.randomUUID(),
    timestamp: Date.now(),
  };
  const existing = loadHistory();
  const updated = [entry, ...existing].slice(0, MAX_RECORDS);
  saveHistory(updated);
}

function formatTimeAgo(ts: number): string {
  const seconds = Math.floor((Date.now() - ts) / 1000);
  if (seconds < 60) return "just now";
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return `${minutes}m ago`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}h ago`;
  const days = Math.floor(hours / 24);
  return `${days}d ago`;
}

export function TransactionHistory() {
  const [history, setHistory] = useState<SwapRecord[]>([]);

  useEffect(() => {
    setHistory(loadHistory());
  }, []);

  if (history.length === 0) return null;

  return (
    <div className="w-full space-y-2">
      <div className="flex items-center justify-between">
        <h3 className="text-white/60 text-xs font-medium uppercase tracking-wider">
          Recent Swaps
        </h3>
        <button
          onClick={() => {
            localStorage.removeItem(STORAGE_KEY);
            setHistory([]);
          }}
          className="text-white/30 hover:text-white/60 text-xs transition-colors"
        >
          Clear
        </button>
      </div>
      <div className="space-y-1">
        {history.map((record) => (
          <div
            key={record.id}
            className="flex items-center justify-between bg-white/5 rounded-lg px-3 py-2 border border-white/5"
          >
            <div className="flex flex-col gap-0.5">
              <span className="text-white/80 text-xs">
                {record.inputAmount} {record.inputSymbol} → {record.outputAmount}{" "}
                {record.outputSymbol}
              </span>
              <span className="text-white/30 text-[10px]">
                {formatTimeAgo(record.timestamp)}
              </span>
            </div>
            <a
              href={`https://explorer.solana.com/tx/${record.signature}?cluster=devnet`}
              target="_blank"
              rel="noopener noreferrer"
              className="text-purple-400 hover:text-purple-300 text-[10px] transition-colors shrink-0 ml-3"
            >
              View ↗
            </a>
          </div>
        ))}
      </div>
    </div>
  );
}
