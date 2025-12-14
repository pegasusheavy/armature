/**
 * NestJS Benchmark Server
 * Port: 3008
 *
 * Implements identical endpoints to Armature for fair comparison.
 */

import { NestFactory } from '@nestjs/core';
import { AppModule } from './app.module';

async function bootstrap() {
  const app = await NestFactory.create(AppModule, {
    logger: false, // Disable logging for benchmarks
  });

  await app.listen(3008);

  console.log(`
╔═══════════════════════════════════════════════════════════════╗
║            NestJS Benchmark Server                             ║
╚═══════════════════════════════════════════════════════════════╝

🚀 Server running on http://localhost:3008

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
  oha -z 10s -c 50 http://localhost:3008/
  oha -z 10s -c 50 http://localhost:3008/json

Press Ctrl+C to stop
`);
}

bootstrap();

