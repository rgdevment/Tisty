import { type MutableRefObject, useCallback, useEffect, useRef, useState } from "react";
import { type Snapshot, settleIn, snapshot } from "./core";
import { adopt, t } from "./locales";
import { saidPlainly } from "./refusal";
import { asView, type Chosen } from "./views";
import { knowAgents } from "./who";

interface Hands {
  seen: Chosen;
  reach: number;
  acted: MutableRefObject<string | null>;
  setError: (text: string | null) => void;
}

export function useSnapshot({ seen, reach, acted, setError }: Hands) {
  const [data, setData] = useState<Snapshot | null>(null);
  const [settling, setSettling] = useState(true);
  const [stuck, setStuck] = useState(false);

  const load = useCallback(() => {
    snapshot(asView(seen, reach))
      .then((fresh) => {
        adopt(fresh.locale);
        knowAgents(fresh.agents, {
          tag: fresh.agent_tag,
          hosts: fresh.hosts,
          machines: fresh.machines,
          here: fresh.machine_here,
          clients: fresh.clients,
        });
        setData(fresh);
        acted.current = null;
      })
      .catch((e) => setError(saidPlainly(e)));
  }, [seen, reach, setError, acted]);

  const latest = useRef(load);
  latest.current = load;

  useEffect(() => {
    settleIn()
      .then((done) => {
        if (done.stuck) {
          const apart = done.stuck.code === "wouldReset" || done.stuck.code === "otherStore";
          setError(apart ? t("stuckApart") : saidPlainly(done.stuck));
          setStuck(apart);
        }
        return done.brought && latest.current();
      })
      .catch((problem) => setError(saidPlainly(problem)))
      .finally(() => setSettling(false));
  }, [setError]);

  useEffect(() => {
    load();
  }, [load]);

  return { data, load, latest, settling, stuck, setStuck };
}
