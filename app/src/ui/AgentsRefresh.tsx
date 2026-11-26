import { useCallback, useEffect, useRef, useState } from "react";
import { AGENTS_CHANGED, agentsTakeable, rescanAgents } from "../agentsSync";
import { countedAs } from "../format";
import { t } from "../locales";
import { stored } from "../stored";

export const LOOK_EVERY_MS = 60_000;

// What the person last saw when they looked again; until then, what was there the first time counts as seen.
const seenNames = stored<{ names: string[] } | null>("tisty:agents-seen", (said) =>
  Array.isArray(said.names)
    ? {
        names: said.names.filter((one): one is string => typeof one === "string"),
      }
    : null,
);

const keep = (names: string[]) => seenNames.put({ names });

const newOf = (now: string[]) => {
  const seen = seenNames.current();
  if (seen === null) {
    keep(now);
    return 0;
  }
  const known = new Set(seen.names);
  return now.filter((one) => !known.has(one)).length;
};

export default function AgentsRefresh() {
  const [fresh, setFresh] = useState(0);
  const latest = useRef<string[]>([]);
  const lastLook = useRef(0);

  const look = useCallback(() => {
    lastLook.current = Date.now();
    agentsTakeable()
      .then((now) => {
        latest.current = now;
        setFresh(newOf(now));
      })
      .catch(() => undefined);
  }, []);

  useEffect(() => {
    look();
    const focused = () => {
      if (Date.now() - lastLook.current >= LOOK_EVERY_MS) look();
    };
    const changed = () => setFresh(newOf(latest.current));
    window.addEventListener("focus", focused);
    window.addEventListener(AGENTS_CHANGED, changed);
    return () => {
      window.removeEventListener("focus", focused);
      window.removeEventListener(AGENTS_CHANGED, changed);
    };
  }, [look]);

  const lookAgain = () => {
    keep(latest.current);
    setFresh(0);
    rescanAgents();
  };

  const said = countedAs(fresh, "agentsNewOne", "agentsNewMany");
  return (
    <button
      type="button"
      onClick={lookAgain}
      title={t("agentsRescan")}
      aria-label={said ? `${t("agentsRescan")} — ${said}` : t("agentsRescan")}
      className="flex shrink-0 cursor-pointer items-center gap-1 rounded-md px-1.5 py-0.5 text-[12.5px] text-faint hover:bg-active hover:text-soft"
    >
      <span aria-hidden="true">⟳</span>
      {said && <span className="text-accent">{said}</span>}
    </button>
  );
}
