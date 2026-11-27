import { useCallback, useEffect, useRef, useState } from "react";
import { AGENTS_CHANGED } from "../agentsSync";
import { agentsWsl, agentsWslRound, agentsWslSet, type InDistro } from "../agentsWsl";
import { fill, t } from "../locales";
import { saidPlainly } from "../refusal";
import type { Word } from "./Card";
import { Knob, Line } from "./Rows";

const small =
  "mt-2 self-start rounded-md border border-line px-2.5 py-0.5 text-[12.5px] text-soft hover:border-accent hover:text-accent";

const ROUND_EVERY_MS = 15_000;

const standingOf = (now: InDistro): string | null => {
  if (!now.agents && !now.skills) return null;
  if (!now.running && !now.looked) return t("wslAgentsStopped");
  if (!now.looked) return null;
  if (now.left > 0) return fill("wslAgentsLeft", String(now.left));
  return now.wrote > 0 ? fill("wslAgentsWrote", String(now.wrote)) : t("wslAgentsUpToDate");
};

export default function DistroAgents({
  name,
  held,
  fail,
}: Readonly<{ name: string; held: boolean; fail: (word?: Word) => void }>) {
  const [now, setNow] = useState<InDistro | null>(null);
  const [busy, setBusy] = useState(false);
  const failed = useCallback(
    (e: unknown) => fail({ card: "wiring", text: saidPlainly(e) }),
    [fail],
  );

  // The log stirs many times a minute while an agent works, and each round reads the distro anew.
  const last = useRef(0);
  const later = useRef<ReturnType<typeof setTimeout> | null>(null);
  const round = useCallback(() => {
    const wait = last.current + ROUND_EVERY_MS - Date.now();
    if (wait > 0) {
      later.current ??= setTimeout(() => {
        later.current = null;
        round();
      }, wait);
      return;
    }
    last.current = Date.now();
    agentsWslRound(name, false)
      .then(setNow)
      .catch(() => undefined);
  }, [name]);

  useEffect(() => {
    agentsWsl([name])
      .then(([first]) => {
        setNow(first ?? null);
        if (first?.running && (first.agents || first.skills)) round();
      })
      .catch(() => undefined);
    window.addEventListener(AGENTS_CHANGED, round);
    return () => {
      window.removeEventListener(AGENTS_CHANGED, round);
      if (later.current) clearTimeout(later.current);
      later.current = null;
    };
  }, [name, round]);

  const run = (step: Promise<InDistro>) => {
    setBusy(true);
    step
      .then(setNow)
      .catch(failed)
      .finally(() => setBusy(false));
  };

  if (!now) return null;
  const said = standingOf(now);
  const stopped = (now.agents || now.skills) && !now.running && !now.looked;
  return (
    <div className="mt-1.5 rounded-[10px] border border-hair px-3 py-1.5">
      <Line item title={t("wslAgentsRules")} why={fill("wslAgentsRulesWhy", name)} which="wiring">
        <Knob
          on={now.agents}
          label={t("wslAgentsRules")}
          onPress={() => run(agentsWslSet(name, { agents: !now.agents }))}
          disabled={held || busy}
        />
      </Line>
      <Line item title={t("wslAgentsSkills")} why={t("wslAgentsSkillsWhy")} which="wiring">
        <Knob
          on={now.skills}
          label={t("wslAgentsSkills")}
          onPress={() => run(agentsWslSet(name, { skills: !now.skills }))}
          disabled={held || busy}
        />
      </Line>
      {said && <p className="mt-1 text-[12.5px] text-soft">{said}</p>}
      {stopped && (
        <button
          type="button"
          disabled={held || busy}
          onClick={() => run(agentsWslRound(name, true))}
          className={small}
        >
          {fill("wslAgentsWake", name)}
        </button>
      )}
    </div>
  );
}
