import { useEffect, useMemo, useRef, useState } from "react";
import type { AgentKind } from "../agentDocs";
import { type AgentRow, agentsPrint, agentsRows, agentsTree } from "../agentsSync";
import type { Filed, Papers } from "../core";
import { skillDirs, skillOf } from "../skillFolders";

export const KIND_GLYPH: Record<AgentKind, string> = {
  rule: "book-text",
  skill: "puzzle",
  memory: "brain",
};

const SKILL_FILE = "skill.md";

export interface SkillGroup {
  readonly key: string;
  readonly label: string;
  readonly docs: readonly Filed[];
}

export type Drawn = { readonly doc: Filed } | { readonly skill: SkillGroup };

type Gathering = { key: string; label: string; docs: Filed[] };

const isSkillFile = (row: AgentRow) =>
  row.skill !== null && row.name.toLowerCase() === `${row.skill}/${SKILL_FILE}`.toLowerCase();

export const inSkill = (row: AgentRow): string =>
  row.skill && row.name.startsWith(`${row.skill}/`)
    ? row.name.slice(row.skill.length + 1)
    : row.name;

export const grouped = (docs: readonly Filed[], rows: ReadonlyMap<string, AgentRow>): Drawn[] => {
  const groups = new Map<string, Gathering>();
  const order: (Filed | Gathering)[] = [];
  for (const doc of docs) {
    const row = rows.get(doc.file);
    if (!row?.skill) {
      order.push(doc);
      continue;
    }
    const key = `skill:${doc.folder ?? ""}:${row.level}:${row.skill}`;
    let group = groups.get(key);
    if (!group) {
      group = { key, label: row.skill, docs: [] };
      groups.set(key, group);
      order.push(group);
    }
    if (isSkillFile(row)) {
      group.label = row.label;
      group.docs.unshift(doc);
    } else group.docs.push(doc);
  }
  return order.map((one) => {
    if (!("docs" in one)) return { doc: one };
    return one.docs.length === 1 ? { doc: one.docs[0] } : { skill: one };
  });
};

const NONE: ReadonlyMap<string, AgentRow> = new Map();

/// A skill inside a category folder belongs to the folder of its SKILL.md, not to the category.
export const nested = (rows: ReadonlyMap<string, AgentRow>): ReadonlyMap<string, AgentRow> => {
  const skills = [...rows.values()].filter((row) => row.skill !== null);
  const dirsOf = new Map<string, string[]>();
  for (const level of new Set(skills.map((row) => row.level)))
    dirsOf.set(
      level,
      skillDirs(skills.filter((row) => row.level === level).map((row) => row.name)),
    );
  const placed = new Map(rows);
  for (const [file, row] of rows) {
    if (row.skill === null) continue;
    const skill = skillOf(row.name, dirsOf.get(row.level) ?? []);
    if (skill === row.skill) continue;
    const own = row.name.toLowerCase() === `${skill}/${SKILL_FILE}`.toLowerCase();
    const label = own && row.label === row.skill ? (skill.split("/").pop() ?? skill) : row.label;
    placed.set(file, { ...row, skill, label });
  }
  return placed;
};

const versionOf = (one: Filed) => `${one.wrote ?? ""}:${one.bytes ?? ""}`;

const underAgents = (papers: Papers): ReadonlyMap<string, string> => {
  const tree = agentsTree(papers.folders);
  return new Map(
    papers.docs
      .filter((one) => one.folder && tree.has(one.folder))
      .map((one) => [one.file, versionOf(one)]),
  );
};

// Only the documents under AGENTS that changed are read again, so a person without AGENTS pays nothing.
export function useAgentRows(papers: Papers): ReadonlyMap<string, AgentRow> {
  const print = useMemo(() => agentsPrint(papers.docs, papers.folders), [papers]);
  const here = useMemo(() => underAgents(papers), [papers]);
  const hereNow = useRef(here);
  hereNow.current = here;
  const asked = useRef(new Map<string, string>());
  const [held, setHeld] = useState<ReadonlyMap<string, AgentRow>>(NONE);
  useEffect(() => {
    if (!print) return;
    const now = hereNow.current;
    const changed = [...now].filter(([file, version]) => asked.current.get(file) !== version);
    if (changed.length === 0) return;
    let gone = false;
    agentsRows(changed.map(([file]) => file))
      .then((answer) => {
        if (gone) return;
        for (const [file, version] of changed) asked.current.set(file, version);
        setHeld((was) => {
          const next = new Map(was);
          for (const [file] of changed) next.delete(file);
          for (const one of answer) next.set(one.file, one);
          return next;
        });
      })
      .catch(() => {});
    return () => {
      gone = true;
    };
  }, [print]);
  return useMemo(
    () => (here.size === 0 ? NONE : nested(new Map([...held].filter(([file]) => here.has(file))))),
    [held, here],
  );
}
