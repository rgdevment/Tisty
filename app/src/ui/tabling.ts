import { syntaxTree } from "@codemirror/language";
import type { EditorState } from "@codemirror/state";
import type { EditorView, KeyBinding } from "@codemirror/view";

export type Retable =
  | "rowAbove"
  | "rowBelow"
  | "columnLeft"
  | "columnRight"
  | "alignLeft"
  | "alignCenter"
  | "alignRight"
  | "rowRemove"
  | "columnRemove";

type Row = { readonly from: number; readonly text: string; readonly edges: number[] };
type Table = { readonly from: number; readonly to: number; readonly rows: Row[] };
type Done = { readonly text: string; readonly head: number };

// A row inside a quote carries its `>` marks before the table's own words, as a list's carries its indent.
const prefixOf = (line: string): string => /^(?:[ \t]*>)*[ \t]*/.exec(line)?.[0] ?? "";

const edgesOf = (line: string): number[] => {
  const pipes: number[] = [];
  const prefix = prefixOf(line).length;
  for (let at = prefix; at < line.length; at++) {
    if (line[at] === "\\") at++;
    else if (line[at] === "|") pipes.push(at);
  }
  const lead = pipes.length > 0 && !line.slice(prefix, pipes[0]).trim();
  const trail = pipes.length > (lead ? 1 : 0) && !line.slice(pipes[pipes.length - 1] + 1).trim();
  const inner = pipes.slice(lead ? 1 : 0, trail ? -1 : undefined);
  return [lead ? pipes[0] : prefix - 1, ...inner, trail ? pipes[pipes.length - 1] : line.length];
};

const RULED = /^:?-+:?$/;

const ruled = (line: string): boolean => {
  const edges = edgesOf(line);
  return edges.slice(1).every((end, at) => RULED.test(line.slice(edges[at] + 1, end).trim()));
};

export const tableAt = (text: string, head: number): Table | null => {
  const lines = text.split("\n");
  let start = 0;
  let index = 0;
  while (index < lines.length - 1 && start + lines[index].length < head) {
    start += lines[index].length + 1;
    index++;
  }
  if (!lines[index]?.includes("|")) return null;
  let first = index;
  while (first > 0 && lines[first - 1].includes("|")) first--;
  let last = index;
  while (last < lines.length - 1 && lines[last + 1].includes("|")) last++;
  if (last === first || !ruled(lines[first + 1])) return null;
  let from = 0;
  for (let at = 0; at < first; at++) from += lines[at].length + 1;
  const rows: Row[] = [];
  let at = from;
  for (const line of lines.slice(first, last + 1)) {
    rows.push({ from: at, text: line, edges: edgesOf(line) });
    at += line.length + 1;
  }
  return { from, to: at - 1, rows };
};

export const placed = (table: Table, head: number): { row: number; column: number } => {
  const row = Math.max(0, table.rows.filter((one) => one.from <= head).length - 1);
  const { edges, from } = table.rows[row];
  const column = edges.filter((edge) => edge < head - from).length - 1;
  return { row, column: Math.min(Math.max(column, 0), edges.length - 2) };
};

export const inside = (row: Row, column: number): [number, number] => {
  const from = row.edges[column] + 1;
  const to = row.edges[column + 1];
  const said = row.text.slice(from, to);
  const lead = said.length - said.trimStart().length;
  const kept = said.trim().length;
  if (!kept) return [row.from + Math.min(from + 1, to), row.from + Math.min(from + 1, to)];
  return [row.from + from + lead, row.from + from + lead + kept];
};

const blank = (row: Row): string => {
  const cells = row.edges.length - 1;
  const prefix = prefixOf(row.text);
  const lead = row.edges[0] >= prefix.length ? `${prefix}|` : prefix;
  const trail = row.edges[row.edges.length - 1] < row.text.length ? "|" : "";
  return `${lead}${Array.from({ length: cells }, () => "  ").join("|")}${trail}`;
};

