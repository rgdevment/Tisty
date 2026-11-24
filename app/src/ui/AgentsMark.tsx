import { useCallback, useEffect, useRef, useState } from "react";
import {
  AGENTS_CHANGED,
  type AgentsStage,
  type AgentsState,
  agentsRoot,
  agentsStage,
  agentsState,
  tistySkills,
} from "../agentsSync";
import type { Papers } from "../core";

// Only whether each assistant's folder exists, so it may ask before the person ever said yes; without a folder AGENTS is off anyway.
export function useAgentsState(root: string | null): AgentsState | null {
  const [state, setState] = useState<AgentsState | null>(null);
  const asked = useRef(0);
  const ask = useCallback(() => {
    const turn = ++asked.current;
    agentsState()
      .then((now) => turn === asked.current && setState(now))
      .catch(() => turn === asked.current && setState(null));
  }, []);
  useEffect(() => {
    if (root) ask();
  }, [ask, root]);
  useEffect(() => {
    if (!root) return;
    window.addEventListener(AGENTS_CHANGED, ask);
    return () => window.removeEventListener(AGENTS_CHANGED, ask);
  }, [ask, root]);
  return root ? state : null;
}

/// Tisty's skills have a row only where some agent would read them, or where there is something of theirs to turn off.
export function useSkillsRow(stance: AgentsState | null): boolean {
  const agents = stance === null ? null : stance.assistants.length > 0;
  const [left, setLeft] = useState(false);
  useEffect(() => {
    if (agents !== false) return setLeft(false);
    let gone = false;
    const ask = () => {
      tistySkills()
        .then((one) => !gone && setLeft(one.on || one.leftover.length > 0))
        .catch(() => {});
    };
    ask();
    window.addEventListener(AGENTS_CHANGED, ask);
    return () => {
      gone = true;
      window.removeEventListener(AGENTS_CHANGED, ask);
    };
  }, [agents]);
  return agents === true || left;
}

export interface AgentsSeen {
  root: string | null;
  stance: AgentsState | null;
  stage: AgentsStage | null;
}

export function useAgentsStage(papers: Papers): AgentsSeen {
  const root = agentsRoot(papers.folders)?.id ?? null;
  const stance = useAgentsState(root);
  return {
    root,
    stance,
    stage: agentsStage(root, stance),
  };
}
