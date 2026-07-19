import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Documentation | Solana Intents",
  description: "Documentation for the Solana Intents platform",
};

export default function DocsLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return children;
}
