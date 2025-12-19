import { NestFactory } from '@nestjs/core';
import { FastifyAdapter, NestFastifyApplication } from '@nestjs/platform-fastify';
import { Module, Controller, Get } from '@nestjs/common';

@Controller()
class BenchController {
  @Get('/json')
  json() {
    return { message: 'Hello, World!' };
  }

  @Get('/plaintext')
  plaintext() {
    return 'Hello, World!';
  }
}

@Module({
  controllers: [BenchController],
})
class AppModule {}

async function bootstrap() {
  const app = await NestFactory.create<NestFastifyApplication>(
    AppModule,
    new FastifyAdapter(),
    { logger: false }
  );
  const port = process.env.PORT || 8086;
  await app.listen(port, '0.0.0.0');
  console.log(`NestJS (Fastify) benchmark server starting on port ${port}`);
}

bootstrap();

