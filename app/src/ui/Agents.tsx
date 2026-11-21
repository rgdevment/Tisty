import { useEffect, useState } from "react";
import {
  type Agent,
  type Assistant,
  agentAsks,
  agentHooks,
  agentState,
  agentTurn,
  assistants,
  copied,
  seenAgents,
  type Wired,
} from "../core";
import { type Credit, credit, useCredit } from "../crediting";
import { stamped } from "../format";
import { fill, t } from "../locales";
import { saidPlainly } from "../refusal";
import { type Shelf, settingAt } from "../settingsRows";
import AgentsSettings from "./AgentsSettings";
import Card, { type Run, type Word } from "./Card";
import { useCareful } from "./Careful";
import { DANGERS } from "./dangers";
import { Note, NoteButton } from "./RowNotes";
import { Knob, Line } from "./Rows";
import { tabKeys } from "./tabbing";
import Wiring from "./Wiring";
import { held as isHeld, type Working } from "./working";
import { More } from "./Zone";

interface Props {
  busy: Working;
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

const SHELF_PANEL = "agents-shelf";
const shelfTab = (shelf: Shelf) => `agents-tab-${shelf}`;

const SHELVES: [Shelf, Parameters<typeof t>[0]][] = [
  ["door", "agentsTabDoor"],
  ["wired", "agentsTabWired"],
  ["folder", "agentsTabFolder"],
  ["credit", "agentsTabCredit"],
  ["can", "agentsTabCan"],
];

const SHELF_KEYS = SHELVES.map(([key]) => key);

export default function Agents({
  sent,
  sentAt,
  ...props
}: Props & { sent?: Shelf; sentAt?: number }) {
  const [shelf, setShelf] = useState<Shelf>(sent ?? "door");
  useEffect(() => {
    if (sent) setShelf(sent);
  }, [sent, sentAt]);
  const moved = tabKeys(SHELF_KEYS, false, shelfTab, setShelf);
  return (
    <>
      <div
        role="tablist"
        aria-label={t("tabAgents")}
        className="mt-3 flex flex-wrap gap-x-0.5 border-b border-hair"
      >
        {SHELVES.map(([key, said]) => (
          <button
            key={key}
            id={shelfTab(key)}
            type="button"
            role="tab"
            aria-selected={shelf === key}
            aria-controls={SHELF_PANEL}
            tabIndex={shelf === key ? 0 : -1}
            onClick={() => setShelf(key)}
            onKeyDown={(event) => moved(event, key)}
            className={`-mb-px border-b-2 px-2.5 py-1.5 text-[12.5px] whitespace-nowrap ${
              shelf === key
                ? "border-accent font-semibold text-ink"
                : "border-transparent text-soft"
            }`}
          >
            {t(said)}
          </button>
        ))}
      </div>
      <div id={SHELF_PANEL} role="tabpanel" aria-labelledby={shelfTab(shelf)}>
        {shelf === "door" && <Door {...props} />}
        {shelf === "wired" && <Connected {...props} onDoor={() => setShelf("door")} />}
        {shelf === "folder" && <AgentsSettings fail={props.fail} quietly={props.quietly} />}
        {shelf === "credit" && <Crediting />}
        {shelf === "can" && <Can />}
      </div>
    </>
  );
}

function Door({ busy, quietly, tell, known }: Props) {
  const held = isHeld(busy, "door");
  const { agent, setAgent } = known;
  const careful = useCareful();
  const turn = async () => {
    if (agent?.on && !(await careful.ask(DANGERS.door()))) return;
    quietly("door", () => agentTurn(!agent?.on), setAgent);
  };
  const habit = (card: "asks" | "hooks", work: () => Promise<Agent>) =>
    quietly(card, work, (fresh) => {
      setAgent(fresh);
      tell({ card, text: t("settingsKept") });
    });
  const open = agent?.on === true;
  return (
    <Card row="door" which="door">
      {careful.shown}
      <div className="flex items-center justify-between gap-3 rounded-[10px] border border-hair px-3 py-2.5">
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
          onClick={() => void turn()}
          className={`shrink-0 rounded-md border px-2.5 py-1 text-[12.5px] disabled:text-faint ${
            agent?.on
              ? "border-line text-soft hover:border-urgent hover:text-urgent"
              : "border-accent text-accent"
          }`}
        >
          {agent?.on ? t("agentsTurnOff") : t("agentsTurnOn")}
        </button>
      </div>
      <More>
        <span className="block">{t("agentsWhat")}</span>
        <span className="mt-2 block">{t("agentsUndo")}</span>
      </More>

      <Line row="asks" why={t("agentsAskWhy")} which="asks">
        <Knob
          on={agent?.asks !== false}
          label={t("agentsAsk")}
          disabled={isHeld(busy, "asks") || !open}
          onPress={() => habit("asks", () => agentAsks(agent?.asks === false))}
        />
      </Line>

      <Line
        row="hooks"
        why={t("agentsHooksWhy")}
        which="hooks"
        note={agent && !open && <Note kind="dep">{t("doorNeeded")}</Note>}
      >
        <Knob
          on={agent?.hooks !== false}
          label={t("agentsHooks")}
          disabled={isHeld(busy, "hooks") || !open}
          onPress={() => habit("hooks", () => agentHooks(agent?.hooks === false))}
        />
      </Line>
    </Card>
  );
}

function Connected({
  busy,
  quietly,
  tell,
  fail,
  binary,
  known,
  onDoor,
}: Props & { onDoor: () => void }) {
  const held = isHeld(busy, "wiring");
  const { agent, agents, setAgents, hands } = known;
  return (
    <Card row="wired" which="wiring">
      {agent?.on === false && agents && agents.length > 0 && (
        <Note kind="dep" act={<NoteButton label={t("goToDoor")} onPress={onDoor} />}>
          {t("wiringMute")}
        </Note>
      )}
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
        <p className="mt-2.5 text-[11.5px] leading-relaxed text-faint">{t("wiringBefore")}</p>
      )}
      <ByHand binary={binary} />
    </Card>
  );
}

