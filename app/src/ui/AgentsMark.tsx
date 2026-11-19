import { useCallback, useEffect, useRef, useState } from "react";
import { asksPerson, type Glance } from "../agentDocs";
import {
  AGENTS_CHANGED,
  type AgentsStage,
  type AgentsState,
  agentsHeld,
  agentsLook,
  agentsRoot,
  agentsStage,
  agentsState,
} from "../agentsSync";
import type { Papers } from "../core";
import { fill } from "../locales";
import { AGENT_DOC_STOOD } from "./useAgentDoc";

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
    stage: agentsStage(root, stance, agentsHeld(papers.docs, papers.folders)),
  };
}

// Looking reads the assistants' folders, so it waits for a round, the window's return or AGENTS appearing.
export function useAgentsGlance(root: string | null): Glance | null {
  const [glance, setGlance] = useState<Glance | null>(null);
  const asked = useRef(0);
  const ask = useCallback(() => {
    const turn = ++asked.current;
    const answered = (now: Glance | null) => {
      if (turn === asked.current) setGlance(now);
    };
    agentsLook()
      .then(answered)
      .catch(() => answered(null));
  }, []);
  useEffect(() => {
    if (root) ask();
  }, [ask, root]);
  useEffect(() => {
    if (!root) return;
    window.addEventListener("focus", ask);
    window.addEventListener(AGENTS_CHANGED, ask);
    window.addEventListener(AGENT_DOC_STOOD, ask);
    return () => {
      window.removeEventListener("focus", ask);
      window.removeEventListener(AGENTS_CHANGED, ask);
      window.removeEventListener(AGENT_DOC_STOOD, ask);
    };
  }, [ask, root]);
  return root ? glance : null;
}

const waitingSaid = (glance: Glance): string => {
  if (glance.waiting > 0) return fill("agentsFolderWaiting", String(glance.waiting));
  if (glance.unagreed > 0) return fill("agentsFolderUnagreed", String(glance.unagreed));
  return fill("agentsFolderProposed", String(glance.proposed));
};

/// Only what waits for the person has a word in the sidebar; otherwise Agents shows its count like any folder.
export function AgentsWord({ glance }: { glance: Glance }) {
  if (!asksPerson(glance)) return null;
  const said =
    glance.changes > 0 ? fill("agentsFolderChanges", String(glance.changes)) : waitingSaid(glance);
  return (
    <span className="ml-auto shrink-0 pr-1 text-[11.5px] text-hue-amber" data-agents="asks">
      {said}
    </span>
  );
}
