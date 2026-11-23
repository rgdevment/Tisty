import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useRef, useState } from "react";
import {
  AGENTS_CHANGED,
  AGENTS_FIRST,
  AGENTS_MORE,
  AGENTS_RESCAN,
  AGENTS_SYNC,
  agentsEmpty,
  agentsLit,
  agentsLook,
  agentsPrint,
  agentsRoot,
  agentsRoots,
  agentsRound,
  agentsState,
  agentsTree,
  agentsTurn,
  inAgents,
  onlySwitchedOff,
  tistySkillsRound,
} from "../agentsSync";
import { useDesk } from "../desk";
import { agentsKeys, show, useHidden } from "../hiding";
import { saidPlainly } from "../refusal";
import AgentsFirst from "./AgentsFirst";
import { useAgentsState } from "./AgentsMark";
import { waitingOf } from "./AgentsReview";

export { AGENTS_CHANGED, askAgents } from "../agentsSync";

const SETTLE_MS = 800;

type Asking = "first" | "more" | "rescan" | null;

export default function AgentsKeeper() {
  const { papers, here, setHere, papersChanged, setError } = useDesk();
  const [asking, setAsking] = useState<Asking>(null);
  const entered = useRef<string | null>(null);

  const changed = useCallback(() => {
    papersChanged();
    window.dispatchEvent(new Event(AGENTS_CHANGED));
  }, [papersChanged]);

  // «Sync» needs this computer's yes, which only the first time asks for; syncing also means seen.
  const sync = useCallback(() => {
    agentsState()
      .then((state) => {
        if (agentsLit(state)) return;
        if (!onlySwitchedOff(state)) return setAsking("first");
        return agentsTurn(null, true).then((round) => {
          show(...agentsKeys(state.folder));
          changed();
          // What syncing finds on both sides, or left behind, is asked about at once.
          if (waitingOf(round).length > 0) setAsking((now) => now ?? "first");
        });
      })
      .catch((e) => setError(saidPlainly(e)));
  }, [changed, setError]);

  useEffect(() => {
    window.addEventListener(AGENTS_SYNC, sync);
    return () => window.removeEventListener(AGENTS_SYNC, sync);
  }, [sync]);

  useEffect(() => {
    const first = () => setAsking("first");
    const more = () => setAsking("more");
    const rescan = () => setAsking("rescan");
    window.addEventListener(AGENTS_FIRST, first);
    window.addEventListener(AGENTS_MORE, more);
    window.addEventListener(AGENTS_RESCAN, rescan);
    return () => {
      window.removeEventListener(AGENTS_FIRST, first);
      window.removeEventListener(AGENTS_MORE, more);
      window.removeEventListener(AGENTS_RESCAN, rescan);
    };
  }, []);

  // Opening the app is when a new release reaches Tisty's own skills, with or without Agents.
  useEffect(() => {
    tistySkillsRound()
      .then(() => window.dispatchEvent(new Event(AGENTS_CHANGED)))
      .catch(() => undefined);
  }, []);

  // An assistant's proposal changes no document the sidebar lists, so the log stirring is the only sign of it.
  useEffect(() => {
    const stirred = listen("stirred", () => window.dispatchEvent(new Event(AGENTS_CHANGED)));
    return () => {
      stirred.then((off) => off()).catch(() => undefined);
    };
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
      .then((look) => look.asks && setAsking((now) => now ?? "first"))
      .catch(() => undefined);
  }, [inside, papers.folders]);

  const root = agentsRoot(papers.folders)?.id;
  const tree = agentsTree(papers.folders);
  // A hidden folder that syncs came from before hiding needed it unsynced; it shows again rather than write unseen.
  const stance = useAgentsState(root ?? null);
  const hidden = useHidden();
  const veiled = JSON.stringify(
    [...agentsKeys(root), ...tree].filter((key) => hidden.includes(key)),
  );
  useEffect(() => {
    const keys: string[] = JSON.parse(veiled);
    if (keys.length === 0 || !stance || !agentsLit(stance)) return;
    // The state held may be from before hiding stopped the sync, so it is asked again.
    agentsState()
      .then((now) => agentsLit(now) && show(...keys))
      .catch(() => undefined);
  }, [veiled, stance]);
  const holds = papers.docs.some((one) => one.folder && tree.has(one.folder));
  // Only an AGENTS emptied while the window watched goes; one made or found empty stays.
  const held = useRef<string | null>(null);
  const leaving = useRef<string | null>(null);
  const letGo = useCallback(() => {
    agentsEmpty()
      .then((gone) => {
        if (!gone) return;
        leaving.current = null;
        if (inside) setHere(null);
        papersChanged();
      })
      .catch(() => undefined);
  }, [inside, setHere, papersChanged]);
  useEffect(() => {
    if (!root) {
      held.current = null;
      leaving.current = null;
      return;
    }
    if (holds) {
      held.current = root;
      leaving.current = null;
      return;
    }
    if (held.current !== root) return;
    held.current = null;
    // The files its last rules wrote are asked about first; AGENTS goes once nothing waits.
    agentsRound()
      .then((round) => {
        const waits = round
          ? waitingOf({
              both: round.both ?? [],
              loose: round.loose ?? [],
              unled: round.unled ?? [],
            })
          : [];
        if (waits.length === 0) return letGo();
        leaving.current = root;
        setAsking((now) => now ?? "first");
      })
      .catch(letGo);
  }, [root, holds, letGo]);

  // An AGENTS left empty still needs a round, and so does a second one arriving to be joined.
  const roots = agentsRoots(papers.folders)
    .map((one) => one.id)
    .join(",");
  const print = root ? `${roots}:${agentsPrint(papers.docs, papers.folders)}` : "";
  useEffect(() => {
    if (!print) return;
    const later = window.setTimeout(() => {
      agentsRound()
        .then((round) => {
          // A rule taken out of AGENTS leaves its file behind, so the person is asked about it right away.
          if (round?.loose?.length) setAsking((now) => now ?? "first");
          window.dispatchEvent(new Event(AGENTS_CHANGED));
        })
        .catch((e) => setError(saidPlainly(e)));
    }, SETTLE_MS);
    return () => window.clearTimeout(later);
  }, [print, setError]);

  if (!asking) return null;
  return (
    <AgentsFirst
      more={asking === "more" || asking === "rescan"}
      rescan={asking === "rescan"}
      onClose={() => {
        setAsking(null);
        window.dispatchEvent(new Event(AGENTS_CHANGED));
        if (leaving.current === root && !holds) letGo();
      }}
      onChanged={changed}
    />
  );
}
