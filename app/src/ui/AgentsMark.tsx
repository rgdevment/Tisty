import { useCallback, useEffect, useRef, useState } from "react";
import { type Glance, type GlanceWord, glanceWord } from "../agentDocs";
import { AGENTS_CHANGED, type AgentsState, agentsLook, agentsState } from "../agentsSync";
import { fill, t } from "../locales";
import { AGENT_DOC_STOOD } from "./useAgentDoc";

const TONE: Record<GlanceWord, string> = {
  empty: "text-faint",
  synced: "text-hue-green",
  changes: "text-hue-amber",
  waiting: "text-hue-amber",
  off: "text-faint",
};

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

export function AgentsWord({ glance }: { glance: Glance }) {
  const word = glanceWord(glance);
  const said = {
    empty: t("agentsFolderEmpty"),
    synced: t("agentsFolderSynced"),
    changes: fill("agentsFolderChanges", String(glance.changes)),
    waiting: waitingSaid(glance),
    off: t("agentsPaused"),
  }[word];
  return (
    <span className={`ml-auto shrink-0 pr-1 text-[11.5px] ${TONE[word]}`} data-agents={word}>
      {said}
    </span>
  );
}
