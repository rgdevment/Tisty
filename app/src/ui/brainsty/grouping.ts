import type { Group } from "../../brainsty";

// Only what the network draws of a group: a folder's other fields change with each write.
export const groupsKey = (groups: readonly Group[]): string =>
  JSON.stringify(groups.map((one) => [one.id, one.name, one.color ?? null, !!one.agents]));
