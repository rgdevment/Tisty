import type { TistySkills } from "../agentsSync";
import type { Wired } from "../core";
import type { KnownAgents } from "../knownAgents";

export const knownFor = (over: Partial<KnownAgents> = {}): KnownAgents => ({
  agent: null,
  setAgent: () => {},
  agents: [],
  setAgents: () => {},
  learnPlugged: () => false,
  hands: [],
  skills: null,
  setSkills: () => {},
  lookIfOpen: () => Promise.resolve(),
  lookDoor: () => Promise.resolve(),
  lookAgents: () => Promise.resolve(),
  lookHands: () => Promise.resolve(),
  lookSkills: () => {},
  ...over,
});

export const wiredOne = (id: string, more: Partial<Wired> = {}): Wired => ({
  id,
  name: id,
  at: `/home/someone/${id}.json`,
  wired: false,
  astray: false,
  maybe: false,
  byHand: false,
  manual: { file: `/home/someone/${id}.json`, snippet: "" },
  ...more,
});

export const lostOne = (id: string, more: Partial<Wired> = {}): Wired =>
  wiredOne(id, { astray: true, gone: true, points: "/old/tisty", ...more });

export const strayHooks = (said = "/old/tisty", gone = true) => ({
  on: true,
  at: "/home/someone/hooks.json",
  stray: [{ said, gone }],
});

export const skillsFor = (more: Partial<TistySkills> = {}): TistySkills => ({
  version: "1.50.0",
  on: false,
  skills: [{ name: "tisty", label: "tisty", about: null, files: ["tisty/SKILL.md"], yours: false }],
  landing: [],
  plugged: [],
  leftover: [],
  ...more,
});
