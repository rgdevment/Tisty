import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useRef, useState } from "react";
import {
  AGENTS_ASLEEP,
  AGENTS_CHANGED,
  AGENTS_LIGHT,
  AGENTS_MORE,
  type AgentsState,
  agentsEmpty,
  agentsLit,
  agentsLook,
  agentsPrint,
  agentsRoot,
  agentsRound,
  agentsState,
  agentsTree,
  agentsTurn,
  inAgents,
  onlySwitchedOff,
} from "../agentsSync";
import { useDesk } from "../desk";
import { AGENTS, agentsHidden, agentsKeys, hide, show, useHidden } from "../hiding";
import { saidPlainly } from "../refusal";
import AgentsAsleep from "./AgentsAsleep";
import AgentsFirst from "./AgentsFirst";
import { useAgentsState } from "./AgentsMark";
import { waitingOf } from "./AgentsReview";

export const AGENTS_FIRST = "tisty:agents-first";
export { AGENTS_CHANGED };

export const askAgents = () => window.dispatchEvent(new Event(AGENTS_FIRST));

const SETTLE_MS = 800;

type Asking = "first" | "more" | null;

export default function AgentsKeeper() {
  const { papers, here, setHere, papersChanged, setError } = useDesk();
  const [asking, setAsking] = useState<Asking>(null);
  const [asleep, setAsleep] = useState<AgentsState | null>(null);
  const [waking, setWaking] = useState(false);
  const [stirring, setStirring] = useState<string>();
  const entered = useRef<string | null>(null);
  // Turned on from the notice: once the first time closes with AGENTS allowed, its switch goes on too.
  const lighting = useRef(false);

  const changed = useCallback(() => {
    papersChanged();
    window.dispatchEvent(new Event(AGENTS_CHANGED));
  }, [papersChanged]);

  // On means seen: AGENTS never writes agent files from behind a hidden row.
  const switchOn = useCallback(
    (folder: string | null) => {
      show(...agentsKeys(folder));
      return agentsTurn(null, true).then(changed);
    },
    [changed],
  );

  const light = useCallback(
    (state: AgentsState) => {
      if (onlySwitchedOff(state)) return switchOn(state.folder);
      lighting.current = true;
      setAsking("first");
      return Promise.resolve();
    },
    [switchOn],
  );

  useEffect(() => {
    // A row still learning its state may ask while AGENTS is on; then there is nothing to wake.
    const notice = () => {
      setStirring(undefined);
      agentsState()
        .then((state) => !agentsLit(state) && setAsleep(state))
        .catch((e) => setError(saidPlainly(e)));
    };
    const now = () => {
      agentsState()
        .then(light)
        .catch((e) => setError(saidPlainly(e)));
    };
    window.addEventListener(AGENTS_ASLEEP, notice);
    window.addEventListener(AGENTS_LIGHT, now);
    return () => {
      window.removeEventListener(AGENTS_ASLEEP, notice);
      window.removeEventListener(AGENTS_LIGHT, now);
    };
  }, [light, setError]);

  useEffect(() => {
    const first = () => setAsking("first");
    const more = () => setAsking("more");
    window.addEventListener(AGENTS_FIRST, first);
    window.addEventListener(AGENTS_MORE, more);
    return () => {
      window.removeEventListener(AGENTS_FIRST, first);
      window.removeEventListener(AGENTS_MORE, more);
    };
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
  // Hidden while on came from before hiding needed AGENTS off; it shows again rather than go off unseen.
  const stance = useAgentsState(root ?? null);
  const veiled = agentsHidden(useHidden(), root);
  useEffect(() => {
    if (veiled && stance && agentsLit(stance)) show(...agentsKeys(stance.folder));
  }, [veiled, stance]);
  const tree = agentsTree(papers.folders);
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

  // The last document leaving still needs a round, so an AGENTS left empty keeps one going.
  const print = root ? `${root}:${agentsPrint(papers.docs, papers.folders)}` : "";
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

  if (!asking && asleep) {
    const woken = () => {
      setWaking(true);
      light(asleep)
        .then(() => setAsleep(null))
        .catch((e) => setStirring(saidPlainly(e)))
        .finally(() => setWaking(false));
    };
    return (
      <AgentsAsleep
        state={asleep}
        busy={waking}
        trouble={stirring}
        onLight={woken}
        onLater={() => setAsleep(null)}
        onHide={() => {
          hide(AGENTS);
          setAsleep(null);
        }}
      />
    );
  }
  if (!asking) return null;
  const closed = () => {
    if (!lighting.current) return;
    lighting.current = false;
    agentsState()
      .then((state) => {
        if (onlySwitchedOff(state)) return switchOn(state.folder);
        if (agentsLit(state)) show(...agentsKeys(state.folder));
      })
      .catch((e) => setError(saidPlainly(e)));
  };
  return (
    <AgentsFirst
      more={asking === "more"}
      onClose={() => {
        setAsking(null);
        window.dispatchEvent(new Event(AGENTS_CHANGED));
        if (leaving.current === root && !holds) letGo();
        closed();
      }}
      onChanged={changed}
    />
  );
}
