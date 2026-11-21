import { useCallback, useEffect, useState } from "react";
import {
  AGENTS_CHANGED,
  type AgentsLook,
  type AgentsState,
  type AssistantSeen,
  agentsLead,
  agentsLook,
  agentsStage,
  agentsState,
  agentsTurn,
  askAgents,
  bringMoreAgents,
  syncAgents,
} from "../agentsSync";
import { AGENTS, agentsHidden, agentsKeys, hide, show, useHidden } from "../hiding";
import { fill, t } from "../locales";
import { saidPlainly } from "../refusal";
import { AgentsFound } from "./AgentsFound";
import { AssistantMark } from "./AssistantMark";
import Card, { type Run, type Word } from "./Card";
import { Knob, Line } from "./Rows";

const small =
  "mt-2 self-start rounded-md border border-line px-2.5 py-0.5 text-[12.5px] text-soft hover:border-accent hover:text-accent";

const whyOf = (look: AgentsLook, one: AssistantSeen): string | undefined => {
  if (!one.on) return t("agentsSettingsOff");
  if (look.unled.includes(one.id)) return t("agentsSettingsUnled");
  return one.files.length === 0 ? fill("agentsNothingYet", one.name) : undefined;
};

/// The same words and the same acts as the sidebar: Bring, Bring and sync, Sync, Stop syncing and Hide.
export default function AgentsSettings({
  fail,
  quietly,
}: Readonly<{ fail: (word?: Word) => void; quietly: Run }>) {
  const [state, setState] = useState<AgentsState>();
  const [look, setLook] = useState<AgentsLook>();
  const setTrouble = fail;
  const hidden = agentsHidden(useHidden(), state?.folder);
  const stage = state ? agentsStage(state.folder, state) : null;

  // Before this computer's yes, only whether each assistant's folder exists is looked at.
  const looked = useCallback(() => {
    agentsState()
      .then((now) => {
        setState(now);
        if (!now.consented) return setLook(undefined);
        return agentsLook().then(setLook);
      })
      .catch((e) => setTrouble({ card: "wiring", text: saidPlainly(e) }));
  }, [setTrouble]);

  useEffect(() => {
    looked();
    window.addEventListener(AGENTS_CHANGED, looked);
    return () => window.removeEventListener(AGENTS_CHANGED, looked);
  }, [looked]);

  const answered = (step: Promise<unknown>) =>
    quietly("wiring", step, () => {
      looked();
      window.dispatchEvent(new Event(AGENTS_CHANGED));
    });

  const hideButton = !hidden && (
    <button type="button" onClick={() => hide(AGENTS)} className={small}>
      {t("agentsHide")}
    </button>
  );

  return (
    <Card row="agentsFolder" which="wiring">
      <p className="text-[12.5px] leading-relaxed text-soft">{t("agentsSettingsLine")}</p>
      {state && stage === "unbrought" && (
        <div className="mt-3">
          <AgentsFound state={state} />
          {/* Both open the first time, where the person chooses what comes in. */}
          <div className="flex gap-2">
            <button type="button" onClick={askAgents} className={small}>
              {t("agentsBring")}
            </button>
            <button type="button" onClick={askAgents} className={small}>
              {t("agentsBringSync")}
            </button>
            {hideButton}
          </div>
        </div>
      )}
      {stage === "brought" && (
        <div className="mt-3">
          <p className="text-[12.5px] text-soft">{t("agentsNotSynced")}</p>
          <div className="flex gap-2">
            <button type="button" onClick={syncAgents} className={small}>
              {t("agentsSync")}
            </button>
            {state?.consented && (
              <button type="button" onClick={bringMoreAgents} className={small}>
                {t("agentsSettingsMore")}
              </button>
            )}
            {hideButton}
          </div>
        </div>
      )}
      {hidden && (
        <>
          <p className="mt-2 text-[12.5px] text-soft">{t("agentsSettingsHidden")}</p>
          <button
            type="button"
            onClick={() => show(...agentsKeys(state?.folder))}
            className={small}
          >
            {t("showAgain")}
          </button>
        </>
      )}
      {stage === "syncs" && look && <Syncing look={look} answered={answered} />}
    </Card>
  );
}

function Syncing({
  look,
  answered,
}: Readonly<{ look: AgentsLook; answered: (step: Promise<unknown>) => void }>) {
  return (
    <>
      {look.both.length > 0 && (
        <>
          <p className="mt-2 text-[12.5px] text-soft">
            {look.both.length === 1
              ? t("agentsSettingsBothOne")
              : fill("agentsSettingsBoth", String(look.both.length))}
          </p>
          <button type="button" onClick={askAgents} className={small}>
            {t("agentsSettingsReview")}
          </button>
        </>
      )}
      {look.loose.length > 0 && (
        <>
          <p className="mt-2 text-[12.5px] text-soft">
            {look.loose.length === 1
              ? t("agentsSettingsLooseOne")
              : fill("agentsSettingsLoose", String(look.loose.length))}
          </p>
          <button type="button" onClick={askAgents} className={small}>
            {t("agentsSettingsReview")}
          </button>
        </>
      )}
      <div className="flex gap-2">
        <button type="button" onClick={bringMoreAgents} className={small}>
          {t("agentsSettingsMore")}
        </button>
        <button type="button" onClick={() => answered(agentsTurn(null, false))} className={small}>
          {t("agentsUnsync")}
        </button>
      </div>
      {look.assistants.length === 0 && (
        <p className="mt-2 text-[12.5px] text-faint">{t("agentsSettingsNone")}</p>
      )}
      <div className="mt-2">
        {look.assistants.map((one) => (
          <Line
            item
            key={one.id}
            title={
              <span className="flex items-center gap-2">
                <AssistantMark id={one.id} className="h-[14px] w-[14px]" />
                {fill("agentsSettingsOne", one.name)}
              </span>
            }
            why={whyOf(look, one)}
            which="wiring"
          >
            <Knob
              on={one.on}
              label={fill("agentsSettingsOne", one.name)}
              onPress={() => answered(agentsTurn(one.id, !one.on))}
            />
          </Line>
        ))}
        {look.unled.length > 0 && (
          <button type="button" onClick={() => answered(agentsLead())} className={small}>
            {t("agentsLeadAgain")}
          </button>
        )}
      </div>
    </>
  );
}
