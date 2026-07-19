"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import dynamic from "next/dynamic";

const WalletMultiButton = dynamic(
    async () => (await import("@solana/wallet-adapter-react-ui")).WalletMultiButton,
    { ssr: false }
);

export function Navbar() {
    const pathname = usePathname();
    const isDocs = pathname.startsWith("/docs");

    return (
        <nav className="w-full px-6 py-4 flex items-center justify-between">
            <div className="flex items-center gap-6">
                <Link href="/" className={`hover:text-white transition-colors ${!isDocs ? "text-white/90" : "text-white/50"}`}>
                    <span className="text-lg font-semibold tracking-tight">Intent</span>
                </Link>
                <Link href="/docs" className={`hover:text-white transition-colors text-sm ${isDocs ? "text-white/90" : "text-white/50"}`}>
                    Docs
                </Link>
            </div>
            <WalletMultiButton className="!rounded-xl !px-4 !py-2 !text-sm !bg-white !text-black hover:!bg-white/90" />
        </nav>
    );
}


