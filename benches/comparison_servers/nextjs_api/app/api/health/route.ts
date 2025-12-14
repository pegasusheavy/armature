// Health check endpoint - /api/health
// Benchmark: Simple JSON response

import { NextResponse } from 'next/server';

export async function GET() {
  return NextResponse.json({
    status: 'ok',
    uptime: process.uptime(),
    memory: process.memoryUsage().heapUsed,
  });
}

