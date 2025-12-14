// User by ID endpoint - /api/users/[id]
// Benchmark: Path parameter extraction and JSON response

import { NextRequest, NextResponse } from 'next/server';

interface RouteParams {
  params: Promise<{ id: string }>;
}

// GET /api/users/:id - Get user by ID
export async function GET(
  request: NextRequest,
  context: RouteParams
) {
  const { id } = await context.params;

  // Simulate user lookup
  const user = {
    id: parseInt(id, 10),
    name: 'John Doe',
    email: 'john@example.com',
    created_at: '2024-01-01T00:00:00Z',
  };

  return NextResponse.json(user);
}

// PUT /api/users/:id - Update user
export async function PUT(
  request: NextRequest,
  context: RouteParams
) {
  const { id } = await context.params;

  try {
    const body = await request.json();

    const updatedUser = {
      id: parseInt(id, 10),
      name: body.name || 'John Doe',
      email: body.email || 'john@example.com',
      updated: true,
    };

    return NextResponse.json(updatedUser);
  } catch {
    return NextResponse.json(
      { error: 'Invalid JSON body' },
      { status: 400 }
    );
  }
}

// DELETE /api/users/:id - Delete user
export async function DELETE(
  request: NextRequest,
  context: RouteParams
) {
  const { id } = await context.params;

  return NextResponse.json({
    id: parseInt(id, 10),
    deleted: true,
  });
}

