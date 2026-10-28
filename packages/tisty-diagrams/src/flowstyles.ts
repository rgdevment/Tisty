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

const saidOf = (props: string): Said =>
  props.split(",").flatMap((one) => {
    const at = one.indexOf(":");
    return at > 0 ? [[one.slice(0, at).trim(), one.slice(at + 1).trim()] as const] : [];
  });

const lineOf = (flow: Flow, id: string) =>
  flow.body.findIndex((line) => STYLE.exec(line)?.[2] === id);

/** The colours each node's `style` line gives it. */
export const looksOf = (flow: Flow): ReadonlyMap<string, Look> =>
  new Map(
    flow.body.flatMap((line) => {
      const found = STYLE.exec(line);
      if (!found) return [];
      const said = new Map(saidOf(found[3]));
      return [
        [
          found[2],
          { fill: said.get("fill"), stroke: said.get("stroke"), color: said.get("color") },
        ],
      ];
    }),
  );

// A key already said is rewritten where it stands, a new one goes last, an empty one goes.
const withSaid = (said: Said, key: string, value: string | null | undefined): Said => {
  if (!value) return said.filter(([one]) => one !== key);
  return said.some(([one]) => one === key)
    ? said.map(([one, was]) => [one, one === key ? value : was] as const)
    : [...said, [key, value] as const];
};

/** A node's colours set or taken off in its own `style` line, the rest of the line kept as it was. */
export const nodeStyled = (
  flow: Flow,
  id: string,
  change: { readonly [key in keyof Look]?: string | null },
): FlowAltered => {
  const at = lineOf(flow, id);
  const found = at >= 0 ? STYLE.exec(flow.body[at]) : null;
  const said = Object.entries(change).reduce<Said>(
    (all, [key, value]) => withSaid(all, key, value),
    found ? saidOf(found[3]) : [],
  );
  const indent = found?.[1] ?? (flow.body.find((one) => one.trim())?.match(/^\s*/)?.[0] || "  ");
  const written = said.length
    ? [`${indent}style ${id} ${said.map(([key, value]) => `${key}:${value}`).join(",")}`]
    : [];
  const body =
    at >= 0
      ? [...flow.body.slice(0, at), ...written, ...flow.body.slice(at + 1)]
      : [...flow.body, ...written];
  return { body, picked: { kind: "node", id } };
};
