import { LANGUAGES } from "./codeLanguages";

type Edit = { readonly from: number; readonly to: number; readonly insert: string };
type Moved = { readonly edit: Edit; readonly caret: number };

const FENCE_WORDS = new Set([
  ...LANGUAGES.map(([tag]) => tag),
  ...["js", "ts", "jsx", "tsx", "py", "sh", "zsh", "rb", "rs", "yml", "md", "kt", "cs", "toml"],
  ...["text", "txt", "mermaid", "mmd", "widget", "math"],
]);

const BARE_FENCE = /^[ \t]*(`{3,}|~{3,})$/;
const OPENING_FENCE = /^[ \t]*(`{3,}|~{3,})[^`~]*$/;

const lineAt = (text: string, at: number) => {
  const start = text.lastIndexOf("\n", at - 1) + 1;
  const ends = text.indexOf("\n", at);
  return { start, line: text.slice(start, ends < 0 ? text.length : ends).replace(/\r$/, "") };
};

// An unclosed fence turns every block under it into code, so the third backtick closes it at once.
export const fenceOpened = (text: string, at: number, typed: string): Moved | null => {
  if (typed !== "`" && typed !== "~") return null;
  const { start, line } = lineAt(text, at);
  const lead = new RegExp(`^([ \\t]*)\\${typed}\\${typed}$`).exec(text.slice(start, at));
  if (!lead || line.slice(at - start).trim() !== "") return null;
  const indent = lead[1];
  return {
    edit: { from: at, to: at, insert: `${typed}\n${indent}\n${indent}${typed.repeat(3)}` },
    caret: at + 2 + indent.length,
  };
};

export const fenceNamed = (text: string, at: number): Moved | null => {
  const { start, line } = lineAt(text, at);
  if (at < start + line.length || start === 0) return null;
  const word = line.trim();
  if (!FENCE_WORDS.has(word.toLowerCase())) return null;
  const above = lineAt(text, start - 1);
  if (!BARE_FENCE.test(above.line)) return null;
  const fenceEnd = above.start + above.line.length;
  return {
    edit: { from: fenceEnd, to: start + line.length, insert: `${word}\n` },
    caret: fenceEnd + word.length + 1,
  };
};

export const fenceJustOpened = (text: string, at: number): boolean => {
  const { start, line } = lineAt(text, at);
  const ends = text.indexOf("\n", at);
  if (line.trim() !== "" || start === 0 || ends < 0) return false;
  return (
    OPENING_FENCE.test(lineAt(text, start - 1).line) && BARE_FENCE.test(lineAt(text, ends + 1).line)
  );
};
