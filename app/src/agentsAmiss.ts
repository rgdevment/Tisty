import { SKILLED, SKILLS_DECLINED } from "./agentsOffered";
import type { TistySkills } from "./agentsSync";
import type { Agent, Wired } from "./core";
import { t, type Word } from "./locales";
import { stored } from "./stored";

// A bare name is found on the PATH of whoever starts it, so it is never said to be broken.
const bare = (points?: string | null): boolean => !!points && !/[\\/]/.test(points);

export const connectionAmiss = (one: Wired): boolean =>
  one.astray && !one.maybe && !bare(one.points);

export const hooksAmiss = (one: Wired): boolean => !!one.hooks?.on && !!one.hooks.astray;

export const amiss = (one: Wired): boolean => connectionAmiss(one) || hooksAmiss(one);

export const lost = (one: Wired): boolean =>
  (connectionAmiss(one) && !!one.gone) || (hooksAmiss(one) && !!one.hooks?.gone);

export const onlyHooks = (one: Wired): boolean => hooksAmiss(one) && !connectionAmiss(one);

export const skillsBehind = (skills: TistySkills | null | undefined): boolean =>
  !!skills?.on && skills.landing.some((one) => one.standing === "write");

const SETTLED = new Set(["upToDate", "wrote", "write"]);

export const leftAsIs = (
  skills: TistySkills,
  of: (assistant: string) => boolean = () => true,
): string | undefined =>
  skills.landing.find((one) => of(one.assistant) && !SETTLED.has(one.standing))?.standing;

export const landingSaid = (standing: string): string => {
  const key = `tistyLanding_${standing}` as Word;
  const said = t(key);
  return said === key ? standing : said;
};

export interface Extras {
  hooks: Wired[];
  skills: Wired[];
}

export interface Offering {
  agent: Agent | null;
  agents: Wired[] | null;
  skills: TistySkills | null;
  declined: readonly string[];
  skillsHidden: boolean;
}

export function extrasOf(all: Offering): Extras | null {
  const { agent, agents, skills, declined } = all;
  if (!agent?.on || !agents || agents.some(amiss) || skillsBehind(skills)) return null;
  const joined = agents.filter((one) => one.wired && !one.astray && !declined.includes(one.id));
  const teaches =
    !!skills &&
    !skills.on &&
    !all.skillsHidden &&
    skills.skills.length > 0 &&
    !declined.includes(SKILLS_DECLINED);
  const hooks = joined.filter((one) => one.hooks && !one.hooks.on && !one.hooks.plugged);
  const taught = joined.filter(
    (one) => teaches && one.id in SKILLED && !skills.plugged.includes(SKILLED[one.id]),
  );
  const extras = { hooks: agent.hooks === false ? [] : hooks, skills: taught };
  return extras.hooks.length + extras.skills.length > 0 ? extras : null;
}

export const extrasIds = (all: Extras): string[] => [
  ...new Set([...all.hooks, ...all.skills].map((one) => one.id)),
  ...(all.skills.length > 0 ? [SKILLS_DECLINED] : []),
];

const printed = (one: Wired): string =>
  [one.id, one.points ?? "", one.hooks?.points ?? ""].join("|");

export const hushedHere = stored<{ said: string[] }>("tisty.agents.hushed", (kept) => ({
  said: Array.isArray(kept.said) ? kept.said.filter((one) => typeof one === "string") : [],
}));

export const hushAmiss = (all: readonly Wired[]) =>
  hushedHere.put({
    said: [...new Set([...hushedHere.current().said, ...all.filter(lost).map(printed)])],
  });

export const unheeded = (all: readonly Wired[] | null, hushed: readonly string[]): Wired[] =>
  (all ?? []).filter((one) => lost(one) && !hushed.includes(printed(one)));
