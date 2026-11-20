import { type Group, type GroupKind, type Network, type NeuronKind, TIES } from "../../brainsty";
import type { Spots } from "./settling";

const MEMBER = TIES.indexOf("member");

export const isGroup = (kind: NeuronKind): kind is GroupKind =>
  kind === "folder" || kind === "list";

export function withGroupsAfter(network: Network, groups: readonly Group[]): Network {
  const held = new Map<string, number[]>();
  network.nodes.forEach((one, node) => {
    if (!one.group) return;
    const members = held.get(one.group);
    if (members) members.push(node);
    else held.set(one.group, [node]);
  });
  const nodes = [...network.nodes];
  const edges = [...network.edges];
  for (const group of groups) {
    const members = held.get(group.id);
    if (!group.kind || !members) continue;
    const hub = nodes.length;
    nodes.push({
      id: `${group.kind}:${group.id}`,
      kind: group.kind,
      title: group.name,
      group: group.id,
    });
    for (const member of members) edges.push([member, hub, MEMBER]);
  }
  return nodes.length === network.nodes.length ? network : { ...network, nodes, edges };
}

// A folder or a list would pull its members into a star, so the layout leaves it out and it sits amid them.
export function laidEdges(network: Network): Network["edges"] {
  const { nodes } = network;
  return network.edges.filter(
    ([from, to]) => !isGroup(nodes[from].kind) && !isGroup(nodes[to].kind),
  );
}

export function amidMembers(spots: Spots, network: Network): Spots {
  const { nodes } = network;
  const sums = new Map<string, [number, number, number]>();
  nodes.forEach((one, n) => {
    if (!one.group || isGroup(one.kind)) return;
    const [x, y, count] = sums.get(one.group) ?? [0, 0, 0];
    sums.set(one.group, [x + spots.x[n], y + spots.y[n], count + 1]);
  });
  const x = spots.x.slice();
  const y = spots.y.slice();
  nodes.forEach((one, n) => {
    const sum = one.group && isGroup(one.kind) ? sums.get(one.group) : undefined;
    if (!sum) return;
    x[n] = sum[0] / sum[2];
    y[n] = sum[1] / sum[2];
  });
  return { x, y };
}
