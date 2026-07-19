"use client";

import { useState } from "react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { DocMeta } from "../lib/docs";

export function DocsSidebar({ docs }: { docs: DocMeta[] }) {
  const pathname = usePathname();
  const [open, setOpen] = useState(false);

  const navContent = (
    <ul className="space-y-1">
      {docs.map((doc) => {
        const href = `/docs/${doc.slug}`;
        const isActive = pathname === href;

        return (
          <li key={doc.slug}>
            <Link
              href={href}
              onClick={() => setOpen(false)}
              className={`block px-3 py-2 rounded-lg text-sm transition-colors ${
                isActive
                  ? "bg-purple-600/20 text-purple-300 border border-purple-500/30"
                  : "text-white/60 hover:text-white hover:bg-white/5"
              }`}
            >
              {doc.title}
            </Link>
          </li>
        );
      })}
    </ul>
  );

  return (
    <>
      <button
        onClick={() => setOpen(!open)}
        className="md:hidden fixed bottom-4 right-4 z-50 bg-purple-600 hover:bg-purple-500 text-white w-12 h-12 rounded-full shadow-lg flex items-center justify-center transition-colors"
        aria-label="Toggle navigation"
      >
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
          {open ? (
            <path d="M18 6L6 18M6 6l12 12" />
          ) : (
            <path d="M3 12h18M3 6h18M3 18h18" />
          )}
        </svg>
      </button>

      {open && (
        <div
          className="md:hidden fixed inset-0 z-40 bg-black/60"
          onClick={() => setOpen(false)}
        />
      )}

      <nav className={`w-64 shrink-0 border-r border-white/10 bg-white/5 p-6
        max-md:fixed max-md:top-0 max-md:left-0 max-md:bottom-0 max-md:z-40 max-md:bg-black max-md:transition-transform max-md:duration-200
        ${open ? "max-md:translate-x-0" : "max-md:-translate-x-full"}`}>
        <Link href="/docs" className="block mb-6">
          <h2 className="text-lg font-semibold text-white">Documentation</h2>
        </Link>
        {navContent}
      </nav>
    </>
  );
}
