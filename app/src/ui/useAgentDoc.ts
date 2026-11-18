import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { type AgentDoc, agentDoc, carded, touched } from "../agentDocs";
import { AGENTS_CHANGED } from "./AgentsKeeper";

const SETTLES = 1200;
export const AGENT_DOC_STOOD = "tisty:agent-doc-stood";

// The card reads files on disk, so it is asked again when the words settle or the window comes back, never on a clock.
export function useAgentDoc(file: string | null, body: string) {
  const [doc, setDoc] = useState<{ file: string; doc: AgentDoc } | null>(null);
  const [answered, setAnswered] = useState<string | null>(null);
  const isAgent = useMemo(() => file !== null && carded(body), [file, body]);
  // A document opens held when it is an agent's already; one whose card is being written stays open to its writer.
  const [opened, setOpened] = useState<string | null>(null);
  const [editing, setEditing] = useState<string | null>(null);
  const asked = useRef(0);
  const askedFor = useRef<string | null>(null);
  if (opened !== file) {
    setOpened(file);
    setEditing(isAgent ? null : file);
    setDoc(null);
    setAnswered(null);
    askedFor.current = null;
  }
  const seen = useRef<string | null>(null);
  const stood = useRef("");

  const ask = useCallback(() => {
    if (!file) return;
    const turn = ++asked.current;
    askedFor.current = file;
    agentDoc(file)
      .then((found) => {
        if (turn !== asked.current) return;
        const said = found ? found.places.map((one) => one.said.state).join() : "";
        // The sidebar learns of a change the card finds first, on opening or later.
        const news =
          seen.current === file ? stood.current !== said : Boolean(found?.places.some(touched));
        if (news) window.dispatchEvent(new Event(AGENT_DOC_STOOD));
        seen.current = found ? file : null;
        stood.current = said;
        setDoc(found ? { file, doc: found } : null);
        setAnswered(file);
      })
      .catch(() => {
        if (turn !== asked.current) return;
        setDoc(null);
        setAnswered(file);
      });
  }, [file]);

  const settledOn = isAgent ? body : null;
  useEffect(() => {
    if (settledOn === null) return;
    const settling = setTimeout(ask, askedFor.current === file ? SETTLES : 0);
    return () => clearTimeout(settling);
  }, [settledOn, ask, file]);

  useEffect(() => {
    if (!isAgent) return;
    window.addEventListener("focus", ask);
    window.addEventListener(AGENTS_CHANGED, ask);
    return () => {
      window.removeEventListener("focus", ask);
      window.removeEventListener(AGENTS_CHANGED, ask);
    };
  }, [isAgent, ask]);

  const shown = isAgent && doc?.file === file ? doc.doc : null;
  const writing = editing === file;
  return {
    doc: shown,
    editing: shown !== null && writing && shown.kind !== "memory",
    // Held while the card is asked for; a document the core does not take for an agent's is let go.
    held: isAgent && !writing && (answered !== file || shown !== null),
    // The agent writes its memory itself, so it is never opened for editing here.
    onEdit: (on: boolean) => setEditing(on && shown?.kind !== "memory" ? file : null),
  };
}
