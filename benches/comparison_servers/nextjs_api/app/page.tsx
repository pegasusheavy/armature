export default function Home() {
  return (
    <main style={{ padding: '2rem', fontFamily: 'system-ui' }}>
      <h1>Next.js API Benchmark Server</h1>
      <p>Port: 3005</p>

      <h2>Available Endpoints</h2>
      <ul>
        <li><code>GET /api</code> - Plaintext response</li>
        <li><code>GET /api/json</code> - JSON response</li>
        <li><code>GET /api/health</code> - Health check</li>
        <li><code>GET /api/users</code> - List users</li>
        <li><code>POST /api/users</code> - Create user</li>
        <li><code>GET /api/users/:id</code> - Get user by ID</li>
        <li><code>PUT /api/users/:id</code> - Update user</li>
        <li><code>DELETE /api/users/:id</code> - Delete user</li>
        <li><code>GET /api/data?size=small|medium|large|xlarge</code> - Complex data</li>
        <li><code>POST /api/data</code> - Process data</li>
      </ul>

      <h2>Test Commands</h2>
      <pre style={{ background: '#f0f0f0', padding: '1rem', borderRadius: '4px' }}>
{`# Plaintext
curl http://localhost:3005/api

# JSON
curl http://localhost:3005/api/json

# Users
curl http://localhost:3005/api/users
curl http://localhost:3005/api/users/123

# Create user
curl -X POST http://localhost:3005/api/users \\
  -H "Content-Type: application/json" \\
  -d '{"name":"Test User"}'

# Complex data
curl http://localhost:3005/api/data?size=large

# Benchmark with oha
oha -z 10s -c 50 http://localhost:3005/api/json`}
      </pre>
    </main>
  );
}

