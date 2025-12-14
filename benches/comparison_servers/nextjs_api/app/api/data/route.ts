// Data endpoint - /api/data
// Benchmark: Complex JSON serialization with nested objects

import { NextRequest, NextResponse } from 'next/server';

// Simulate realistic API response data
function generateProducts(count: number) {
  const products = [];
  for (let i = 0; i < count; i++) {
    products.push({
      id: i + 1,
      name: `Product ${i + 1}`,
      description: `This is the description for product ${i + 1}. It contains detailed information about the product.`,
      price: Math.round((Math.random() * 1000 + 10) * 100) / 100,
      category: ['Electronics', 'Clothing', 'Home', 'Sports'][i % 4],
      tags: ['sale', 'new', 'popular'].slice(0, (i % 3) + 1),
      inventory: {
        quantity: Math.floor(Math.random() * 100),
        warehouse: `WH-${(i % 5) + 1}`,
        last_updated: new Date().toISOString(),
      },
      metadata: {
        views: Math.floor(Math.random() * 10000),
        rating: Math.round((Math.random() * 2 + 3) * 10) / 10,
        reviews_count: Math.floor(Math.random() * 500),
      },
    });
  }
  return products;
}

// GET /api/data - Get complex data
export async function GET(request: NextRequest) {
  const { searchParams } = new URL(request.url);
  const size = searchParams.get('size') || 'medium';

  let count: number;
  switch (size) {
    case 'small':
      count = 10;
      break;
    case 'large':
      count = 100;
      break;
    case 'xlarge':
      count = 500;
      break;
    default:
      count = 50;
  }

  const products = generateProducts(count);

  return NextResponse.json({
    data: products,
    meta: {
      total: count,
      page: 1,
      per_page: count,
      timestamp: Date.now(),
    },
  });
}

// POST /api/data - Process complex data
export async function POST(request: NextRequest) {
  try {
    const body = await request.json();

    // Simulate data processing
    const processed = {
      received: true,
      items_count: Array.isArray(body.items) ? body.items.length : 0,
      processed_at: new Date().toISOString(),
      checksum: Math.random().toString(36).substring(2, 15),
    };

    return NextResponse.json(processed, { status: 201 });
  } catch {
    return NextResponse.json(
      { error: 'Invalid JSON body' },
      { status: 400 }
    );
  }
}

