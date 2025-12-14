/**
 * Koa Benchmark Server
 * Port: 3007
 *
 * Implements identical endpoints to Armature for fair comparison.
 */

import Koa from 'koa';
import Router from 'koa-router';
import bodyParser from 'koa-bodyparser';

const app = new Koa();
const router = new Router();
const PORT = 3007;

// Middleware
app.use(bodyParser());

// Helper to generate products
function generateProducts(count) {
  const categories = ['Electronics', 'Clothing', 'Home', 'Sports'];
  const products = [];

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

// Routes

// GET / - Plaintext
router.get('/', (ctx) => {
  ctx.type = 'text/plain';
  ctx.body = 'Hello, World!';
});

// GET /json - JSON response
router.get('/json', (ctx) => {
  ctx.body = { message: 'Hello, World!' };
});

// GET /health - Health check
router.get('/health', (ctx) => {
  ctx.body = {
    status: 'ok',
    uptime: process.uptime(),
    memory: process.memoryUsage().heapUsed,
  };
});

// GET /users - List users
router.get('/users', (ctx) => {
  const users = [
    { id: 1, name: 'Alice', email: 'alice@example.com' },
    { id: 2, name: 'Bob', email: 'bob@example.com' },
    { id: 3, name: 'Charlie', email: 'charlie@example.com' },
  ];

  ctx.body = {
    users,
    total: users.length,
    page: 1,
    per_page: 10,
  };
});

// GET /users/:id - Get user by ID
router.get('/users/:id', (ctx) => {
  const { id } = ctx.params;

  ctx.body = {
    id: parseInt(id, 10),
    name: 'John Doe',
    email: 'john@example.com',
    created_at: '2024-01-01T00:00:00Z',
  };
});

// POST /api/users - Create user
router.post('/api/users', (ctx) => {
  const { name, email } = ctx.request.body;

  if (!name) {
    ctx.status = 400;
    ctx.body = { error: 'Name is required' };
    return;
  }

  ctx.status = 201;
  ctx.body = {
    id: Math.floor(Math.random() * 10000),
    name,
    email: email || `${name.toLowerCase()}@example.com`,
    created: true,
  };
});

// PUT /users/:id - Update user
router.put('/users/:id', (ctx) => {
  const { id } = ctx.params;
  const { name, email } = ctx.request.body;

  ctx.body = {
    id: parseInt(id, 10),
    name: name || 'John Doe',
    email: email || 'john@example.com',
    updated: true,
  };
});

// DELETE /users/:id - Delete user
router.delete('/users/:id', (ctx) => {
  const { id } = ctx.params;

  ctx.body = {
    id: parseInt(id, 10),
    deleted: true,
  };
});

// GET /data - Complex data
router.get('/data', (ctx) => {
  const size = ctx.query.size || 'medium';
  let count;

  switch (size) {
    case 'small': count = 10; break;
    case 'large': count = 100; break;
    case 'xlarge': count = 500; break;
    default: count = 50;
  }

  const products = generateProducts(count);

  ctx.body = {
    data: products,
    meta: {
      total: count,
      page: 1,
      per_page: count,
      timestamp: Date.now(),
    },
  };
});

// POST /data - Process data
router.post('/data', (ctx) => {
  const { items } = ctx.request.body;

  ctx.status = 201;
  ctx.body = {
    received: true,
    items_count: Array.isArray(items) ? items.length : 0,
    processed_at: new Date().toISOString(),
    checksum: Math.random().toString(36).substring(2, 15),
  };
});

// Use routes
app.use(router.routes());
app.use(router.allowedMethods());

// Start server
app.listen(PORT, () => {
  console.log(`
╔═══════════════════════════════════════════════════════════════╗
║              Koa Benchmark Server                              ║
╚═══════════════════════════════════════════════════════════════╝

🚀 Server running on http://localhost:${PORT}

Endpoints:
  GET  /           Plaintext response
  GET  /json       JSON response
  GET  /health     Health check
  GET  /users      List users
  GET  /users/:id  Get user by ID
  POST /api/users  Create user
  PUT  /users/:id  Update user
  DELETE /users/:id Delete user
  GET  /data       Complex data (?size=small|medium|large|xlarge)
  POST /data       Process data

Benchmark commands:
  oha -z 10s -c 50 http://localhost:${PORT}/
  oha -z 10s -c 50 http://localhost:${PORT}/json

Press Ctrl+C to stop
`);
});

