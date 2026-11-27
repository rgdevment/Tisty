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

// Kept across visits to Agents, so opening the page again does not list the folders again within the minute.
const lastLook = { at: 0, list: null as string[] | null };

export default function AgentsRefresh() {
  const [fresh, setFresh] = useState(0);
  const counting = useRef(false);

  // Without this computer's yes there is nothing to compare, so what was seen stays as it was.
  const look = useCallback(() => {
    lastLook.at = Date.now();
    return agentsTakeable()
      .then((now) => {
        lastLook.list = now;
        const many = now ? newOf(now) : 0;
        counting.current = many > 0;
        setFresh(many);
        return now;
      })
      .catch(() => null);
  }, []);

  useEffect(() => {
    const due = () => Date.now() - lastLook.at >= LOOK_EVERY_MS;
    if (due()) look();
    else {
      const many = lastLook.list ? newOf(lastLook.list) : 0;
      counting.current = many > 0;
      setFresh(many);
    }
    const focused = () => {
      if (due()) look();
    };
    // Only a count already shown can go stale; anything new waits for the next focus.
    const changed = () => {
      if (counting.current) look();
    };
    window.addEventListener("focus", focused);
    window.addEventListener(AGENTS_CHANGED, changed);
    return () => {
      window.removeEventListener("focus", focused);
      window.removeEventListener(AGENTS_CHANGED, changed);
    };
  }, [look]);

  const lookAgain = () => {
    setFresh(0);
    rescanAgents();
    look().then((now) => {
      if (now) keep(now);
      setFresh(0);
    });
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
