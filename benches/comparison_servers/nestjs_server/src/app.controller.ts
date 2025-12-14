import { Controller, Get, Header } from '@nestjs/common';

@Controller()
export class AppController {

  // GET / - Plaintext
  @Get()
  @Header('Content-Type', 'text/plain')
  getPlaintext(): string {
    return 'Hello, World!';
  }

  // GET /json - JSON response
  @Get('json')
  getJson(): object {
    return { message: 'Hello, World!' };
  }

  // GET /health - Health check
  @Get('health')
  getHealth(): object {
    return {
      status: 'ok',
      uptime: process.uptime(),
      memory: process.memoryUsage().heapUsed,
    };
  }
}

