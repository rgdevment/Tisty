import type { ButtonHTMLAttributes, ReactNode } from "react";
import type { AgentGroups } from "../agentGroups";
import type { Filed, Wired } from "../core";
import { fill, t, type Word } from "../locales";
import type { SkillGroup } from "./agentRows";
import Glyph from "./Glyph";
import HooksRow, { tistyHookLines } from "./HooksRow";
import { TistySkillsRow } from "./TistySkills";

/// What makes a button one row of the tree: reached by the arrow keys, never a folder nor a place to drop.
export type RowOf = (key: string) => ButtonHTMLAttributes<HTMLButtonElement>;

interface Props {
  groups: AgentGroups;
  depth: number;
  step: number;
  paper: (doc: Filed, depth: number) => ReactNode;
  skill: (group: SkillGroup, depth: number) => ReactNode;
  rowOf: RowOf;
  tistySkills: boolean;
  agents: readonly Wired[] | null;
  onHooks?: () => void;
  onNewRule?: (level: string) => void;
}

function Heading({ label, depth, step }: Readonly<{ label: Word; depth: number; step: number }>) {
  return (
    <li
      className="pt-1.5 pb-0.5 text-[10.5px] font-semibold tracking-[0.06em] text-faint uppercase"
      style={{ paddingLeft: `${8 + depth * step + 4}px` }}
    >
      {t(label)}
    </li>
  );
}

function AgentHeading({
  named,
  level,
  depth,
  step,
  rowOf,
  onNewRule,
}: Readonly<{
  named: string;
  level: string;
  depth: number;
  step: number;
  rowOf: RowOf;
  onNewRule?: (level: string) => void;
}>) {
  const asked = fill("agentGroupNewRuleOf", named);
  return (
    <li
      className="group/row flex items-center py-0.5 text-[12.5px] text-soft"
      style={{ paddingLeft: `${8 + depth * step + 4}px` }}
    >
      <span className="truncate">{named}</span>
      {onNewRule && (
        <button
          type="button"
          {...rowOf(`rules:${level}`)}
          onClick={() => onNewRule(level)}
          aria-label={asked}
          title={asked}
          className="ml-auto mr-1 grid h-5 w-5 place-items-center rounded-md text-faint hover:bg-hover hover:text-ink"
        >
          <Glyph name="plus" className="h-[12px] w-[12px]" />
        </button>
      )}
    </li>
  );
}

/// Whether Global's groups draw any row at all, so an empty Global can still say so.
export const groupsDrawn = (
  groups: AgentGroups,
  tistySkills: boolean,
  agents: readonly Wired[] | null,
  hooks: boolean,
): boolean =>
  groups.rules.length > 0 ||
  groups.skills.length > 0 ||
  tistySkills ||
  (hooks && tistyHookLines(agents).length > 0);

/// Global as three groups drawn from each document's `agent:` line: nothing here is a folder, nothing moves.
export default function AgentGroupRows({
  groups,
  depth,
  step,
  paper,
  skill,
  rowOf,
  tistySkills,
  agents,
  onHooks,
  onNewRule,
}: Readonly<Props>) {
  const showsSkills = groups.skills.length > 0 || tistySkills;
  return (
    <>
      {groups.rules.length > 0 && <Heading label="agentGroupRules" depth={depth} step={step} />}
      {groups.rules.map((one) => (
        <li key={`rules:${one.level}`}>
          <ul>
            <AgentHeading
              named={one.named}
              level={one.level}
              depth={depth}
              step={step}
              rowOf={rowOf}
              onNewRule={onNewRule}
            />
            {one.docs.map((doc) => paper(doc, depth + 1))}
          </ul>
        </li>
      ))}
      {showsSkills && <Heading label="agentGroupSkills" depth={depth} step={step} />}
      {groups.skills.map((one) => ("doc" in one ? paper(one.doc, depth) : skill(one.skill, depth)))}
      {tistySkills && <TistySkillsRow depth={depth} step={step} rowOf={rowOf} />}
      {tistyHookLines(agents).length > 0 && onHooks && (
        <>
          <Heading label="agentGroupHooks" depth={depth} step={step} />
          <li style={{ paddingLeft: `${8 + depth * step}px` }}>
            <HooksRow agents={agents} onOpen={onHooks} rowOf={rowOf} />
          </li>
        </>
      )}
    </>
  );
}
