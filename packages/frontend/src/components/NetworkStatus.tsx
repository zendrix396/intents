"use client";

import { useEffect, useState } from "react";
import { getHealth, getFees, HealthResponse, FeesResponse } from "@/app/lib/api";

export function NetworkStatus() {
  const [health, setHealth] = useState<HealthResponse | null>(null);
  const [fees, setFees] = useState<FeesResponse | null>(null);
  const [error, setError] = useState(false);

  useEffect(() => {
    let cancelled = false;
    async function load() {
      try {
        const [h, f] = await Promise.all([getHealth(), getFees()]);
        if (!cancelled) {
          setHealth(h);
          setFees(f);
          setError(false);
        }
      } catch {
        if (!cancelled) setError(true);
      }
    }
    load();
    const interval = setInterval(load, 30000);
    return () => {
      cancelled = true;
      clearInterval(interval);
    };
  }, []);

  if (error) return null;

  const healthyCount = health?.rpcEndpoints.filter((e) => e.healthy).length ?? 0;
  const totalCount = health?.rpcEndpoints.length ?? 0;

  return (
    <div className="w-full flex items-center justify-between bg-white/5 rounded-lg px-4 py-2 border border-white/10 text-xs">
      <div className="flex items-center gap-3">
        <div className="flex items-center gap-1.5">
          <div
            className={`w-1.5 h-1.5 rounded-full ${
              healthyCount === totalCount && totalCount > 0 ? "bg-green-400" : "bg-yellow-400"
            }`}
          />
          <span className="text-white/50">RPC</span>
          <span className="text-white/70">
            {healthyCount}/{totalCount}
          </span>
        </div>
        {fees && (
          <div className="flex items-center gap-1.5">
            <span className="text-white/50">Priority</span>
            <span className="text-white/70">{fees.priorityFees.low.toLocaleString()} μLam</span>
          </div>
        )}
      </div>
      <div className="text-white/40">Devnet</div>
    </div>
  );
}
