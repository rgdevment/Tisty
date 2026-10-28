import type { Flow } from "./flow";
import type { FlowAltered } from "./flowing";
import type { Word } from "./words";

/** What a `style` line says about a node's colours. */
export interface Look {
  readonly fill?: string;
  readonly stroke?: string;
  readonly color?: string;
}

// A soft fill with dark words reads the same on a light or a dark page, and in Mermaid's drawing.
export const FILLS: readonly {
  readonly word: Word;
  readonly fill: string;
  readonly color: string;
}[] = [
  { word: "colorBlue", fill: "#dbeafe", color: "#1e3a8a" },
  { word: "colorGreen", fill: "#dcfce7", color: "#14532d" },
  { word: "colorAmber", fill: "#fef3c7", color: "#78350f" },
  { word: "colorRed", fill: "#fee2e2", color: "#7f1d1d" },
  { word: "colorPurple", fill: "#ede9fe", color: "#4c1d95" },
  { word: "colorGray", fill: "#f1f5f9", color: "#1e293b" },
];

export const LINES: readonly { readonly word: Word; readonly stroke: string }[] = [
  { word: "colorBlue", stroke: "#2563eb" },
  { word: "colorGreen", stroke: "#16a34a" },
  { word: "colorAmber", stroke: "#d97706" },
  { word: "colorRed", stroke: "#dc2626" },
  { word: "colorPurple", stroke: "#7c3aed" },
  { word: "colorGray", stroke: "#64748b" },
];

const STYLE = /^(\s*)style\s+([\p{L}\p{N}_-]+)\s+(.+?)\s*;?\s*$/u;

type Said = readonly (readonly [string, string])[];

// A comma inside brackets, as in `rgb(255,0,0)`, is part of the value, not the end of it.
const saidOf = (props: string): Said =>
  props.split(/,(?![^(]*\))/).flatMap((one) => {
    const at = one.indexOf(":");
    return at > 0 ? [[one.slice(0, at).trim(), one.slice(at + 1).trim()] as const] : [];
  });

// A key already said is rewritten where it stands, a new one goes last, an empty one goes.
const withSaid = (said: Said, key: string, value: string | null | undefined): Said => {
  if (!value) return said.filter(([one]) => one !== key);
  return said.some(([one]) => one === key)
    ? said.map(([one, was]) => [one, one === key ? value : was] as const)
    : [...said, [key, value] as const];
};

/** Every `style` line of the flowchart, by node, all its lines said as one as Mermaid reads them. */
const styleLines = (flow: Flow) => {
  const byNode = new Map<string, { lines: number[]; indent: string; said: Said }>();
  flow.body.forEach((line, at) => {
    const found = STYLE.exec(line);
    if (!found) return;
    const known = byNode.get(found[2]);
    // A later line says a key again over an earlier one.
    const said = saidOf(found[3]).reduce(
      (all, [key, value]) => withSaid(all, key, value),
      known?.said ?? [],
    );
    byNode.set(found[2], {
      lines: [...(known?.lines ?? []), at],
      indent: known?.indent ?? found[1],
      said,
    });
  });
  return byNode;
};

/** The colours each node's `style` lines give it. */
export const looksOf = (flow: Flow): ReadonlyMap<string, Look> =>
  new Map(
    [...styleLines(flow)].map(([id, { said }]) => {
      const props = new Map(said);
      return [
        id,
        { fill: props.get("fill"), stroke: props.get("stroke"), color: props.get("color") },
      ];
    }),
  );

/** A node's colours set or taken off, its `style` lines made one where the first stood, the rest kept. */
export const nodeStyled = (
  flow: Flow,
  id: string,
  change: { readonly [key in keyof Look]?: string | null },
): FlowAltered => {
  const known = styleLines(flow).get(id);
  const said = Object.entries(change).reduce<Said>(
    (all, [key, value]) => withSaid(all, key, value),
    known?.said ?? [],
  );
  const indent = known?.indent ?? (flow.body.find((one) => one.trim())?.match(/^\s*/)?.[0] || "  ");
  const written = said.length
    ? `${indent}style ${id} ${said.map(([key, value]) => `${key}:${value}`).join(",")}`
    : null;
  const first = known?.lines[0] ?? -1;
  const body = flow.body.flatMap((line, at) => {
    if (at === first) return written === null ? [] : [written];
    return known?.lines.includes(at) ? [] : [line];
  });
  return {
    body: first < 0 && written !== null ? [...body, written] : body,
    picked: { kind: "node", id },
  };
};
