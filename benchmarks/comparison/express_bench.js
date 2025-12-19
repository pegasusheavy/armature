// Express.js benchmark server
// Run with: node express_bench.js

const express = require('express');
const app = express();

const port = process.env.PORT || 8081;

// Disable ETag and other middleware for raw performance
app.disable('etag');
app.disable('x-powered-by');

app.get('/json', (req, res) => {
    res.json({ message: 'Hello, World!' });
});

app.get('/plaintext', (req, res) => {
    res.set('Content-Type', 'text/plain');
    res.send('Hello, World!');
});

app.listen(port, () => {
    console.log(`Express benchmark server starting on port ${port}`);
});

