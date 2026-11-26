import { type AgentRow, ASSISTANT_FOLDERS, ASSISTANTS } from "./agentsSync";
import type { Filed, Folded } from "./core";
import { type Drawn, grouped } from "./ui/agentRows";

const GLOBAL_LEVEL = "global";

export interface RulesOf {
  readonly level: string;
  readonly named: string;
  readonly docs: readonly Filed[];
}

export interface AgentGroups {
  /// Rules for all computers, by agent, wherever each one is kept.
  readonly rules: readonly RulesOf[];
  /// Skills for all computers, a skill of several files drawn as one.
  readonly skills: readonly Drawn[];
  /// For all computers yet in no group: drawn where it is kept, as before.
  readonly left: readonly Filed[];
  /// Agent folders at the root whose every document is drawn in a group, so they are not drawn.
  readonly emptied: ReadonlySet<string>;
}

const named = (level: string): string => ASSISTANTS.find(([id]) => id === level)?.[1] ?? level;

const rank = (level: string): number => {
  const at = ASSISTANTS.findIndex(([id]) => id === level);
  return at === -1 ? ASSISTANTS.length : at;
};

// Cursor keeps its rules in its own settings, so it has no rules file to write.
const RULELESS: ReadonlySet<string> = new Set(["cursor"]);

const levelNamed = (name: string): string | undefined =>
  ASSISTANTS.find(([, called]) => called === name)?.[0];

/// The agents this computer has a folder for, so «New rule» stands ready before any rule for all exists.
const agentsHere = (folders: readonly Folded[]): string[] =>
  folders
    .filter((one) => one.agents?.mine && one.agents.room)
    .map((one) => levelNamed(one.name))
    .filter((level): level is string => !!level && !RULELESS.has(level));

const isRule = (row: AgentRow | undefined): row is AgentRow =>
  row?.everyone === true && row.kind === "rule" && row.level !== GLOBAL_LEVEL;

const isSkill = (row: AgentRow | undefined): row is AgentRow =>
  row?.everyone === true && row.kind === "skill";

// A folder under a computer's own one says so, and so does every folder below it.
const forAll = (folder: Folded | undefined): boolean => !!folder && !folder.agents;

const below = (folders: readonly Folded[], top: string): Set<string> => {
  const all = new Set([top]);
  let grew = true;
  while (grew) {
    grew = false;
    for (const one of folders)
      if (one.parent && all.has(one.parent) && !all.has(one.id)) {
        all.add(one.id);
        grew = true;
      }
  }
  return all;
};

/// What Global draws: its three groups, built from each document's `agent:` line and never stored as folders.
export const agentGroups = (
  docs: readonly Filed[],
  folders: readonly Folded[],
  rows: ReadonlyMap<string, AgentRow>,
  root: string,
): AgentGroups => {
  const tree = below(folders, root);
  const byId = new Map(folders.map((one) => [one.id, one]));
  const shared = docs.filter(
    (one) =>
      !one.pageOf &&
      !one.away &&
      !!one.folder &&
      tree.has(one.folder) &&
      forAll(byId.get(one.folder)),
  );
  const rules = new Map<string, Filed[]>();
  const skills: Filed[] = [];
  const left: Filed[] = [];
  for (const doc of shared) {
    const row = rows.get(doc.file);
    if (isRule(row)) rules.set(row.level, [...(rules.get(row.level) ?? []), doc]);
    else if (isSkill(row)) skills.push(doc);
    else left.push(doc);
  }
  for (const level of agentsHere(folders)) if (!rules.has(level)) rules.set(level, []);
  const drawnElsewhere = new Set(left.map((one) => one.folder));
  const emptied = new Set(
    folders
      .filter((one) => one.parent === root && !one.agents && ASSISTANT_FOLDERS.has(one.name))
      .filter((one) => {
        const inside = below(folders, one.id);
        const holds = shared.some((doc) => doc.folder && inside.has(doc.folder));
        return holds && ![...inside].some((at) => drawnElsewhere.has(at));
      })
      .map((one) => one.id),
  );
  return {
    rules: [...rules]
      .sort(([one], [two]) => rank(one) - rank(two) || one.localeCompare(two))
      .map(([level, kept]) => ({ level, named: named(level), docs: kept })),
    skills: grouped(skills, rows),
    left,
    emptied,
  };
};
