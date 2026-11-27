import { type ButtonHTMLAttributes, type ReactNode, useRef, useState } from "react";
import type { AgentGroups, RuleTarget } from "../agentGroups";
import type { Filed, Wired } from "../core";
import { fill, t, type Word } from "../locales";
import type { SkillGroup } from "./agentRows";
import Glyph from "./Glyph";
import TistyHooksRow, { tistyHookLines } from "./HooksRow";
import Menu, { type Choice } from "./Menu";
import type { Anchor } from "./menuKeys";
import { TistySkillsRow } from "./TistySkills";
import UserHookRows from "./UserHooks";

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
  onNewHook?: () => void;
}

function Heading({
  label,
  depth,
  step,
  add,
}: Readonly<{ label: Word; depth: number; step: number; add?: ReactNode }>) {
  return (
    <li
      className="flex items-center pt-1.5 pb-0.5 text-[10.5px] font-semibold tracking-[0.06em] text-faint uppercase"
      style={{ paddingLeft: `${8 + depth * step + 4}px` }}
    >
      {t(label)}
      {add}
    </li>
  );
}

function AddButton({
  label,
  rowKey,
  rowOf,
  onAdd,
}: Readonly<{
  label: string;
  rowKey: string;
  rowOf: RowOf;
  onAdd: (button: HTMLButtonElement) => void;
}>) {
  return (
    <button
      type="button"
      {...rowOf(rowKey)}
      onClick={(e) => onAdd(e.currentTarget)}
      aria-label={label}
      title={label}
      className="ml-auto mr-1 grid h-5 w-5 place-items-center rounded-md text-faint normal-case hover:bg-hover hover:text-ink"
    >
      <Glyph name="plus" className="h-[12px] w-[12px]" />
    </button>
  );
}

const REOPENS_MS = 200;

/// The one «+» of Rules: straight to the rule when a single agent can take one, else a menu of agents.
function NewRuleButton({
  targets,
  rowOf,
  onNewRule,
}: Readonly<{ targets: readonly RuleTarget[]; rowOf: RowOf; onNewRule: (level: string) => void }>) {
  const [at, setAt] = useState<Anchor | null>(null);
  const shutAt = useRef(Number.NEGATIVE_INFINITY);
  const only = targets.length === 1 ? targets[0] : null;
  const label = only ? fill("agentGroupNewRuleOf", only.named) : t("agentGroupNewRule");
  const open = (button: HTMLElement) => {
    if (only) return onNewRule(only.level);
    if (performance.now() - shutAt.current < REOPENS_MS) return;
    const box = button.getBoundingClientRect();
    setAt((was) => (was ? null : { x: box.left, y: box.bottom + 4, top: box.top }));
  };
  const choices: Choice[] = targets.map((one) => ({
    key: one.level,
    label: one.named,
    onPick: () => onNewRule(one.level),
  }));
  return (
    <>
      <AddButton label={label} rowKey="rules:new" rowOf={rowOf} onAdd={open} />
      {at && (
        <Menu
          at={at}
          choices={choices}
          label={label}
          onClose={() => {
            shutAt.current = performance.now();
            setAt(null);
          }}
        />
      )}
    </>
  );
}

function AgentHeading({
  named,
  depth,
  step,
}: Readonly<{ named: string; depth: number; step: number }>) {
  return (
    <li
      className="truncate py-0.5 text-[12.5px] text-soft"
      style={{ paddingLeft: `${8 + depth * step + 4}px` }}
    >
      {named}
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
  groups.ruleTargets.length > 0 ||
  groups.skills.length > 0 ||
  groups.hooks.length > 0 ||
  tistySkills ||
  (hooks && tistyHookLines(agents).length > 0);

/// Global drawn from each document's `agent:` line: first what the person wrote, then what Tisty brings.
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
  onNewHook,
}: Readonly<Props>) {
  const showsRules = groups.rules.length > 0 || (!!onNewRule && groups.ruleTargets.length > 0);
  const showsHooks = groups.hooks.length > 0 || !!onNewHook;
  const tistyHooks = tistyHookLines(agents).length > 0 && !!onHooks;
  return (
    <>
      {showsRules && (
        <Heading
          label="agentGroupRules"
          depth={depth}
          step={step}
          add={
            onNewRule &&
            groups.ruleTargets.length > 0 && (
              <NewRuleButton targets={groups.ruleTargets} rowOf={rowOf} onNewRule={onNewRule} />
            )
          }
        />
      )}
      {groups.rules.map((one) => (
        <li key={`rules:${one.level}`}>
          <ul>
            <AgentHeading named={one.named} depth={depth} step={step} />
            {one.docs.map((doc) => paper(doc, depth + 1))}
          </ul>
        </li>
      ))}
      {groups.skills.length > 0 && <Heading label="agentGroupSkills" depth={depth} step={step} />}
      {groups.skills.map((one) => ("doc" in one ? paper(one.doc, depth) : skill(one.skill, depth)))}
      {showsHooks && (
        <Heading
          label="agentGroupHooks"
          depth={depth}
          step={step}
          add={
            onNewHook && (
              <AddButton
                label={t("agentHookNew")}
                rowKey="hooks:new"
                rowOf={rowOf}
                onAdd={onNewHook}
              />
            )
          }
        />
      )}
      <UserHookRows hooks={groups.hooks} depth={depth} paper={paper} rowOf={rowOf} />
      {tistyHooks && <Heading label="agentGroupTisty" depth={depth} step={step} />}
      {tistySkills && (
        <TistySkillsRow
          depth={depth}
          step={step}
          rowOf={rowOf}
          lead={!tistyHooks && <Heading label="agentGroupTisty" depth={depth} step={step} />}
        />
      )}
      {tistyHooks && onHooks && (
        <TistyHooksRow agents={agents} depth={depth} step={step} onOpen={onHooks} rowOf={rowOf} />
      )}
    </>
  );
}
