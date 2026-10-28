import type { NodeChange } from "@xyflow/react";

/** What React Flow's select changes leave chosen: several together, one newly chosen alone, or no news. */
export const choiceAfter = (
  chosen: readonly string[],
  changes: readonly NodeChange[],
  skip: (id: string) => boolean,
): { readonly several: readonly string[] } | { readonly one: string } | null => {
  const now = new Set(chosen);
  const added: string[] = [];
  for (const one of changes) {
    if (one.type !== "select" || skip(one.id)) continue;
    if (one.selected) added.push(one.id);
    one.selected ? now.add(one.id) : now.delete(one.id);
  }
  if (now.size > 1) return { several: [...now] };
  return now.size === 1 && added.length ? { one: added[0] } : null;
};
