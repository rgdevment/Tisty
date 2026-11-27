import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { forgetHealed } from "./agentsAmiss";
import { AGENTS_CHANGED, type TistySkills, tistySkills } from "./agentsSync";
import {
  type Agent,
  type Assistant,
  agentState,
  assistants,
  seenAgents,
  type Wired,
  watchedAgents,
  wslWatched,
} from "./core";
import { looked } from "./wslLooked";

// Writing Claude Code's hooks is what finds its plugin out, so the next look at them already says it.
const broughtTo = (one: Wired, brought: readonly string[]): Wired =>
  one.hooks && brought.includes(one.id) ? { ...one, hooks: { ...one.hooks, plugged: true } } : one;

// A look with no click does not open the plugin's records, so what the page already found out stands.
const foundOut = (one: Wired, was: readonly Wired[] | null): Wired => {
  const before = was?.find((it) => it.id === one.id)?.hooks;
  if (!one.hooks?.pluginUnknown || !before || before.pluginUnknown) return one;
  return { ...one, hooks: { ...one.hooks, plugged: before.plugged, pluginUnknown: false } };
};

export function useKnownAgents(idle = false) {
  const [agent, setAgent] = useState<Agent | null>(null);
  const [seen, keepAgents] = useState<Wired[] | null>(null);
  const [inDistros, setInDistros] = useState<Wired[]>([]);
  const [brought, setBrought] = useState<string[]>([]);
  const [hands, setHands] = useState<Assistant[] | null>(null);
  const [skills, setSkills] = useState<TistySkills | null>(null);

  const setAgents = useCallback((all: Wired[]) => {
    forgetHealed(all);
    keepAgents(all);
  }, []);
  const learnPlugged = useCallback((ids: string[], gone: string[] = []) => {
    if (ids.length + gone.length === 0) return;
    setBrought((was) => [...new Set([...was, ...ids])].filter((id) => !gone.includes(id)));
  }, []);
  const agents = useMemo(
    () => seen?.map((one) => broughtTo(one, brought)) ?? null,
    [seen, brought],
  );
  // The startup look also reaches the distros already looked into, so one left behind by an install is said too.
  const everywhere = useMemo(() => agents && [...agents, ...inDistros], [agents, inDistros]);
  const lastLook = useRef(0);
  const lookDistros = useCallback(() => {
    const asked = ++lastLook.current;
    wslWatched(looked())
      .then((found) => {
        if (asked !== lastLook.current) return;
        forgetHealed(found);
        setInDistros(found);
      })
      .catch(() => undefined);
  }, []);
  const lookDoor = useCallback(() => agentState().then(setAgent), []);
  const lookAgents = useCallback(() => {
    lookDistros();
    return seenAgents().then(setAgents);
  }, [setAgents, lookDistros]);
  const lookHands = useCallback(() => assistants().then(setHands), []);
  const lookSkills = useCallback(() => {
    tistySkills()
      .then(setSkills)
      .catch(() => undefined);
  }, []);
  const lookIfOpen = useCallback(
    () =>
      watchedAgents().then((watched) => {
        if (!watched) return;
        setAgent(watched.agent);
        const { seen } = watched;
        if (!seen) return;
        forgetHealed(seen);
        keepAgents((was) => seen.map((one) => foundOut(one, was)));
        lookDistros();
      }),
    [lookDistros],
  );

  // Opening the app brings the skills up to date first, so they are read once that round has said so.
  useEffect(() => {
    if (idle) return;
    window.addEventListener(AGENTS_CHANGED, lookSkills);
    return () => window.removeEventListener(AGENTS_CHANGED, lookSkills);
  }, [idle, lookSkills]);

  return {
    agent,
    setAgent,
    agents,
    everywhere,
    inDistros,
    lookDistros,
    setAgents,
    learnPlugged,
    hands,
    skills,
    setSkills,
    lookIfOpen,
    lookDoor,
    lookAgents,
    lookHands,
    lookSkills,
  };
}

export type KnownAgents = ReturnType<typeof useKnownAgents>;
