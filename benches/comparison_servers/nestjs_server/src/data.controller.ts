import { Controller, Get, Post, Query, Body, HttpCode, HttpStatus } from '@nestjs/common';

interface Product {
  id: number;
  name: string;
  description: string;
  price: number;
  category: string;
  tags: string[];
  inventory: {
    quantity: number;
    warehouse: string;
    last_updated: string;
  };
  metadata: {
    views: number;
    rating: number;
    reviews_count: number;
  };
}

function generateProducts(count: number): Product[] {
  const categories = ['Electronics', 'Clothing', 'Home', 'Sports'];
  const products: Product[] = [];

  for (let i = 0; i < count; i++) {
    products.push({
      id: i + 1,
      name: `Product ${i + 1}`,
      description: `This is the description for product ${i + 1}. It contains detailed information about the product.`,
      price: Math.round((Math.random() * 1000 + 10) * 100) / 100,
      category: categories[i % 4],
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

@Controller('data')
export class DataController {

  // GET /data - Complex data
  @Get()
  getData(@Query('size') size: string = 'medium'): object {
    let count: number;

    switch (size) {
      case 'small': count = 10; break;
      case 'large': count = 100; break;
      case 'xlarge': count = 500; break;
      default: count = 50;
    }

    const products = generateProducts(count);

    return {
      data: products,
      meta: {
        total: count,
        page: 1,
        per_page: count,
        timestamp: Date.now(),
      },
    };
  }

  // POST /data - Process data
  @Post()
  @HttpCode(HttpStatus.CREATED)
  processData(@Body() body: { items?: any[] }): object {
    return {
      received: true,
      items_count: Array.isArray(body?.items) ? body.items.length : 0,
      processed_at: new Date().toISOString(),
      checksum: Math.random().toString(36).substring(2, 15),
    };
  }
}

