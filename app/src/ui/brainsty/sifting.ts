import type { Network, NeuronKind } from "../../brainsty";
import { isGroup } from "./grouping";
import { type Standing, standingOf } from "./looks";

export const KINDS: readonly NeuronKind[] = [
  "doc",
  "page",
  "task",
  "tag",
  "file",
  "folder",
  "list",
];
export const STANDINGS: readonly Standing[] = ["live", "done", "archived"];
export const PERIODS = [0, 7, 30, 365] as const;

export type Period = (typeof PERIODS)[number];

export interface Sift {
  kinds: ReadonlySet<NeuronKind>;
  standings: ReadonlySet<Standing>;
  tag: number | null;
  days: Period;
  alike: boolean;
}

export const EVERYTHING: Sift = {
  kinds: new Set(KINDS),
  standings: new Set(STANDINGS),
  tag: null,
  days: 0,
  alike: true,
};

const TAG_TIE = 3;
const DAY = 86_400_000;
const MOST_FOUND = 8;

const kept = (kind: NeuronKind): boolean =>
  kind !== "tag" && kind !== "file" && kind !== "folder" && kind !== "list";

export const sifting = (sift: Sift): boolean =>
  sift.kinds.size < KINDS.length ||
  sift.standings.size < STANDINGS.length ||
  sift.tag !== null ||
  sift.days !== 0 ||
  !sift.alike;

export function holdersOf(network: Network, tag: number): Set<number> {
  const held = new Set([tag]);
  for (const [from, to, tie] of network.edges) {
    if (tie !== TAG_TIE) continue;
    if (to === tag) held.add(from);
    if (from === tag) held.add(to);
  }
  return held;
}

export function hiddenBy(network: Network, sift: Sift, today: Date): boolean[] {
  const tagged = sift.tag === null ? null : holdersOf(network, sift.tag);
  const since = sift.days
    ? new Date(today.getTime() - sift.days * DAY).toISOString().slice(0, 10)
    : null;
  const hidden = network.nodes.map((one, n) => {
    if (!sift.kinds.has(one.kind)) return true;
    if (tagged && !tagged.has(n)) return true;
    if (!kept(one.kind)) return false;
    if (!sift.standings.has(standingOf(one))) return true;
    return since !== null && (one.when === undefined || one.when < since);
  });
  const seen = new Set(
    network.nodes.flatMap((one, n) =>
      one.group && !isGroup(one.kind) && !hidden[n] ? [one.group] : [],
    ),
  );
  return hidden.map(
    (gone, n) =>
      gone || (isGroup(network.nodes[n].kind) && !seen.has(network.nodes[n].group ?? "")),
  );
}

export const folded = (text: string): string =>
  text
    .normalize("NFD")
    .replace(/\p{Diacritic}/gu, "")
    .toLowerCase();

const scoreOf = (title: string, asked: string): number => {
  if (title.startsWith(asked)) return 0;
  if (title.includes(` ${asked}`) || title.includes(`#${asked}`)) return 1;
  return title.includes(asked) ? 2 : -1;
};

export function found(network: Network, query: string, hidden: readonly boolean[]): number[] {
  const asked = folded(query.trim());
  if (!asked) return [];
  return network.nodes
    .map((one, n) => {
      const title = folded(one.title);
      return { n, title, score: scoreOf(title, asked) };
    })
    .filter((one) => one.score >= 0 && !hidden[one.n])
    .sort((one, other) => one.score - other.score || one.title.length - other.title.length)
    .slice(0, MOST_FOUND)
    .map((one) => one.n);
}