// The rule keeps every dash it had: Tisty reads a column's width from how many there are.
const alignedRule = (rule: string, from: number, to: number, how: Retable): string => {
  const cell = rule.slice(from, to);
  const said = cell.trim();
  const lead = cell.slice(0, cell.length - cell.trimStart().length);
  const tail = cell.slice(lead.length + said.length);
  const dashes = "-".repeat(Math.max(1, said.replace(/:/g, "").length));
  const left = said.startsWith(":");
  const right = said.endsWith(":") && said.length > 1;
  const wanted = { alignLeft: [true, false], alignCenter: [true, true], alignRight: [false, true] }[
    how as "alignLeft"
  ];
  const [l, r] = wanted[0] === left && wanted[1] === right ? [false, false] : wanted;
  return `${rule.slice(0, from)}${lead}${l ? ":" : ""}${dashes}${r ? ":" : ""}${tail}${rule.slice(to)}`;
};

const lined = (table: Table, made: string[], text: string): string =>
  `${text.slice(0, table.from)}${made.join("\n")}${text.slice(table.to)}`;

const cellStart = (lines: string[], from: number, row: number, column: number): number => {
  let at = from;
  for (let one = 0; one < row; one++) at += lines[one].length + 1;
  const edges = edgesOf(lines[row]);
  const start = edges[Math.min(column, edges.length - 2)] + 1;
  return at + Math.min(start + 1, edges[Math.min(column, edges.length - 2) + 1]);
};

type Asked = {
  readonly text: string;
  readonly table: Table;
  readonly row: number;
  readonly column: number;
  readonly head: number;
};

const rowAdded = ({ text, table, row }: Asked, below: boolean): Done | null => {
  if (!below && row < 2) return null;
  const where = Math.max(below ? row + 1 : row, 2);
  const lines = table.rows.map((one) => one.text);
  lines.splice(where, 0, blank(table.rows[row]));
  return { text: lined(table, lines, text), head: cellStart(lines, table.from, where, 0) };
};

const rowRemoved = ({ text, table, row, column }: Asked): Done | null => {
  if (row < 2) return null;
  const lines = table.rows.map((one) => one.text);
  lines.splice(row, 1);
  const next = Math.min(row, lines.length - 1);
  return {
    text: lined(table, lines, text),
    head: next < 2 ? table.from + lines[0].length : cellStart(lines, table.from, next, column),
  };
};

const columnAdded = ({ text, table, row, column }: Asked, right: boolean): Done => {
  const made = table.rows.map((one, at) => {
    const cell = at === 1 ? " --- " : "  ";
    const last = one.edges.length - 1;
    if (!right) {
      const put = one.edges[Math.min(column, last)] + 1;
      return `${one.text.slice(0, put)}${cell}|${one.text.slice(put)}`;
    }
    const put = Math.min(one.edges[Math.min(column + 1, last)], one.text.length);
    return `${one.text.slice(0, put)}|${cell}${one.text.slice(put)}`;
  });
  const aimed = right ? column + 1 : column;
  return { text: lined(table, made, text), head: cellStart(made, table.from, row, aimed) };
};

const columnRemoved = ({ text, table, row, column }: Asked): Done | null => {
  if (table.rows[0].edges.length <= 2) return null;
  const made = table.rows.map(({ edges, text: line }) => {
    if (column >= edges.length - 1) return line;
    const inner = column + 1 < edges.length - 1;
    const left = inner
      ? `${line.slice(0, edges[column] + 1)}${line.slice(edges[column + 1] + 1)}`
      : `${line.slice(0, Math.max(edges[column], prefixOf(line).length))}${line.slice(edges[column + 1])}`;
    // One column without its outer pipes would no longer read as a table.
    return left.includes("|")
      ? left
      : `${prefixOf(line)}| ${left.slice(prefixOf(left).length).trim()} |`;
  });
  return {
    text: lined(table, made, text),
    head: cellStart(made, table.from, row, Math.max(0, column - 1)),
  };
};

