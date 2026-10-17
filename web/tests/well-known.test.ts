import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { describe, expect, it } from 'vitest';

const read = (p: string) => readFileSync(resolve(process.cwd(), 'public', p), 'utf-8');

function fields(text: string): Map<string, string[]> {
  const out = new Map<string, string[]>();
  for (const line of text.split('\n')) {
    const m = /^([A-Za-z-]+):\s*(.+)$/.exec(line);
    if (!m) continue;
    out.set(m[1], [...(out.get(m[1]) ?? []), m[2].trim()]);
  }
  return out;
}

describe('security.txt (RFC 9116)', () => {
  const f = fields(read('.well-known/security.txt'));

  it('has exactly one Expires, in the future and at most a year out', () => {
    const expires = f.get('Expires') ?? [];
    expect(expires).toHaveLength(1);
    const when = Date.parse(expires[0]);
    expect(when).toBeGreaterThan(Date.now());
    // RFC 9116 2.5.5: recommended to be less than a year in the future.
    expect(when - Date.now()).toBeLessThanOrEqual(366 * 24 * 3600 * 1000);
  });

  it('routes reports to GitHub private advisories', () => {
    expect(f.get('Contact')?.[0]).toBe('https://github.com/quinnjr/armature/security/advisories/new');
  });

  it('declares the URI it is actually served from as Canonical', () => {
    expect(f.get('Canonical')).toEqual(['https://quinnjr.github.io/armature/.well-known/security.txt']);
  });
});

describe('ai.txt', () => {
  const text = read('ai.txt');

  it('makes no production-readiness or completion claims', () => {
    expect(text).not.toMatch(/production[- ]ready/i);
    expect(text).not.toMatch(/\d+% feature/i);
  });

  it('states the pre-1.0 status', () => {
    expect(text).toMatch(/pre-1\.0/i);
  });
});
