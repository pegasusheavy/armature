// Vitest setup file
import { afterEach } from 'vitest';

// Cleanup after each test
afterEach(() => {
  // Cleanup DOM elements
  document.body.innerHTML = '';
});
