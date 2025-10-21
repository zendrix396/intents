"use client"
import { Navbar } from '@/components/Navbar';
import Link from 'next/link';
import { docsNav } from '@/lib/docs-nav';
import { usePrefetch } from '@/lib/usePrefetch';

export default function DocsIndexPage() {
    // Prefetch all documentation pages for instant navigation
    usePrefetch();

    return (
        <div className="min-h-screen bg-gradient-to-br from-gray-900 via-black to-gray-900 text-white">
            <Navbar />

            <div className="max-w-6xl mx-auto px-6 py-12">
                {/* Hero Section */}
                <div className="text-center mb-16">
                    <h1 className="text-5xl md:text-6xl font-bold bg-gradient-to-r from-white via-gray-200 to-gray-400 bg-clip-text text-transparent mb-6">
                        Documentation
                    </h1>
                    <p className="text-xl text-gray-400 max-w-2xl mx-auto leading-relaxed">
                        Explore our comprehensive guides and learn how to build robust Solana applications with our intent-centric layer.
                    </p>
                </div>

                {/* Documentation Cards */}
                <div className="grid gap-8 md:grid-cols-2 lg:grid-cols-3">
                    {docsNav.map((doc) => (
                        <Link
                            key={doc.slug}
                            href={`/docs/${doc.slug}`}
                            prefetch={true}
                            className="group relative overflow-hidden rounded-2xl bg-gradient-to-br from-gray-800/50 to-gray-900/50 backdrop-blur-sm border border-gray-700/50 hover:border-gray-600/50 transition-all duration-300 hover:scale-[1.02] hover:shadow-2xl hover:shadow-purple-500/10"
                        >
                            {/* Gradient overlay */}
                            <div className="absolute inset-0 bg-gradient-to-br from-purple-500/5 to-blue-500/5 opacity-0 group-hover:opacity-100 transition-opacity duration-300" />

                            {/* Content */}
                            <div className="relative p-8">
                                {/* Icon placeholder - you can replace with actual icons */}
                                <div className="w-12 h-12 rounded-xl bg-gradient-to-br from-purple-500 to-blue-500 flex items-center justify-center mb-6 group-hover:scale-110 transition-transform duration-300">
                                    <span className="text-white font-bold text-lg">
                                        {doc.title.charAt(0)}
                                    </span>
                                </div>

                                <h2 className="text-2xl font-bold mb-4 group-hover:text-white transition-colors duration-300">
                                    {doc.title}
                                </h2>

                                <p className="text-gray-400 group-hover:text-gray-300 transition-colors duration-300 leading-relaxed">
                                    {doc.description}
                                </p>

                                {/* Arrow indicator */}
                                <div className="mt-6 flex items-center text-purple-400 group-hover:text-purple-300 transition-colors duration-300">
                                    <span className="text-sm font-medium">Learn more</span>
                                    <svg
                                        className="ml-2 w-4 h-4 transform group-hover:translate-x-1 transition-transform duration-300"
                                        fill="none"
                                        stroke="currentColor"
                                        viewBox="0 0 24 24"
                                    >
                                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
                                    </svg>
                                </div>
                            </div>
                        </Link>
                    ))}
                </div>

                {/* Additional Info Section */}
                <div className="mt-20 text-center">
                    <div className="inline-flex items-center px-6 py-3 rounded-full bg-gray-800/50 border border-gray-700/50 backdrop-blur-sm">
                        <div className="w-2 h-2 rounded-full bg-green-400 mr-3 animate-pulse" />
                        <span className="text-gray-300 text-sm font-medium">
                            Documentation is actively maintained and updated
                        </span>
                    </div>
                </div>
            </div>
        </div>
    );
}
