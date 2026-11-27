import { SKILLED, takesHooks } from "./agentsOffered";
import type { TistySkills } from "./agentsSync";
import type { Agent, Wired } from "./core";

export interface HookLine {
  readonly id: string;
  readonly name: string;
  readonly on: boolean;
}

export const tistyHookLines = (agents: readonly Wired[] | null): HookLine[] =>
  (agents ?? [])
    .filter((one) => one.wired && !!one.hooks)
    .map((one) => ({
      id: one.id,
      name: one.name,
      on: !!(one.hooks?.on || one.hooks?.plugged),
    }));

export type Dot = "on" | "partly" | "off";

export interface OwnSkills {
  readonly on: boolean;
  readonly adds: boolean;
}

export interface OwnHooks {
  readonly lines: HookLine[];
  readonly adds: Wired[];
  readonly muted: boolean;
}

export interface Own {
  readonly skills: OwnSkills | null;
  readonly hooks: OwnHooks | null;
  readonly dot: Dot;
}

interface Seen {
  agent: Agent | null | undefined;
  agents: readonly Wired[] | null;
  skills: TistySkills | null | undefined;
}

const here = (agents: readonly Wired[] | null) =>
  (agents ?? []).filter((one) => one.wired && !one.astray && !one.place);

function skillsOf(agents: readonly Wired[], skills: TistySkills | null | undefined) {
  if (!skills || skills.skills.length === 0) return null;
  const wanted = agents.filter((one) => one.id in SKILLED);
  const plugged =
    wanted.length > 0 && wanted.every((one) => skills.plugged.includes(SKILLED[one.id]));
  const on = skills.on || plugged;
  if (!on && skills.landing.length === 0) return null;
  return { on, adds: !on && skills.landing.length > 0 };
}

function hooksOf(agents: readonly Wired[], agent: Agent | null | undefined): OwnHooks | null {
  const lines = tistyHookLines(agents);
  if (lines.length === 0) return null;
  const muted = agent?.hooks === false;
  const held = muted || agent?.on === false;
  const adds = held
    ? []
    : agents.filter(
        (one) => takesHooks(one) && !one.hooks?.pluginUnknown && !one.hooks?.stray?.length,
      );
  return { lines, adds, muted };
}

const dotOf = (skills: OwnSkills | null, hooks: OwnHooks | null): Dot => {
  const hooksOn = hooks && !hooks.muted ? hooks.lines.filter((one) => one.on).length : 0;
  const whole =
    (!skills || skills.on) && (!hooks || (!hooks.muted && hooksOn === hooks.lines.length));
  if (whole) return "on";
  return skills?.on || hooksOn > 0 ? "partly" : "off";
};

export function ownOf({ agent, agents, skills }: Seen): Own {
  const joined = here(agents);
  const taught = skillsOf(joined, skills);
  const hooks = hooksOf(joined, agent);
  return { skills: taught, hooks, dot: dotOf(taught, hooks) };
}

export const shown = (own: Own): boolean => own.skills !== null || own.hooks !== null;

export const adds = (own: Own): boolean =>
  (own.skills?.adds ?? false) || (own.hooks?.adds.length ?? 0) > 0;
