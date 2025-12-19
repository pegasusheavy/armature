import { NestFactory } from '@nestjs/core';
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
  const app = await NestFactory.create(AppModule, { logger: false });
  const port = process.env.PORT || 8085;
  await app.listen(port);
  console.log(`NestJS (Express) benchmark server starting on port ${port}`);
}

bootstrap();

