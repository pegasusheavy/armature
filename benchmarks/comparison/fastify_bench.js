// Fastify benchmark server
// Run with: node fastify_bench.js

const fastify = require('fastify')({ logger: false });

const port = process.env.PORT || 8084;

fastify.get('/json', async (request, reply) => {
    return { message: 'Hello, World!' };
});

fastify.get('/plaintext', async (request, reply) => {
    reply.type('text/plain');
    return 'Hello, World!';
});

const start = async () => {
    try {
        await fastify.listen({ port: port, host: '0.0.0.0' });
        console.log(`Fastify benchmark server starting on port ${port}`);
    } catch (err) {
        console.error(err);
        process.exit(1);
    }
};

start();

