import { useEffect, useMemo, useState } from "react";
import type { AgentKind } from "../agentDocs";
import { type AgentRow, agentsPrint, agentsRows } from "../agentsSync";
import type { Filed, Papers } from "../core";

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

// Asked again only when a document under AGENTS changes, so a person without AGENTS pays nothing.
export function useAgentRows(papers: Papers): ReadonlyMap<string, AgentRow> {
  const print = useMemo(() => agentsPrint(papers.docs, papers.folders), [papers]);
  const [held, setHeld] = useState<ReadonlyMap<string, AgentRow>>(NONE);
  useEffect(() => {
    if (!print) return;
    let gone = false;
    agentsRows()
      .then((all) => {
        if (!gone) setHeld(new Map(all.map((one) => [one.file, one])));
      })
      .catch(() => {});
    return () => {
      gone = true;
    };
  }, [print]);
  return print ? held : NONE;
}
