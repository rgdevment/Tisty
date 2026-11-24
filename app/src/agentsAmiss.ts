import { hooksToken, SKILLED, SKILLS_DECLINED } from "./agentsOffered";
import type { TistySkills } from "./agentsSync";
import type { Agent, Wired } from "./core";
import { t, type Word } from "./locales";
import { stored } from "./stored";

export type State = "lost" | "elsewhere";

export interface Parts {
  connection?: string;
  hooks: string[];
}

const strayParts = (one: Wired, wanted: (gone: boolean) => boolean): Parts => ({
  connection: one.astray && wanted(!!one.gone) ? (one.points ?? "") : undefined,
  hooks: (one.hooks?.stray ?? []).filter((it) => wanted(it.gone)).map((it) => it.said),
});

export const partsIn = (one: Wired, state: State): Parts =>
  strayParts(one, (gone) => gone === (state === "lost"));

export const allParts = (one: Wired): Parts => strayParts(one, () => true);

export const some = (parts: Parts): boolean =>
  parts.connection !== undefined || parts.hooks.length > 0;

export const onlyHooks = (parts: Parts): boolean => parts.connection === undefined;

// Settings outlive an uninstall, so one that is no longer here is never said to be broken.
const present = (one: Wired): boolean => one.installed !== false && !one.maybe;

export const inState = (all: readonly Wired[] | null, state: State): Wired[] =>
  (all ?? []).filter((one) => present(one) && some(partsIn(one, state)));

export const filesOf = (one: Wired, parts: Parts): string[] => {
  const files: string[] = [];
  if (parts.connection !== undefined) files.push(one.at);
  if (parts.hooks.length > 0 && one.hooks) files.push(one.hooks.at);
  return [...new Set(files)];
};

export const filesOut = (one: Wired): string[] => {
  const hooks = one.hooks?.on ? [one.hooks.at] : [];
  return [...new Set([one.at, ...hooks])];
};

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
  if (!agent?.on || !agents || inState(agents, "lost").length > 0) return null;
  const joined = agents.filter((one) => one.wired && !declined.includes(one.id));
  const teaches =
    !!skills &&
    !skills.on &&
    !all.skillsHidden &&
    skills.landing.length > 0 &&
    !declined.includes(SKILLS_DECLINED);
  const hooks = joined.filter(
    ({ id, hooks }) =>
      hooks &&
      !hooks.on &&
      !hooks.unread &&
      !hooks.apart &&
      !hooks.plugged &&
      !hooks.pluginUnknown &&
      !declined.includes(hooksToken(id)),
  );
  const taught = joined.filter(
    (one) => teaches && one.id in SKILLED && !skills.plugged.includes(SKILLED[one.id]),
  );
  const extras = { hooks: agent.hooks === false ? [] : hooks, skills: taught };
  return extras.hooks.length + extras.skills.length > 0 ? extras : null;
}

export const extrasIds = (all: Extras): string[] => [
  ...all.hooks.map((one) => hooksToken(one.id)),
  ...(all.skills.length > 0 ? [SKILLS_DECLINED] : []),
];

const printed = (one: Wired, state: State): string => {
  const parts = partsIn(one, state);
  return [state, one.id, parts.connection ?? "", ...parts.hooks].join("|");
};

export const hushedHere = stored<{ said: string[] }>("tisty.agents.hushed", (kept) => ({
  said: Array.isArray(kept.said) ? kept.said.filter((one) => typeof one === "string") : [],
}));

const hushed = (said: string[]) => {
  if (said.length !== hushedHere.current().said.length) hushedHere.put({ said });
};

export const hush = (all: readonly Wired[], state: State) =>
  hushed([...new Set([...hushedHere.current().said, ...all.map((one) => printed(one, state))])]);

const readable = (one: Wired): boolean => present(one) && !one.unread && !one.hooks?.unread;

export const forgetHealed = (all: readonly Wired[]) => {
  const now = (["lost", "elsewhere"] as const).flatMap((state) =>
    inState(all, state).map((one) => printed(one, state)),
  );
  const told = new Set(all.filter(readable).map((one) => one.id));
  hushed(
    hushedHere.current().said.filter((one) => now.includes(one) || !told.has(one.split("|")[1])),
  );
};

export const unheeded = (
  all: readonly Wired[] | null,
  state: State,
  quiet: readonly string[],
): Wired[] => inState(all, state).filter((one) => !quiet.includes(printed(one, state)));
