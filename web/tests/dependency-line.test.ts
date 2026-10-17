import { readFileSync, readdirSync, statSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { describe, expect, it } from 'vitest';
import { DEPENDENCY_LINE, DEPENDENCY_REQ, FRAMEWORK_VERSION } from '../src/lib/version';

function files(dir: string): string[] {
  return readdirSync(dir).flatMap((name) => {
    const path = join(dir, name);
    return statSync(path).isDirectory() ? files(path) : [path];
  });
}

// `armature` on crates.io is an unrelated crate; the framework is published as
// `armature-framework`. A copy-pasted `armature = "…"` line installs the wrong
// package, so every place the site tells a reader what to depend on must
// rename the package.
describe('dependency line', () => {
  it('renames the armature-framework package at the minor version', () => {
    expect(DEPENDENCY_REQ).toBe(FRAMEWORK_VERSION.split('.').slice(0, 2).join('.'));
    expect(DEPENDENCY_LINE).toBe(`armature = { package = "armature-framework", version = "${DEPENDENCY_REQ}" }`);
  });

  it('getting-started uses the shared line', () => {
    const page = readFileSync(resolve(process.cwd(), 'src/pages/getting-started.astro'), 'utf-8');
    expect(page).toContain('{DEPENDENCY_LINE}');
  });

  it('ai.txt quotes the current dependency line', () => {
    const ai = readFileSync(resolve(process.cwd(), 'public/ai.txt'), 'utf-8');
    expect(ai).toContain(DEPENDENCY_LINE);
  });

  it('no page or public file names the unrelated `armature` crate', () => {
    const roots = ['src', 'public'].map((d) => resolve(process.cwd(), d));
    for (const file of roots.flatMap(files).filter((f) => /\.(astro|ts|txt|md|json)$/.test(f))) {
      const text = readFileSync(file, 'utf-8');
      expect(text, file).not.toMatch(/^\s*armature\s*=\s*"/m);
      expect(text, file).not.toMatch(/(crates\.io\/crates|docs\.rs)\/armature(?![-\w])/);
    }
  });
});
