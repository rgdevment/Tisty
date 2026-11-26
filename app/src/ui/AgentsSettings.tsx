import { type ReactNode, useCallback, useEffect, useState } from "react";
import { hideAgents } from "../agentsHiding";
import {
  AGENTS_CHANGED,
  type AgentsLook,
  type AgentsState,
  type AssistantSeen,
  agentsLead,
  agentsLook,
  agentsRestore,
  agentsStage,
  agentsState,
  agentsTurn,
  askAgents,
  askAgentsStayed,
  bringMoreAgents,
  syncAgents,
} from "../agentsSync";
import { agentsHidden, agentsKeys, show, useHidden } from "../hiding";
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
    <button
      type="button"
      onClick={() =>
        hideAgents(agentsKeys(state?.folder), stage)
          .then((hidden) => hidden && answered(Promise.resolve()))
          .catch((e) => setTrouble({ card: "wiring", text: saidPlainly(e) }))
      }
      className={small}
    >
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
      {stage === "syncs" && look && (
        <Syncing look={look} answered={answered} hideButton={hideButton} />
      )}
      {stage !== "syncs" && look && <Stayed look={look} />}
      {look && look.vanished > 0 && (
        <Vanished
          many={look.vanished}
          onWrite={() =>
            quietly("wiring", agentsRestore(), (done) => {
              if (done.failed === 1)
                setTrouble({ card: "wiring", text: t("agentsVanishedFailedOne") });
              else if (done.failed > 1)
                setTrouble({
                  card: "wiring",
                  text: fill("agentsVanishedFailed", String(done.failed)),
                });
              looked();
              window.dispatchEvent(new Event(AGENTS_CHANGED));
            })
          }
        />
      )}
    </Card>
  );
}

function Vanished({ many, onWrite }: Readonly<{ many: number; onWrite: () => void }>) {
  return (
    <>
      <p className="mt-2 text-[12.5px] text-soft">
        {many === 1 ? t("agentsVanishedOne") : fill("agentsVanished", String(many))}
      </p>
      <p className="mt-0.5 text-[11.5px] text-faint">{t("agentsVanishedWhy")}</p>
      <button type="button" onClick={onWrite} className={small}>
        {many === 1 ? t("agentsVanishedDoOne") : t("agentsVanishedDo")}
      </button>
    </>
  );
}

// What the person left with their agents stays theirs to take away, even once Agents is gone.
function Stayed({ look }: Readonly<{ look: AgentsLook }>) {
  const stayed = look.loose.filter((one) => one.stayed).length;
  if (stayed === 0) return null;
  return (
    <>
      <p className="mt-2 text-[12.5px] text-soft">
        {stayed === 1 ? t("agentsSettingsStayedOne") : fill("agentsSettingsStayed", String(stayed))}
      </p>
      <button type="button" onClick={askAgentsStayed} className={small}>
        {t("agentsSettingsReview")}
      </button>
    </>
  );
}

function Syncing({
  look,
  answered,
  hideButton,
}: Readonly<{
  look: AgentsLook;
  answered: (step: Promise<unknown>) => void;
  hideButton: ReactNode;
}>) {
  const loose = look.loose.filter((one) => !one.stayed).length;
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
      {loose > 0 && (
        <>
          <p className="mt-2 text-[12.5px] text-soft">
            {loose === 1 ? t("agentsSettingsLooseOne") : fill("agentsSettingsLoose", String(loose))}
          </p>
          <button type="button" onClick={askAgents} className={small}>
            {t("agentsSettingsReview")}
          </button>
        </>
      )}
      <Stayed look={look} />
      <div className="flex gap-2">
        <button type="button" onClick={bringMoreAgents} className={small}>
          {t("agentsSettingsMore")}
        </button>
        <button type="button" onClick={() => answered(agentsTurn(null, false))} className={small}>
          {t("agentsUnsync")}
        </button>
        {hideButton}
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
