"use client";

import { usePrefetchRelated } from "@/lib/usePrefetch";

interface ClientPageWrapperProps {
    children: React.ReactNode;
    currentPage: string;
}

export function ClientPageWrapper({ children, currentPage }: ClientPageWrapperProps) {
    // Handle prefetching on the client side
    usePrefetchRelated(currentPage);

    return <>{children}</>;
}
