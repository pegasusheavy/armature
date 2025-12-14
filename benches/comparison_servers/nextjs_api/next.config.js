/** @type {import('next').NextConfig} */
const nextConfig = {
  // Optimize for API-only usage
  reactStrictMode: false,
  poweredByHeader: false,

  // Disable unnecessary features for API benchmark
  images: {
    unoptimized: true,
  },

  // Optimize for production
  productionBrowserSourceMaps: false,

  // API response optimization
  async headers() {
    return [
      {
        source: '/api/:path*',
        headers: [
          { key: 'Cache-Control', value: 'no-store' },
        ],
      },
    ];
  },
};

module.exports = nextConfig;