function ByHand({ binary }: { binary?: string }) {
  const [wired, setWired] = useState(false);
  const [typed, setTyped] = useState(false);
  return (
    <details className="mt-4 border-t border-hair pt-3">
      <summary className="cursor-pointer text-[12.5px] font-semibold text-accent hover:underline">
        {t("agentsHowTitle")}
      </summary>
      <p className="mt-2 text-[12.5px] leading-relaxed text-soft">
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
    </details>
  );
}

const PREVIEWED = "Claude Code";

const previewed = (shown: Credit) =>
  shown.mark ? "◆" : fill("agentWrote", shown.rows ? PREVIEWED : t("anAgent"));

function Crediting() {
  const shown = useCredit();
  return (
    <Card row="credit" which="settings">
      <p className="text-[12.5px] leading-relaxed text-soft">{t("creditWhat")}</p>
      <div className="mt-3 rounded-[10px] border border-hair bg-panel px-3 py-2.5">
        <p className="text-[11.5px] text-faint">{t("creditPreview")}</p>
        <p className="mt-1.5 flex items-center gap-3 text-[13px]">
          <span className="min-w-0 flex-1 truncate">{t("creditPreviewTask")}</span>
          <span className="shrink-0 text-[12.5px] text-hue-teal" data-testid="credit-preview">
            {previewed(shown)}
          </span>
        </p>
      </div>
      <Line row="creditMark" why={t("creditMarkWhy")} which="settings">
        <Knob on={shown.mark} label={t("creditMark")} onPress={() => credit("mark", !shown.mark)} />
      </Line>
      <Line row="creditRows" why={t("creditRowsWhy")} which="settings">
        <Knob
          on={shown.rows && !shown.mark}
          label={t("creditRows")}
          disabled={shown.mark}
          onPress={() => credit("rows", !shown.rows)}
        />
      </Line>
      <Line
        row="creditInside"
        why={t("creditInsideWhy")}
        which="settings"
        note={shown.mark && <Note kind="dep">{t("creditMarkHolds")}</Note>}
      >
        <Knob
          on={shown.inside && !shown.mark}
          label={t("creditInside")}
          disabled={shown.mark}
          onPress={() => credit("inside", !shown.inside)}
        />
      </Line>
    </Card>
  );
}

const LISTS = [
  ["agentsCanTitle", "agentsCan", "✓", "text-hue-green"],
  ["agentsCannotTitle", "agentsCannot", "✕", "text-urgent"],
] as const;

function Can() {
  return (
    <section
      {...settingAt("can")}
      aria-label={t("agentsTabCan")}
      className="mt-4 grid gap-4 sm:grid-cols-2"
    >
      {LISTS.map(([title, items, mark, hue]) => (
        <section key={title} aria-label={t(title)}>
          <h3 className="text-[11.5px] font-semibold tracking-[0.06em] text-faint uppercase">
            {t(title)}
          </h3>
          <ul className="mt-2 text-[12.5px] leading-snug text-soft">
            {t(items)
              .split(" · ")
              .map((one) => (
                <li key={one} className="flex gap-2 border-t border-hair py-1.5">
                  <span aria-hidden="true" className={`w-3 shrink-0 font-bold ${hue}`}>
                    {mark}
                  </span>
                  <span className="min-w-0">{one}</span>
                </li>
              ))}
          </ul>
        </section>
      ))}
    </section>
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
