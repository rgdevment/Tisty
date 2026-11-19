import { useCallback, useEffect, useRef, useState } from "react";
import { type Glance, type GlanceWord, glanceWord } from "../agentDocs";
import { agentsLook } from "../agentsSync";
import { fill, t } from "../locales";
import { AGENTS_CHANGED } from "./AgentsKeeper";
import { AGENT_DOC_STOOD } from "./useAgentDoc";

const TONE: Record<GlanceWord, string> = {
  empty: "text-faint",
  synced: "text-hue-green",
  changes: "text-hue-amber",
  off: "text-faint",
};

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

export function AgentsWord({ glance }: { glance: Glance }) {
  const word = glanceWord(glance);
  const said = {
    empty: t("agentsFolderEmpty"),
    synced: t("agentsFolderSynced"),
    changes: fill("agentsFolderChanges", String(glance.changes)),
    off: t("agentsPaused"),
  }[word];
  return (
    <span className={`ml-auto shrink-0 pr-1 text-[11.5px] ${TONE[word]}`} data-agents={word}>
      {said}
    </span>
  );
}
