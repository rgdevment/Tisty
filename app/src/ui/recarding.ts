export type Reshape = "link" | "remove";

const quoted = (said: string): string => said.replace(/[.*+?^${}()|[\]\\]/g, String.raw`\$&`);

const found = (said: string, target: string): RegExpExecArray | null =>
  new RegExp(
    String.raw`!\[((?:\\.|[^\]\\])*)\]\(<?${quoted(target)}>?(\s+(?:"[^"]*"|'[^']*'|\([^)]*\)))?\)`,
  ).exec(said);

const FILED = /^\s+["'(]file["')]$/;

// A card is one picture-shaped line of Markdown; every change to it is made to that line and nothing else.
export const reshaped = (
  text: string,
  [from, end]: [number, number],
  target: string,
  how: Reshape,
): string | null => {
  const block = text.slice(from, end);
  const hit = found(block, target);
  if (!hit) return null;
  const [whole, label, title = ""] = hit;
  const at = from + hit.index;
  const aimed = target.includes(" ") ? `<${target}>` : target;
  const named = `](${aimed}`;
  const kept = FILED.test(title) ? "" : title;
  const made = {
    link: `[${label}${named}${kept})`,
    remove: "",
  }[how];
  let next = `${text.slice(0, at)}${made}${text.slice(at + whole.length)}`;
  if (how === "remove" && !next.slice(from, end - whole.length).trim()) {
    const rest = next.slice(from + (end - from - whole.length)).replace(/^(?:\r?\n){1,2}/, "");
    next = `${next.slice(0, from)}${rest}`;
  }
  return next;
};
