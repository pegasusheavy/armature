// Users endpoint - /api/users
// Benchmark: JSON body parsing and response

import { NextRequest, NextResponse } from 'next/server';

// GET /api/users - List users
export async function GET() {
  const users = [
    { id: 1, name: 'Alice', email: 'alice@example.com' },
    { id: 2, name: 'Bob', email: 'bob@example.com' },
    { id: 3, name: 'Charlie', email: 'charlie@example.com' },
  ];

  return NextResponse.json({
    users,
    total: users.length,
    page: 1,
    per_page: 10,
  });
}

// POST /api/users - Create user
export async function POST(request: NextRequest) {
  try {
    const body = await request.json();

    // Validate required fields
    if (!body.name) {
      return NextResponse.json(
        { error: 'Name is required' },
        { status: 400 }
      );
    }

    // Simulate user creation
    const newUser = {
      id: Math.floor(Math.random() * 10000),
      name: body.name,
      email: body.email || `${body.name.toLowerCase()}@example.com`,
      created: true,
    };

    return NextResponse.json(newUser, { status: 201 });
  } catch {
    return NextResponse.json(
      { error: 'Invalid JSON body' },
      { status: 400 }
    );
  }
}

