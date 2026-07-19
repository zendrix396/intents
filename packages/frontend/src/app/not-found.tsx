import Link from "next/link";

export default function NotFound() {
  return (
    <div className="min-h-screen bg-black text-white flex items-center justify-center">
      <div className="text-center space-y-4">
        <h1 className="text-6xl font-semibold tracking-tight text-white/20">404</h1>
        <p className="text-white/50 text-lg">This page could not be found.</p>
        <Link
          href="/"
          className="inline-block mt-4 px-5 py-2.5 bg-purple-600 hover:bg-purple-500 text-white text-sm font-medium rounded-lg transition-colors"
        >
          Back to Home
        </Link>
      </div>
    </div>
  );
}
