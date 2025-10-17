"use client";

import { ReactNode, useMemo } from "react";
import { ConnectionProvider, WalletProvider } from "@solana/wallet-adapter-react";
import { WalletModalProvider } from "@solana/wallet-adapter-react-ui";
import {
    PhantomWalletAdapter,
    SolflareWalletAdapter,
} from "@solana/wallet-adapter-wallets";
import { clusterApiUrl } from "@solana/web3.js";

// Required CSS for default wallet modal styles
import "@solana/wallet-adapter-react-ui/styles.css";

type WalletProvidersProps = {
    children: ReactNode;
};

export function WalletProviders({ children }: WalletProvidersProps) {
    const endpoint = useMemo(() => clusterApiUrl("devnet"), []);

    const wallets = useMemo(() => {
        const initial = [new PhantomWalletAdapter(), new SolflareWalletAdapter()];
        const seen = new Set<string>();
        return initial.filter((adapter) => {
            const name = adapter.name ?? "";
            if (seen.has(name)) return false;
            seen.add(name);
            return true;
        });
    }, []);

    return (
        <ConnectionProvider endpoint={endpoint}>
            <WalletProvider wallets={wallets} autoConnect>
                <WalletModalProvider>{children}</WalletModalProvider>
            </WalletProvider>
        </ConnectionProvider>
    );
}


