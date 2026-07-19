import Link from "next/link";

export function Footer() {
  return (
    <footer className="border-t border-white/10 bg-black/50 mt-auto">
      <div className="max-w-5xl mx-auto px-6 py-8 flex flex-col md:flex-row items-center justify-between gap-4 text-xs text-white/40">
        <div className="flex items-center gap-2">
          <span className="font-semibold text-white/60">Intent</span>
          <span>&middot;</span>
          <span>Intent-centric execution for Solana</span>
        </div>
        <div className="flex items-center gap-4">
          <Link href="/docs" className="hover:text-white/70 transition-colors">
            Docs
          </Link>
          <a
            href="https://github.com/zendrix396/intents"
            target="_blank"
            rel="noopener noreferrer"
            className="hover:text-white/70 transition-colors"
          >
            GitHub
          </a>
          <a
            href="https://solana.com"
            target="_blank"
            rel="noopener noreferrer"
            className="hover:text-white/70 transition-colors"
          >
            Solana
          </a>
        </div>
      </div>
    </footer>
  );
}
