"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { DocMeta } from "../lib/docs";

export function DocsSidebar({ docs }: { docs: DocMeta[] }) {
  const pathname = usePathname();

  return (
    <nav className="w-64 shrink-0 border-r border-white/10 bg-white/5 p-6">
      <Link href="/docs" className="block mb-6">
        <h2 className="text-lg font-semibold text-white">Documentation</h2>
      </Link>
      <ul className="space-y-1">
        {docs.map((doc) => {
          const href = `/docs/${doc.slug}`;
          const isActive = pathname === href;

          return (
            <li key={doc.slug}>
              <Link
                href={href}
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
    </nav>
  );
}
