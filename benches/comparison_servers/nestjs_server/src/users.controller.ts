import { Controller, Get, Post, Put, Delete, Param, Body, HttpCode, HttpStatus } from '@nestjs/common';

interface CreateUserDto {
  name: string;
  email?: string;
}

interface UpdateUserDto {
  name?: string;
  email?: string;
}

@Controller()
export class UsersController {

  // GET /users - List users
  @Get('users')
  getUsers(): object {
    const users = [
      { id: 1, name: 'Alice', email: 'alice@example.com' },
      { id: 2, name: 'Bob', email: 'bob@example.com' },
      { id: 3, name: 'Charlie', email: 'charlie@example.com' },
    ];

    return {
      users,
      total: users.length,
      page: 1,
      per_page: 10,
    };
  }

  // GET /users/:id - Get user by ID
  @Get('users/:id')
  getUser(@Param('id') id: string): object {
    return {
      id: parseInt(id, 10),
      name: 'John Doe',
      email: 'john@example.com',
      created_at: '2024-01-01T00:00:00Z',
    };
  }

  // POST /api/users - Create user
  @Post('api/users')
  @HttpCode(HttpStatus.CREATED)
  createUser(@Body() dto: CreateUserDto): object {
    return {
      id: Math.floor(Math.random() * 10000),
      name: dto.name,
      email: dto.email || `${dto.name?.toLowerCase() || 'user'}@example.com`,
      created: true,
    };
  }

  // PUT /users/:id - Update user
  @Put('users/:id')
  updateUser(@Param('id') id: string, @Body() dto: UpdateUserDto): object {
    return {
      id: parseInt(id, 10),
      name: dto.name || 'John Doe',
      email: dto.email || 'john@example.com',
      updated: true,
    };
  }

  // DELETE /users/:id - Delete user
  @Delete('users/:id')
  deleteUser(@Param('id') id: string): object {
    return {
      id: parseInt(id, 10),
      deleted: true,
    };
  }
}

