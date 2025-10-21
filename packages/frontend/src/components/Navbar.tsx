"use client";

import Link from "next/link";
import dynamic from "next/dynamic";

const WalletMultiButton = dynamic(
    async () => (await import("@solana/wallet-adapter-react-ui")).WalletMultiButton,
    { ssr: false }
);

export function Navbar() {
    return (
        <nav className="w-full px-6 py-4 flex items-center justify-between">
            <Link href="/" prefetch={true} className="text-white/90 hover:text-white transition-colors">
                <span className="text-lg font-semibold tracking-tight">Intent</span>
            </Link>
            <div className="flex items-center space-x-6">
                <Link
                    href="/docs"
                    prefetch={true}
                    className="text-white/70 hover:text-white transition-colors text-sm font-medium"
                >
                    Docs
                </Link>
                <WalletMultiButton className="!rounded-xl !px-4 !py-2 !text-sm !bg-white !text-black hover:!bg-white/90" />
            </div>
        </nav>
    );
}
