const SKILL_FILE = "/skill.md";

/// The folders holding a SKILL.md, shortest first: a SKILL.md deeper inside a skill is one of its files.
export const skillDirs = (names: readonly string[]): string[] =>
  names
    .filter((name) => name.toLowerCase().endsWith(SKILL_FILE))
    .map((name) => name.slice(0, -SKILL_FILE.length))
    .sort((one, two) => one.length - two.length);

/// The skill a file belongs to: the folder of its SKILL.md, wherever its category folders put it.
export const skillOf = (name: string, dirs: readonly string[]): string =>
  dirs.find((dir) => name.startsWith(`${dir}/`)) ?? name.split("/")[0];
