/**
 * Make a serialized JSON-LD document safe to inline in a `<script>` element.
 *
 * JSON.stringify leaves `<`, `>` and `&` untouched, so a string value holding
 * `</script>` (or `<!--`) ends the element early and whatever follows is parsed
 * as HTML. U+2028/U+2029 are legal in JSON strings but are line terminators to
 * older JavaScript parsers. Replacing each with its `\uXXXX` escape keeps the
 * document byte-for-byte equivalent JSON while making it inert as markup.
 */
const UNSAFE = /[<>&\u2028\u2029]/g;

const ESCAPES: Record<string, string> = {
  '<': '\\u003c',
  '>': '\\u003e',
  '&': '\\u0026',
  '\u2028': '\\u2028',
  '\u2029': '\\u2029',
};

export function escapeJsonLd(json: string): string {
  return json.replace(UNSAFE, (ch) => ESCAPES[ch]);
}
