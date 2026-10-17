import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { describe, expect, it } from 'vitest';
import { escapeJsonLd } from '../src/lib/jsonld';

describe('escapeJsonLd', () => {
  it('cannot close the surrounding script element', () => {
    const json = JSON.stringify({ name: '</script><script>alert(1)</script><!--' });
    const out = escapeJsonLd(json);
    expect(out).not.toMatch(/[<>]/);
    expect(out.toLowerCase()).not.toContain('</script');
  });

  it('escapes &, U+2028 and U+2029', () => {
    const out = escapeJsonLd(JSON.stringify({ a: 'x & y\u2028z\u2029' }));
    expect(out).not.toMatch(/[&\u2028\u2029]/);
    expect(out).toContain('\\u0026');
    expect(out).toContain('\\u2028');
    expect(out).toContain('\\u2029');
  });

  it('round-trips to the same value', () => {
    const value = { s: '<a href="?x=1&y=2">\u2028\u2029</a>', n: 1, nested: ['>'] };
    expect(JSON.parse(escapeJsonLd(JSON.stringify(value)))).toEqual(value);
  });
});

describe('BaseLayout JSON-LD', () => {
  const layout = readFileSync(resolve(process.cwd(), 'src/layouts/BaseLayout.astro'), 'utf-8');

  it('routes every ld+json set:html through escapeJsonLd', () => {
    const tags = layout.match(/<script type="application\/ld\+json"[^>]*>/g) ?? [];
    expect(tags.length).toBeGreaterThan(0);
    for (const tag of tags) {
      expect(tag).toMatch(/set:html=\{escapeJsonLd\(/);
    }
  });
});
