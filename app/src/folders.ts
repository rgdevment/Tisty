import { DEEPEST, type Folded } from "./core";
import { t } from "./locales";
import type { Choice } from "./ui/Menu";

export function deep(folders: Folded[], at: string | null | undefined): number {
  let steps = 0;
  const seen = new Set<string>();
  for (let up = at; up && !seen.has(up); ) {
    seen.add(up);
    steps += 1;
    up = folders.find((one) => one.id === up)?.parent ?? null;
  }
  return steps;
}

export function trail(folders: Folded[], at: string): string {
  const names: string[] = [];
  const seen = new Set<string>();
  for (let up: string | null | undefined = at; up && !seen.has(up); ) {
    seen.add(up);
    const one = folders.find((each) => each.id === up);
    if (!one) break;
    names.unshift(one.name);
    up = one.parent;
  }
  return names.join(" / ");
}

export function destinations(
  folders: Folded[],
  skip: string | null,
  land: (folder?: string) => void,
  moving?: Folded,
): Choice[] {
  const under = (at: string): string[] => {
    const kids = folders.filter((one) => one.parent === at);
    return kids.flatMap((one) => [one.id, ...under(one.id)]);
  };
  const forbidden = moving ? new Set([moving.id, ...under(moving.id)]) : new Set<string>();
  const tallest = (at: string): number =>
    1 +
    folders
      .filter((one) => one.parent === at)
      .reduce((most, one) => Math.max(most, tallest(one.id)), 0);
  const tall = moving ? tallest(moving.id) : 0;

  return [
    {
      key: "unfiled",
      icon: "↥",
      label: t("unfiled"),
      off: skip === null,
      onPick: () => land(undefined),
    },
    ...folders
      .filter((one) => one.id !== skip && !forbidden.has(one.id))
      .filter((one) => !moving || deep(folders, one.id) + tall <= DEEPEST)
      .map((one) => ({
        key: one.id,
        icon: one.parent ? "↳" : "▸",
        label: trail(folders, one.id),
        onPick: () => land(one.id),
      })),
  ];
}
