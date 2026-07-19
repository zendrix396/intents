const SOLVER_API_URL = process.env.NEXT_PUBLIC_SOLVER_API_URL || "http://localhost:3000";

export interface SwapIntent {
  inputMint: string;
  outputMint: string;
  amount: number;
  taker: string;
}

export interface JupiterOrderResponse {
  inAmount: string;
  outAmount: string;
  transaction?: string;
  errorMessage?: string;
}

export interface HealthResponse {
  status: string;
  payerWallet: string;
  rpcEndpoints: Array<{ endpoint: string; healthy: boolean; latency_ms?: number }>;
  priorityFees: {
    low: number;
    medium: number;
    high: number;
  };
}

export async function getQuote(intent: SwapIntent): Promise<JupiterOrderResponse> {
  const res = await fetch(`${SOLVER_API_URL}/solve`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(intent),
  });

  if (!res.ok) {
    const errText = await res.text();
    throw new Error(`Failed to get quote: ${errText}`);
  }

  return res.json();
}

export async function executeSwap(intent: SwapIntent): Promise<{ signature: string }> {
  const res = await fetch(`${SOLVER_API_URL}/execute`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(intent),
  });

  if (!res.ok) {
    const errText = await res.text();
    throw new Error(`Failed to execute swap: ${errText}`);
  }

  return res.json();
}

export async function getHealth(): Promise<HealthResponse> {
  const res = await fetch(`${SOLVER_API_URL}/health`);

  if (!res.ok) {
    throw new Error("Failed to fetch health status");
  }

  return res.json();
}
