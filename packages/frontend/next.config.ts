import type { NextConfig } from "next";
import createMDX from "@next/mdx";

const nextConfig: NextConfig = {
  // Configure `pageExtensions` to include MDX files
  pageExtensions: ["js", "jsx", "mdx", "ts", "tsx"],
  // Performance optimizations
  experimental: {
    optimizeCss: true,
  },
  // Enable Turbopack for faster builds (Next.js 15+)
  turbopack: {
    rules: {
      "*.svg": {
        loaders: ["@svgr/webpack"],
        as: "*.js",
      },
    },
  },
  // Optimize images
  images: {
    formats: ["image/webp", "image/avif"],
  },
  // Enable compression
  compress: true,
  // Fix for pino-pretty module resolution in browser
  webpack: (config, { isServer }) => {
    if (!isServer) {
      config.resolve.fallback = {
        ...config.resolve.fallback,
        fs: false,
        net: false,
        tls: false,
        crypto: false,
        stream: false,
        url: false,
        zlib: false,
        http: false,
        https: false,
        assert: false,
        os: false,
        path: false,
      };
    }

    // Handle pino-pretty module resolution
    config.resolve.alias = {
      ...config.resolve.alias,
      "pino-pretty": false,
    };

    return config;
  },
  // Optionally, add any other Next.js config below
};

const withMDX = createMDX({
  // Add markdown plugins here, as desired
});

// Wrap MDX and Next.js config together
export default withMDX(nextConfig);
