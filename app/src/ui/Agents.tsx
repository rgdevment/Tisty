import { useEffect, useState } from "react";
import {
  type Agent,
  type Assistant,
  agentState,
  agentTurn,
  assistants,
  copied,
  openLink,
  seenAgents,
  type Wired,
} from "../core";
import { stamped } from "../format";
import { fill, t } from "../locales";
import { saidPlainly } from "../refusal";
import Card, { type Run, type Which, type Word } from "./Card";
import { Group } from "./Rows";
import Wiring from "./Wiring";

interface Props {
  busy: Which | null;
  said?: Word;
  trouble?: Word;
  quietly: Run;
  tell: (word?: Word) => void;
  fail: (word?: Word) => void;
  binary?: string;
  known: Known;
}

export function useAgents(shown: boolean, fail: (word?: Word) => void) {
  const [agent, setAgent] = useState<Agent | null>(null);
  const [agents, setAgents] = useState<Wired[] | null>(null);
  const [hands, setHands] = useState<Assistant[] | null>(null);

  useEffect(() => {
    if (!shown) return;
    agentState()
      .then((fresh) => setAgent(fresh))
      .catch((e) => fail({ card: "settings", text: saidPlainly(e) }));
    seenAgents()
      .then(setAgents)
      .catch((e) => fail({ card: "wiring", text: saidPlainly(e) }));
    assistants()
      .then(setHands)
      .catch((e) => fail({ card: "wiring", text: saidPlainly(e) }));
  }, [shown, fail]);

  return { agent, setAgent, agents, setAgents, hands };
}

export type Known = ReturnType<typeof useAgents>;

export default function Agents({ busy, said, trouble, quietly, tell, fail, binary, known }: Props) {
  const held = busy !== null;
  const { agent, setAgent, agents, setAgents, hands } = known;
  const [wired, setWired] = useState(false);
  const [typed, setTyped] = useState(false);

  return (
    <>
      <Group label={t("tabAgents")} />

      <Card title={t("agentsTitle")} which="settings" busy={busy} said={said} trouble={trouble}>
        <p className="text-[12.5px] leading-relaxed text-soft">{t("agentsWhat")}</p>

        <div className="mt-3 flex items-center justify-between gap-3 rounded-[10px] border border-hair px-3 py-2.5">
          <span className="min-w-0">
            <span className="block text-[13px] font-semibold">
              {agent?.on ? t("agentsOn") : t("agentsOff")}
            </span>
            {agent?.on && (
              <span className="block text-[12.5px] text-soft">
                {fill("agentsSignsAs", agent.called ?? "", agent.code ?? "—")}
              </span>
            )}
          </span>
          <button
            type="button"
            disabled={held}
            onClick={() => {
              agentTurn(!agent?.on)
                .then((fresh) => setAgent(fresh))
                .catch((e) => fail({ card: "settings", text: saidPlainly(e) }));
            }}
            className={`shrink-0 rounded-md border px-2.5 py-1 text-[12.5px] disabled:text-faint ${
              agent?.on
                ? "border-line text-soft hover:border-urgent hover:text-urgent"
                : "border-accent text-accent"
            }`}
          >
            {agent?.on ? t("agentsTurnOff") : t("agentsTurnOn")}
          </button>
        </div>

        <p className="mt-3 text-[12.5px] leading-relaxed text-soft">{t("agentsUndo")}</p>
        <button
          type="button"
          onClick={() =>
            openLink(t("agentsGuideAt")).catch((e) =>
              fail({ card: "settings", text: saidPlainly(e) }),
            )
          }
          className="mt-2 text-[12.5px] text-accent hover:underline"
        >
          {t("agentsGuide")} <span aria-hidden="true">↗</span>
        </button>
      </Card>

      <Card title={t("wiringTitle")} which="wiring" busy={busy} said={said} trouble={trouble}>
        <Wiring
          agents={agents}
          setAgents={setAgents}
          hands={hands}
          held={held}
          quietly={quietly}
          tell={tell}
          fail={fail}
          wroteSaid={wroteSaid}
        />
        {agents && agents.length > 0 && (
          <>
            {agent?.on === false && (
              <p className="mt-2.5 text-[12.5px] leading-relaxed text-soft">{t("wiringMute")}</p>
            )}
            <p className="mt-2.5 text-[11.5px] leading-relaxed text-faint">{t("wiringBefore")}</p>
          </>
        )}
      </Card>

      <Card title={t("agentsCanTitle")} which="settings" busy={busy} said={said} trouble={trouble}>
        <p className="text-[12.5px] leading-relaxed text-soft">{t("agentsCan")}</p>
      </Card>

      <Card
        title={t("agentsCannotTitle")}
        which="settings"
        busy={busy}
        said={said}
        trouble={trouble}
      >
        <p className="text-[12.5px] leading-relaxed text-soft">{t("agentsCannot")}</p>
      </Card>

      <Card title={t("agentsHowTitle")} which="settings" busy={busy} said={said} trouble={trouble}>
        <p className="text-[12.5px] leading-relaxed text-soft">
          {t(binary ? "agentsHow" : "agentsHowAlias")}
        </p>

        <p className="mt-3 text-[12.5px] font-semibold">{t("agentsByFile")}</p>
        <pre className="mt-1 overflow-x-auto rounded-[10px] border border-hair px-3 py-2 font-mono text-[11.5px] text-soft">
          {wiring(binary)}
        </pre>
        <button
          type="button"
          onClick={() => {
            void copied(wiring(binary)).then(() => {
              setWired(true);
              window.setTimeout(() => setWired(false), 1500);
            });
          }}
          className="mt-2 rounded-md border border-line px-2.5 py-0.5 text-[12.5px] text-soft hover:border-accent hover:text-accent"
        >
          {wired ? t("agentsCopied") : t("agentsCopy")}
        </button>

        <p className="mt-4 text-[12.5px] font-semibold">{t("agentsByLine")}</p>
        <pre className="mt-1 rounded-[10px] border border-hair px-3 py-2 font-mono text-[11.5px] whitespace-pre-wrap text-soft [overflow-wrap:anywhere]">
          {oneLine(binary, t("agentsCalled"))}
        </pre>
        <p className="mt-1 text-[11.5px] text-faint">{t("agentsWhichever")}</p>
        <button
          type="button"
          onClick={() => {
            void copied(oneLine(binary, t("agentsCalled"))).then(() => {
              setTyped(true);
              window.setTimeout(() => setTyped(false), 1500);
            });
          }}
          className="mt-2 rounded-md border border-line px-2.5 py-0.5 text-[12.5px] text-soft hover:border-accent hover:text-accent"
        >
          {typed ? t("agentsCopied") : t("agentsCopy")}
        </button>
      </Card>
    </>
  );
}

const wroteSaid = (hand: Assistant | undefined): string => {
  if (!hand || hand.wrote === 0) return t("assistantNothing");
  const said = [fill("assistantFiled", String(hand.filed))];
  if (hand.wrote > hand.filed) said.push(fill("assistantWrote", String(hand.wrote)));
  if (hand.last) said.push(fill("assistantLast", stamped(hand.last)));
  return said.join(" · ");
};

const wiring = (at?: string) =>
  `{
  "mcpServers": {
    "tisty": { "command": ${JSON.stringify(at ?? "tisty")}, "args": ["mcp"] }
  }
}`;

const oneLine = (at?: string, agent = "agent") =>
  `${agent} mcp add tisty -- ${JSON.stringify(at ?? "tisty")} mcp`;