const aligned = ({ text, table, column, head }: Asked, how: Retable): Done | null => {
  const rule = table.rows[1];
  if (column >= rule.edges.length - 1) return null;
  const lines = table.rows.map((one) => one.text);
  lines[1] = alignedRule(rule.text, rule.edges[column] + 1, rule.edges[column + 1], how);
  return { text: lined(table, lines, text), head };
};

export const retabled = (text: string, head: number, how: Retable): Done | null => {
  const table = tableAt(text, head);
  if (!table) return null;
  const asked: Asked = { text, table, head, ...placed(table, head) };
  if (how === "rowAbove" || how === "rowBelow") return rowAdded(asked, how === "rowBelow");
  if (how === "rowRemove") return rowRemoved(asked);
  if (how === "columnLeft" || how === "columnRight")
    return columnAdded(asked, how === "columnRight");
  if (how === "columnRemove") return columnRemoved(asked);
  return aligned(asked, how);
};

export const widened = (
  text: string,
  [from, end]: [number, number],
  column: number,
  dashes: number,
): string | null => {
  const lines = text.slice(from, end).split("\n");
  if (lines.length < 2 || !ruled(lines[1])) return null;
  const edges = edgesOf(lines[1]);
  if (column >= edges.length - 1) return null;
  const start = edges[column] + 1;
  const cell = lines[1].slice(start, edges[column + 1]);
  const made = cell.replace(/-+/, "-".repeat(dashes));
  if (made === cell) return null;
  const at = from + lines[0].length + 1 + start;
  return `${text.slice(0, at)}${made}${text.slice(at + cell.length)}`;
};

// A table shown inside a code block is code, and Tab there indents as it always did.
export const tableHere = (state: EditorState): Table | null => {
  const { head } = state.selection.main;
  for (let node = syntaxTree(state).resolveInner(head, 1); node.parent; node = node.parent) {
    if (node.name === "FencedCode" || node.name === "CodeBlock") return null;
  }
  return tableAt(state.doc.toString(), head);
};

// Only what changed is written, so undo and the caret's neighbours stay as they were.
export const replaced = (view: EditorView, done: Done) => {
  const text = view.state.doc.toString();
  let from = 0;
  while (from < text.length && from < done.text.length && text[from] === done.text[from]) from++;
  let end = 0;
  while (
    end < text.length - from &&
    end < done.text.length - from &&
    text[text.length - 1 - end] === done.text[done.text.length - 1 - end]
  )
    end++;
  view.dispatch({
    changes: { from, to: text.length - end, insert: done.text.slice(from, done.text.length - end) },
    selection: { anchor: done.head },
    scrollIntoView: true,
    userEvent: "input",
  });
};

// Tab walks the cells and past the last one starts a new row, as a spreadsheet does.
const walked = (view: EditorView, forward: boolean): boolean => {
  const table = tableHere(view.state);
  if (!table) return false;
  const text = view.state.doc.toString();
  const { row, column } = placed(table, view.state.selection.main.head);
  const cells = table.rows.map((one) => one.edges.length - 1);
  let [r, c] = [row, column + (forward ? 1 : -1)];
  if (c >= cells[r]) [r, c] = [r === 0 ? 2 : r + 1, 0];
  if (c < 0) {
    r = r === 2 ? 0 : r - 1;
    c = r >= 0 ? cells[r] - 1 : 0;
  }
  if (r < 0) return true;
  if (r >= table.rows.length) {
    const done = retabled(text, table.rows[table.rows.length - 1].from, "rowBelow");
    if (done) replaced(view, done);
    return true;
  }
  const [from, to] = inside(table.rows[r], c);
  view.dispatch({ selection: { anchor: from, head: to }, scrollIntoView: true });
  return true;
};

export const tableKeys: KeyBinding[] = [
  { key: "Tab", run: (view) => walked(view, true), shift: (view) => walked(view, false) },
];

export const retable = (view: EditorView, how: Retable): boolean => {
  const done = tableHere(view.state)
    ? retabled(view.state.doc.toString(), view.state.selection.main.head, how)
    : null;
  if (done) replaced(view, done);
  view.focus();
  return Boolean(done);
};
