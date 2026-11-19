import { type Group, type Network, type NeuronKind, TIES, type Tie } from "../../brainsty";
import { WIDTH } from "./curves";
import { alphaOf } from "./guessing";
import { lineLook, nodeLook, type Palette, rgba, type Shape, type Standing } from "./looks";

export interface KindKey {
  kind: NeuronKind;
  shape: Shape;
  size: number;
  color: string;
}

export interface GroupKey {
  id: string;
  name: string;
  color: string;
  count: number;
}

export interface StandingKey {
  standing: Standing;
  size: number;
  color: string;
}

export interface LineKey {
  tie: Tie;
  color: string;
  size: number;
  dashed: boolean;
}

export interface Legend {
  kinds: KindKey[];
  groups: GroupKey[];
  standings: StandingKey[];
  lines: LineKey[];
  alike: { color: string; size: number } | null;
}

const KINDS: readonly NeuronKind[] = ["doc", "page", "task", "tag", "file"];

const STATES: Record<Standing, string | undefined> = {
  live: undefined,
  done: "done",
  archived: "archived",
};

// Each key is drawn by the same looks as the network, so the legend cannot drift from the drawing.
export function legendOf(network: Network, groups: readonly Group[], palette: Palette): Legend {
  const present = new Set(network.nodes.map((one) => one.kind));
  const kinds = KINDS.filter((kind) => present.has(kind)).map((kind) => {
    const look = nodeLook({ id: "", kind, title: "" }, undefined, palette);
    return { kind, ...look };
  });

  const counted = new Map<string, number>();
  for (const one of network.nodes)
    if (one.group) counted.set(one.group, (counted.get(one.group) ?? 0) + 1);
  const tinted = groups.flatMap((group) => {
    const count = counted.get(group.id);
    const color = group.color ? palette.hues[group.color] : undefined;
    return count && color ? [{ id: group.id, name: group.name, color, count }] : [];
  });
  tinted.sort((one, other) => other.count - one.count || one.name.localeCompare(other.name));

  const standings = (Object.keys(STATES) as Standing[]).map((standing) => {
    const look = nodeLook(
      { id: "", kind: "task", title: "", state: STATES[standing] },
      undefined,
      palette,
    );
    return { standing, size: look.size, color: look.color };
  });

  const tied = new Set(network.edges.map(([, , code]) => TIES[code]));
  const lines = TIES.filter((tie) => tied.has(tie)).map((tie) => {
    const look = lineLook(tie, palette);
    return { tie, color: look.color, size: look.size, dashed: look.gap > 0 };
  });

  const alike = network.alike?.length
    ? { color: rgba(palette.hues.amber, alphaOf(1)), size: WIDTH }
    : null;

  return { kinds, groups: tinted, standings, lines, alike };
}
