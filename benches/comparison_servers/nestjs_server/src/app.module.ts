import { Module } from '@nestjs/common';
import { AppController } from './app.controller';
import { UsersController } from './users.controller';
import { DataController } from './data.controller';

@Module({
  imports: [],
  controllers: [AppController, UsersController, DataController],
  providers: [],
})
export class AppModule {}

