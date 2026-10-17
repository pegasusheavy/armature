import { createHash } from 'node:crypto';

/** Minimal slice of Astro's CSP runtime API that this helper needs. */
interface CspContext {
  csp?: { insertStyleHash: (payload: { hash: string; kind: 'attribute' }) => void };
}

/**
 * Return `css` for use as a `style` attribute after allow-listing its exact
 * value in the page's CSP (`style-src-attr 'unsafe-hashes' 'sha256-…'`).
 *
 * The site computes bar widths and colours at build time, so the values are
 * known when the page is rendered; hashing each one keeps `'unsafe-inline'`
 * out of the policy entirely.
 */
export function styleAttr(ctx: CspContext, css: string): string {
  ctx.csp?.insertStyleHash({ hash: styleHash(css), kind: 'attribute' });
  return css;
}

/** CSP source expression for a style attribute value. */
export function styleHash(css: string): string {
  return `sha256-${createHash('sha256').update(css, 'utf8').digest('base64')}`;
}
