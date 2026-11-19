import { useCallback, useEffect, useState } from "react";
import {
  AGENTS_CHANGED,
  type AgentsLook,
  type AgentsState,
  type AssistantSeen,
  agentsLead,
  agentsLit,
  agentsLook,
  agentsState,
  agentsTurn,
  bringMoreAgents,
  lightAgents,
} from "../agentsSync";
import { AGENTS, agentsHidden, agentsKeys, hide, show, useHidden } from "../hiding";
import { fill, t } from "../locales";
import { saidPlainly } from "../refusal";
import { AgentsFound } from "./AgentsAsleep";
import { askAgents } from "./AgentsKeeper";
import Card, { type Which, type Word } from "./Card";
import { Knob, Line } from "./Rows";

const small =
  "mt-2 self-start rounded-md border border-line px-2.5 py-0.5 text-[12.5px] text-soft hover:border-accent hover:text-accent";

const whyOf = (look: AgentsLook, one: AssistantSeen): string | undefined => {
  if (!look.allOn) return undefined;
  if (!one.on) return t("agentsSettingsOff");
  return look.unled.includes(one.id) ? t("agentsSettingsUnled") : undefined;
};

export default function AgentsSettings({ busy }: Readonly<{ busy: Which | null }>) {
  const [state, setState] = useState<AgentsState>();
  const [look, setLook] = useState<AgentsLook>();
  const [trouble, setTrouble] = useState<Word>();
  const hidden = agentsHidden(useHidden(), state?.folder);

  // Before this computer's yes, only whether each assistant's folder exists is looked at.
  const looked = useCallback(() => {
    agentsState()
      .then((now) => {
        setState(now);
        if (!now.consented) return setLook(undefined);
        return agentsLook().then(setLook);
      })
      .catch((e) => setTrouble({ card: "wiring", text: saidPlainly(e) }));
  }, []);

  useEffect(() => {
    looked();
    window.addEventListener(AGENTS_CHANGED, looked);
    return () => window.removeEventListener(AGENTS_CHANGED, looked);
  }, [looked]);

  const lead = () => {
    setTrouble(undefined);
    agentsLead()
      .then(() => {
        looked();
        window.dispatchEvent(new Event(AGENTS_CHANGED));
      })
      .catch((e) => setTrouble({ card: "wiring", text: saidPlainly(e) }));
  };

  const turn = (assistant: string | null, on: boolean) => {
    setTrouble(undefined);
    agentsTurn(assistant, on)
      .then(() => {
        looked();
        window.dispatchEvent(new Event(AGENTS_CHANGED));
      })
      .catch((e) => setTrouble({ card: "wiring", text: saidPlainly(e) }));
  };

  return (
    <Card title={t("agentsSettingsTitle")} which="wiring" busy={busy} trouble={trouble}>
      <p className="text-[12.5px] leading-relaxed text-soft">{t("agentsSettingsLine")}</p>
      {state && !agentsLit(state) && (
        <div className="mt-3">
          <AgentsFound state={state} />
          <div className="flex gap-2">
            <button type="button" onClick={lightAgents} className={small}>
              {t("agentsLight")}
            </button>
            {!hidden && (
              <button type="button" onClick={() => hide(AGENTS)} className={small}>
                {t("agentsHide")}
              </button>
            )}
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
      {look?.consented && look.both.length > 0 && (
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
      {look?.consented && look.loose.length > 0 && (
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
      {look?.consented && look.folder && (
        <button type="button" onClick={bringMoreAgents} className={small}>
          {t("agentsSettingsMore")}
        </button>
      )}
      {look && state && agentsLit(state) && look.assistants.length === 0 && (
        <p className="mt-2 text-[12.5px] text-faint">{t("agentsSettingsNone")}</p>
      )}
      {look?.consented && (
        <div className="mt-2">
          <Line
            title={t("agentsSettingsAll")}
            why={look.allOn ? undefined : t("agentsSettingsOff")}
            which="wiring"
          >
            <Knob
              on={look.allOn}
              label={t("agentsSettingsAll")}
              onPress={() => turn(null, !look.allOn)}
            />
          </Line>
          {look.assistants.map((one) => (
            <Line
              key={one.id}
              title={fill("agentsSettingsOne", one.name)}
              why={whyOf(look, one)}
              which="wiring"
            >
              <Knob
                on={one.on && look.allOn}
                disabled={!look.allOn}
                label={fill("agentsSettingsOne", one.name)}
                onPress={() => turn(one.id, !one.on)}
              />
            </Line>
          ))}
          {look.unled.length > 0 && (
            <button type="button" onClick={lead} className={small}>
              {t("agentsLeadAgain")}
            </button>
          )}
        </div>
      )}
    </Card>
  );
}
