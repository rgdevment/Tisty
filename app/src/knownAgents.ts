import { useCallback, useEffect, useState } from "react";
import { AGENTS_CHANGED, type TistySkills, tistySkills } from "./agentsSync";
import { type Agent, type Assistant, agentState, assistants, seenAgents, type Wired } from "./core";

export function useKnownAgents(idle = false) {
  const [agent, setAgent] = useState<Agent | null>(null);
  const [agents, setAgents] = useState<Wired[] | null>(null);
  const [hands, setHands] = useState<Assistant[] | null>(null);
  const [skills, setSkills] = useState<TistySkills | null>(null);

  const lookDoor = useCallback(
    () =>
      agentState().then((fresh) => {
        setAgent(fresh);
        return fresh;
      }),
    [],
  );
  const lookAgents = useCallback(() => seenAgents().then(setAgents), []);
  const lookHands = useCallback(() => assistants().then(setHands), []);
  const lookSkills = useCallback(() => {
    tistySkills()
      .then(setSkills)
      .catch(() => undefined);
  }, []);
  const lookIfOpen = useCallback(
    () => lookDoor().then((fresh) => (fresh?.on ? lookAgents() : undefined)),
    [lookDoor, lookAgents],
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
    setAgents,
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
