import { useEffect, useRef, useState } from "react";
import {
  agentsEmpty,
  agentsLook,
  agentsPrint,
  agentsRoot,
  agentsRound,
  agentsTree,
  inAgents,
} from "../agentsSync";
import { useDesk } from "../desk";
import { saidPlainly } from "../refusal";
import AgentsFirst from "./AgentsFirst";

export const AGENTS_FIRST = "tisty:agents-first";
export const AGENTS_CHANGED = "tisty:agents-changed";

export const askAgents = () => window.dispatchEvent(new Event(AGENTS_FIRST));

const SETTLE_MS = 800;

export default function AgentsKeeper() {
  const { papers, here, setHere, papersChanged, setError } = useDesk();
  const [asking, setAsking] = useState(false);
  const entered = useRef<string | null>(null);

  useEffect(() => {
    const ask = () => setAsking(true);
    window.addEventListener(AGENTS_FIRST, ask);
    return () => window.removeEventListener(AGENTS_FIRST, ask);
  }, []);

  const inside = inAgents(here, papers.folders);
  useEffect(() => {
    if (!inside) {
      entered.current = null;
      return;
    }
    const root = agentsRoot(papers.folders)?.id ?? null;
    if (entered.current === root) return;
    entered.current = root;
    agentsLook()
      .then((look) => look.asks && setAsking(true))
      .catch(() => undefined);
  }, [inside, papers.folders]);

  const root = agentsRoot(papers.folders);
  const tree = agentsTree(papers.folders);
  const holds = papers.docs.some((one) => one.folder && tree.has(one.folder));
  const emptying = useRef<string | null>(null);
  useEffect(() => {
    if (!root || holds) {
      emptying.current = null;
      return;
    }
    if (emptying.current === root.id) return;
    emptying.current = root.id;
    agentsEmpty()
      .then((gone) => {
        if (!gone) return;
        if (inside) setHere(null);
        papersChanged();
      })
      .catch(() => undefined);
  }, [root, holds, inside, setHere, papersChanged]);

  const print = agentsPrint(papers.docs, papers.folders);
  useEffect(() => {
    if (!print) return;
    const later = window.setTimeout(() => {
      agentsRound()
        .then(() => window.dispatchEvent(new Event(AGENTS_CHANGED)))
        .catch((e) => setError(saidPlainly(e)));
    }, SETTLE_MS);
    return () => window.clearTimeout(later);
  }, [print, setError]);

  if (!asking) return null;
  const changed = () => {
    papersChanged();
    window.dispatchEvent(new Event(AGENTS_CHANGED));
  };
  return (
    <AgentsFirst
      onClose={() => {
        setAsking(false);
        window.dispatchEvent(new Event(AGENTS_CHANGED));
      }}
      onChanged={changed}
    />
  );
}
