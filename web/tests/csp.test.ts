import { createHash } from 'node:crypto';
import { readFileSync, readdirSync, statSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { describe, expect, it } from 'vitest';
import { styleAttr, styleHash } from '../src/lib/csp';

function files(dir: string): string[] {
  return readdirSync(dir).flatMap((name) => {
    const path = join(dir, name);
    return statSync(path).isDirectory() ? files(path) : [path];
  });
}

const SRC = resolve(process.cwd(), 'src');
const sources = files(SRC).filter((f) => /\.(astro|ts|css|scss)$/.test(f));

describe('styleAttr', () => {
  it('registers the sha256 of the exact attribute value', () => {
    const inserted: unknown[] = [];
    const ctx = { csp: { insertStyleHash: (p: unknown) => inserted.push(p) } };
    const css = 'width: 42%; background-color: #ce422b';
    expect(styleAttr(ctx, css)).toBe(css);
    const expected = `sha256-${createHash('sha256').update(css).digest('base64')}`;
    expect(inserted).toEqual([{ hash: expected, kind: 'attribute' }]);
    expect(styleHash(css)).toBe(expected);
  });
});

describe('content security policy', () => {
  const config = readFileSync(resolve(process.cwd(), 'astro.config.mjs'), 'utf-8');

  it('is enabled with a self-only default and no unsafe-inline', () => {
    expect(config).toMatch(/security:\s*\{\s*csp:/);
    expect(config).toContain(`"default-src 'self'"`);
    expect(config).toContain(`"object-src 'none'"`);
    expect(config).not.toContain('unsafe-inline');
    expect(config).not.toContain('unsafe-eval');
  });

  it('every style attribute in src goes through styleAttr', () => {
    for (const file of sources.filter((f) => f.endsWith('.astro'))) {
      const text = readFileSync(file, 'utf-8');
      for (const match of text.matchAll(/\sstyle=(\{\s*styleAttr\(\s*Astro\s*,)?/g)) {
        expect(match[1], `${file}: raw style attribute`).toBeDefined();
      }
    }
  });

  it('no inline event handlers or javascript: URLs', () => {
    for (const file of sources.filter((f) => f.endsWith('.astro'))) {
      const text = readFileSync(file, 'utf-8');
      expect(text, file).not.toMatch(/\son[a-z]+=["{]/);
      expect(text, file).not.toMatch(/javascript:/i);
    }
  });

  it('loads no third-party fonts', () => {
    for (const file of sources) {
      const text = readFileSync(file, 'utf-8');
      expect(text, file).not.toMatch(/fonts\.(googleapis|gstatic)\.com/);
    }
  });

  it('ships the OFL license for each self-hosted font', () => {
    for (const name of ['Inter-OFL.txt', 'FiraCode-OFL.txt']) {
      const text = readFileSync(resolve(process.cwd(), 'public/fonts', name), 'utf-8');
      expect(text).toContain('SIL Open Font License');
    }
  });
});
