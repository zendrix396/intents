"use client";

import { useEffect } from "react";
import { useRouter } from "next/navigation";

/**
 * Custom hook for intelligent prefetching of documentation pages
 * This prefetches all documentation pages when the user is on any docs page
 */
export function usePrefetch() {
  const router = useRouter();

  useEffect(() => {
    // Prefetch all documentation pages for instant navigation
    const prefetchPages = [
      "/docs",
      "/docs/about",
      "/docs/architecture",
      "/docs/instructions",
    ];

    // Prefetch each page after a short delay to avoid blocking initial render
    const timeoutId = setTimeout(() => {
      prefetchPages.forEach((page) => {
        router.prefetch(page);
      });
    }, 100);

    return () => clearTimeout(timeoutId);
  }, [router]);
}

/**
 * Hook for prefetching related pages based on current page
 */
export function usePrefetchRelated(currentPage: string) {
  const router = useRouter();

  useEffect(() => {
    const relatedPages: Record<string, string[]> = {
      "/docs": ["/docs/about", "/docs/architecture", "/docs/instructions"],
      "/docs/about": ["/docs/architecture", "/docs"],
      "/docs/architecture": ["/docs/about", "/docs/instructions", "/docs"],
      "/docs/instructions": ["/docs/architecture", "/docs"],
    };

    const pagesToPrefetch = relatedPages[currentPage] || [];

    // Prefetch related pages after a short delay
    const timeoutId = setTimeout(() => {
      pagesToPrefetch.forEach((page) => {
        router.prefetch(page);
      });
    }, 200);

    return () => clearTimeout(timeoutId);
  }, [currentPage, router]);
}
