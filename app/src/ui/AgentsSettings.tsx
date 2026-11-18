import { useCallback, useEffect, useState } from "react";
import { type AgentsLook, agentsLook, agentsTurn } from "../agentsSync";
import { fill, t } from "../locales";
import { saidPlainly } from "../refusal";
import { AGENTS_CHANGED, askAgents } from "./AgentsKeeper";
import Card, { type Which, type Word } from "./Card";
import { Knob, Line } from "./Rows";

const small =
  "mt-2 self-start rounded-md border border-line px-2.5 py-0.5 text-[12.5px] text-soft hover:border-accent hover:text-accent";

export default function AgentsSettings({ busy }: Readonly<{ busy: Which | null }>) {
  const [look, setLook] = useState<AgentsLook>();
  const [trouble, setTrouble] = useState<Word>();

  const looked = useCallback(() => {
    agentsLook()
      .then(setLook)
      .catch((e) => setTrouble({ card: "wiring", text: saidPlainly(e) }));
  }, []);

  useEffect(() => {
    looked();
    window.addEventListener(AGENTS_CHANGED, looked);
    return () => window.removeEventListener(AGENTS_CHANGED, looked);
  }, [looked]);

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
      {look && !look.folder && (
        <button type="button" onClick={askAgents} className={small}>
          {t("agentsSettingsCreate")}
        </button>
      )}
      {look?.folder && !look.consented && (
        <>
          <p className="mt-2 text-[12.5px] text-soft">{t("agentsSettingsAsk")}</p>
          <button type="button" onClick={askAgents} className={small}>
            {t("agentsSettingsAllow")}
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
      {look?.assistants.length === 0 && (
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
              why={one.on || !look.allOn ? undefined : t("agentsSettingsOff")}
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
        </div>
      )}
    </Card>
  );
}
