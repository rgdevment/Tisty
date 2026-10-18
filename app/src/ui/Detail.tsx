import { useEffect, useRef, useState } from "react";
import {
  type Change,
  erasable,
  type Filed,
  type List,
  readingOf,
  type Task,
  type Whole,
} from "../core";
import { WROTE } from "../detailing";
import { cadence, daysFrom, stamped, whenLabel, wroteAt } from "../format";
import { fill, t } from "../locales";
import { placed, said } from "../quadrants";
import { agentNamed, clientNamed, hostedOn, signedBy } from "../who";
import Belongs from "./Belongs";
import { useEdge } from "./edging";
import Fields from "./Fields";
import History from "./History";
import Left from "./Left";
import Menu, { type Choice } from "./Menu";
import Panel from "./Panel";
import Routine from "./Routine";
import Section from "./Section";
import { saidBy } from "./Spoke";
import TaskWords from "./TaskWords";
import Trail from "./Trail";

interface Props {
  task: Task;
  lists: List[];
  known: string[];
  from?: string;
  apart?: string;
  expanded?: boolean;
  onExpand?: () => void;
  onCollapse: () => void;
  onPatch: (change: Change) => void;
  onComplete: () => void;
  onDiscard: () => void;
  onReopen: () => void;
  onStillOpen: () => void;
  onErase: () => void;
  onFold: (away: boolean) => void;
  onReadAs: (how: "story" | "trace") => void;
  onOpenToAgents: (open: boolean) => void;
  onClose: () => void;
  onError?: (problem: unknown) => void;
  onDoc?: (id: string) => void;
  papers?: readonly Filed[];
  whole?: Whole;
  partOf?: string;
  onAddPart?: (title: string) => void;
  onOpenPart?: (id: string) => void;
  onCompletePart?: (id: string, title: string) => void;
  onHang?: (whole: string | null) => void;
  onSettle?: (
    before: string,
    now: string,
    renamed?: readonly [string, string][],
  ) => Promise<string | null>;
}

// Keys from the page reach the window from the window itself, so what holds them is found through its frames.
const focused = (): Element | null => {
  let at: Element | null = document.activeElement;
  while (at instanceof HTMLIFrameElement) at = at.contentDocument?.activeElement ?? null;
  return at;
};

export default function Detail({
  task,
  lists,
  known,
  from,
  apart,
  expanded = true,
  onExpand,
  onCollapse,
  onPatch,
  onComplete,
  onDiscard,
  onReopen,
  onStillOpen,
  onErase,
  onFold,
  onReadAs,
  onOpenToAgents,
  onClose,
  onError,
  onDoc,
  papers,
  whole,
  partOf,
  onAddPart,
  onOpenPart,
  onCompletePart,
  onHang,
  onSettle,
}: Props) {
  const opened = useRef<HTMLElement>(null);
  const [shaping, setShaping] = useState(false);
  const [held, setHeld] = useState(false);
  // A page written on changes nothing the task holds, so the trail is told to look again.
  const [wrote, setWrote] = useState(0);
  const settle =
    onSettle &&
    ((before: string, now: string, renamed?: readonly [string, string][]) =>
      onSettle(before, now, renamed).then((problem) => {
        if (problem === WROTE) setWrote((one) => one + 1);
        return problem;
      }));
  // The template is edited only while the task is open and its words can be written.
  useEffect(() => {
    if (task.status !== "open" || held) setShaping(false);
  }, [task.status, held]);
  const card = useRef<HTMLDivElement>(null);
  const edge = useEdge(card);
  const live = task.status === "open";
  const holding = live && !task.repeat && !task.after && !task.hidden;
  useEffect(() => {
    opened.current?.focus({ preventScroll: true });
  }, [task.id]);

  // The page hands its keys to the window, so leaving the template is heard there, before Escape closes the task.
  useEffect(() => {
    if (!shaping) return;
    const done = (event: KeyboardEvent) => {
      if (event.key !== "Escape" && event.key !== "Enter") return;
      if (event.ctrlKey || event.metaKey || event.altKey || event.shiftKey) return;
      const at = event.target instanceof Element ? event.target : focused();
      if (at?.closest("input, textarea, select, button, [contenteditable='true']")) return;
      event.preventDefault();
      event.stopImmediatePropagation();
      setShaping(false);
    };
    window.addEventListener("keydown", done, true);
    return () => window.removeEventListener("keydown", done, true);
  }, [shaping]);

  const leave = (event: React.KeyboardEvent) => {
    if (event.key !== "Escape") return;
    const at = event.target as HTMLElement;
    if (at.isContentEditable || at.closest("input, textarea, select")) return;
    event.preventDefault();
    onClose();
  };

  const aside = (
    <>
      <Belongs
        task={task}
        whole={whole}
        partOf={partOf}
        holding={holding}
        onAddPart={onAddPart}
        onHang={onHang}
        onOpenPart={onOpenPart}
        onCompletePart={onCompletePart}
        onError={onError}
      />
      <Trail
        task={task.id}
        moved={[
          task.status,
          task.read_as ?? "",
          task.completed_at ?? "",
          task.hidden ? "hidden" : "",
          task.open_to_agents ? "open" : "",
          task.volume?.journal ?? 0,
          task.steps?.length ?? task.volume?.steps ?? 0,
          task.steps?.filter((step) => step.done).length ?? task.volume?.steps_done ?? 0,
          task.description ? "described" : "",
          task.title,
          wrote,
        ].join("|")}
        lists={lists}
        onError={onError}
        heading={<Section label={t("trail")} />}
        before={(from) => <Facts task={task} from={from} />}
        summed={task.status === "open" ? undefined : journalled(task)}
      />
      <Left
        task={task.id}
        onDoc={onDoc}
        onError={onError}
        heading={<Section label={t("left")} />}
      />
      {task.status !== "open" && (
        <p className="mt-4 border-t border-hair pt-3 text-[11.5px] leading-relaxed text-faint">
          {t("trailSealed")}
        </p>
      )}
    </>
  );

  const body = (
    <>
      <Title task={task} onRename={(title) => onPatch({ title })} />
      {task.status === "open" && signedBy(task.created_by, task.created_via) && (
        <p className="-mt-1.5 mb-3 text-[11.5px] text-hue-teal" title={hostedOn(task.created_by)}>
          {signedBy(task.created_by, task.created_via)}
        </p>
      )}
      {task.resolved && (
        <p className="mt-3 mb-4 flex items-center gap-2 rounded-md border border-hue-teal/40 bg-hue-teal/10 px-2.5 py-1.5 text-[12.5px] font-medium text-hue-teal">
          <span aria-hidden="true">◆</span>
          {clientNamed(task.resolved.via)
            ? fill(
                task.resolved.drop ? "agentNamedSaidNotDoing" : "agentNamedSaidDone",
                clientNamed(task.resolved.via) as string,
              )
            : t(task.resolved.drop ? "agentSaidNotDoing" : "agentSaidDone")}
          <span className="ml-auto font-normal text-faint">{stamped(task.resolved.at)}</span>
        </p>
      )}
      {task.status === "open" && task.open_to_agents && (
        <p className="mt-3 mb-4 flex items-center gap-2 rounded-md border border-hair bg-hover px-2.5 py-1.5 text-[12.5px] text-soft">
          <span aria-hidden="true">⊚</span>
          {t("openToAgents")}
        </p>
      )}
      <Fields task={task} lists={lists} known={known} onPatch={onPatch} />

      <TaskWords
        task={task}
        onSettle={settle}
        onError={onError}
        onDoc={onDoc}
        shaping={shaping}
        onHeld={setHeld}
        papers={papers}
        onTask={onOpenPart}
      />
      <History task={task} onError={onError} onDoc={onDoc} />
    </>
  );

  const sealed = (
    <>
      <h1 className="text-[21px] leading-snug font-semibold">{task.title}</h1>
      <Stamps task={task} lists={lists} />

      <TaskWords task={task} onError={onError} onDoc={onDoc} onTask={onOpenPart} />

      <History task={task} onError={onError} onDoc={onDoc} />

      {(task.repeat || task.after) && (
        <Routine task={task.id} onError={onError} heading={<Section label={t("routine")} />} />
      )}
    </>
  );

  const shown = task.status === "open" ? body : sealed;
  const left = (whole?.open ?? 0) + (whole?.away ?? 0);

  const settled = (edge?: number) => (
    <Settled
      task={task}
      edge={edge}
      onComplete={onComplete}
      onDiscard={onDiscard}
      onReopen={onReopen}
      onStillOpen={onStillOpen}
      onErase={onErase}
      onFold={onFold}
      onReadAs={onReadAs}
      onOpenToAgents={onOpenToAgents}
      shaping={shaping}
      onShape={held ? undefined : setShaping}
      left={left}
    />
  );

  if (!expanded) {
    return (
      <Panel
        label={task.title}
        apart={apart}
        held={opened}
        onKeyDown={leave}
        onExpand={onExpand}
        onClose={onClose}
        footer={settled()}
      >
        {shown}
        {aside}
      </Panel>
    );
  }

  return (
    <main
      ref={opened}
      tabIndex={-1}
      onKeyDown={leave}
      className="flex flex-col overflow-hidden outline-none focus-visible:ring-2 focus-visible:ring-accent focus-visible:ring-inset"
    >
      <div data-tauri-drag-region className="h-9 shrink-0" />
      <div className="flex items-center gap-1 px-6 pb-2 text-[13px] text-faint">
        <button
          type="button"
          onClick={onCollapse}
          aria-label={from ? `${t("collapse")} — ${from}` : t("collapse")}
          className="-ml-2 rounded-md px-2 py-1 text-accent hover:bg-hover"
        >
          <span aria-hidden="true">‹</span> {from || t("collapse")}
        </button>
      </div>
      <div className="scroller @container flex-1 px-5 pb-6">
        <div
          ref={card}
          className="mx-auto grid w-fit gap-x-10 rounded-[10px] border border-hair bg-sheet px-8 pt-6 pb-10 shadow-lift @min-[1120px]:grid-cols-[minmax(0,720px)_320px] [&_.gutter]:px-0 [&_.task-words]:-mx-8"
        >
          <div className="min-w-0">{shown}</div>
          <div className="min-w-0">{aside}</div>
        </div>
      </div>
      {settled(edge)}
    </main>
  );
}

// The card's padding less the button's, less the footer's own: the label lines up with the title.
const LINED = 32 - 8 - 12;

function Settled({
  task,
  edge,
  onComplete,
  onDiscard,
  onReopen,
  onStillOpen,
  onErase,
  onFold,
  onReadAs,
  onOpenToAgents,
  shaping,
  onShape,
  left,
}: {
  task: Task;
  edge?: number;
  left?: number;
  onComplete: () => void;
  onDiscard: () => void;
  onReopen: () => void;
  onStillOpen: () => void;
  onErase: () => void;
  onFold: (away: boolean) => void;
  onReadAs: (how: "story" | "trace") => void;
  onOpenToAgents: (open: boolean) => void;
  shaping: boolean;
  onShape?: (shaping: boolean) => void;
}) {
  const [more, setMore] = useState<{ x: number; y: number } | null>(null);
  const owing = task.status === "open" && (left ?? 0) > 0;
  const open = task.status === "open";
  const reading = readingOf(task);
  const seat =
    "flex shrink-0 items-center gap-1 whitespace-nowrap rounded-md px-2 py-1 hover:bg-hover";

  const discard: Choice = {
    key: "discard",
    label: task.repeat ? t("endRepeat") : t("discardIt"),
    icon: "⊘",
    hint: task.repeat ? t("endRepeatWhy") : undefined,
    onPick: onDiscard,
  };
  const choices: Choice[] = open
    ? [
        ...(task.resolved?.drop
          ? [{ key: "done", label: t("markDone"), icon: "✓", onPick: onComplete }]
          : task.resolved
            ? [discard]
            : []),
        ...(agentNamed(task.created_by)
          ? []
          : [
              {
                key: "agents",
                label: t(task.open_to_agents ? "keepToMyself" : "letAgentFill"),
                icon: "⊚",
                hint: t(task.open_to_agents ? "keepToMyselfWhy" : "letAgentFillWhy"),
                apart: task.resolved !== undefined,
                onPick: () => onOpenToAgents(!task.open_to_agents),
              },
            ]),
      ]
    : [
        ...(reading === "routine"
          ? []
          : [
              {
                key: "move",
                label: t(reading === "trace" ? "moveToStories" : "moveToTrace"),
                icon: "◇",
                hint: t(reading === "trace" ? "moveToStoriesWhy" : "moveToTraceWhy"),
                onPick: () => onReadAs(reading === "trace" ? "story" : "trace"),
              },
            ]),
        ...(erasable(task)
          ? [
              {
                key: "erase",
                label: t("eraseIt"),
                icon: "✕",
                hint: t("eraseForGood"),
                danger: true,
                apart: reading !== "routine",
                onPick: onErase,
              },
            ]
          : []),
      ];

  return (
    <>
      <footer
        aria-label={t("taskDoings")}
        className="shrink-0 border-hair border-t bg-panel/70 px-3 py-1.5 text-[12.5px] text-soft backdrop-blur"
      >
        <div
          className="flex items-center gap-1"
          style={edge === undefined ? undefined : { paddingLeft: edge + LINED }}
        >
          {open ? (
            <>
              {task.resolved?.drop ? (
                <button
                  type="button"
                  onClick={onDiscard}
                  title={discard.hint}
                  className={`${seat} font-medium text-accent`}
                >
                  <span aria-hidden="true">⊘</span> {discard.label}
                </button>
              ) : (
                <>
                  {owing && (
                    <span className="shrink-0 px-2">{fill("partsLeft", String(left))}</span>
                  )}
                  <button
                    type="button"
                    onClick={onComplete}
                    title={owing ? t("closeLettingGoWhy") : undefined}
                    className={`${seat} font-medium text-accent`}
                  >
                    <span aria-hidden="true">✓</span> {owing ? t("closeLettingGo") : t("markDone")}
                  </button>
                </>
              )}
              {task.resolved ? (
                <button
                  type="button"
                  onClick={onStillOpen}
                  title={fill("stillOpenIt", task.title)}
                  className={`${seat} hover:text-ink`}
                >
                  <span aria-hidden="true">↩</span> {t("stillOpen")}
                </button>
              ) : (
                <button
                  type="button"
                  onClick={onDiscard}
                  title={discard.hint}
                  className={`${seat} hover:text-ink`}
                >
                  <span aria-hidden="true">⊘</span> {discard.label}
                </button>
              )}
            </>
          ) : (
            <>
              <button type="button" onClick={onReopen} className={`${seat} hover:text-ink`}>
                <span aria-hidden="true">↺</span> {t("reopenIt")}
              </button>
              {task.status !== "dropped" && (
                <button
                  type="button"
                  onClick={() => onFold(!task.hidden)}
                  className={`${seat} hover:text-ink`}
                >
                  <span aria-hidden="true">{task.hidden ? "⊕" : "⊖"}</span>{" "}
                  {t(task.hidden ? "showIt" : "hideIt")}
                </button>
              )}
            </>
          )}
          {open && onShape && (
            <button
              type="button"
              aria-pressed={shaping}
              title={t("shapeWhy")}
              onClick={() => onShape(!shaping)}
              className={`${seat} ml-auto ${shaping ? "font-medium text-accent" : "hover:text-ink"}`}
            >
              <span aria-hidden="true">{shaping ? "✓" : "✎"}</span>{" "}
              {t(shaping ? "shapeDone" : "shapeEdit")}
            </button>
          )}
          {choices.length > 0 && (
            <button
              type="button"
              aria-label={t("more")}
              title={t("more")}
              aria-haspopup="menu"
              aria-expanded={more !== null}
              // The menu closes on any mousedown outside it, this button included, so a mouse
              // toggles here and only a keyboard's click — detail 0 — reaches onClick.
              onMouseDown={(e) => {
                const box = e.currentTarget.getBoundingClientRect();
                setMore(more ? null : { x: box.right - 8, y: box.top - 4 });
              }}
              onClick={(e) => {
                if (e.detail !== 0) return;
                const box = e.currentTarget.getBoundingClientRect();
                setMore(more ? null : { x: box.right - 8, y: box.top - 4 });
              }}
              className={`${open && onShape ? "" : "ml-auto "}flex h-6 w-6 shrink-0 items-center justify-center rounded-md text-faint hover:bg-hover hover:text-ink`}
            >
              <span aria-hidden="true">⋯</span>
            </button>
          )}
        </div>
      </footer>
      {more && (
        <Menu
          at={more}
          up
          choices={choices}
          label={t("taskDoings")}
          onClose={() => setMore(null)}
        />
      )}
    </>
  );
}

function Title({ task, onRename }: { task: Task; onRename: (title: string) => void }) {
  const [text, setText] = useState(task.title);
  const dropped = useRef(false);

  useEffect(() => setText(task.title), [task.id, task.title]);

  const settle = () => {
    if (dropped.current) {
      dropped.current = false;
      setText(task.title);
      return;
    }
    const kept = text.trim();
    if (kept && kept !== task.title) onRename(kept);
    else setText(task.title);
  };

  return (
    <textarea
      rows={1}
      value={text}
      aria-label={t("fieldTitle")}
      onChange={(e) => setText(e.target.value)}
      onBlur={settle}
      onKeyDown={(e) => {
        if (e.key === "Enter") {
          e.preventDefault();
          e.currentTarget.blur();
        }
        if (e.key === "Escape") {
          dropped.current = true;
          e.currentTarget.blur();
        }
      }}
      className="mb-3 field-sizing-content w-full resize-none rounded-md bg-transparent text-[21px] leading-snug font-semibold -tracking-[0.01em] outline-none hover:bg-hover focus:bg-hover"
    />
  );
}

function Stamps({ task, lists }: { task: Task; lists: List[] }) {
  const seals: { key: string; text: string; look?: string }[] = [];
  const list = lists.find((one) => one.id === task.list);

  if (list) seals.push({ key: "list", text: `▤ ${list.name}`, look: "bg-mark-list" });
  for (const tag of task.tags ?? []) {
    seals.push({ key: `tag-${tag}`, text: `◈ ${tag}`, look: "bg-mark-tag" });
  }
  if (placed(task.priority)) {
    seals.push({ key: "priority", text: `⊞ ${said(task.priority)}`, look: "bg-accent-soft" });
  }
  if (task.date) seals.push({ key: "date", text: whenLabel(task.date) });
  if (task.deadline) seals.push({ key: "deadline", text: `⚑ ${whenLabel(task.deadline)}` });
  if (task.repeat) seals.push({ key: "repeat", text: `↻ ${cadence(task.repeat)}` });

  const closed = task.completed_at ? wroteAt(task.completed_at) : "";

  return (
    <>
      <p className="mt-2 text-[12.5px] text-faint">
        <span aria-hidden="true">{task.status === "dropped" ? "⨯" : "▣"}</span>{" "}
        {t(task.status === "dropped" ? "dropped" : "done")}
        {closed && ` · ${closed}`}
        {task.read_as &&
          readingOf(task) !== "routine" &&
          ` · ${t(task.read_as === "story" ? "keptAsStory" : "readAsTraceNow")}`}
        {task.open_to_agents && ` · ${t("wasOpenToAgents")}`}
        {signedBy(task.created_by, task.created_via) && (
          <span className="text-hue-teal" title={hostedOn(task.created_by)}>
            {" · "}
            {signedBy(task.created_by, task.created_via)}
          </span>
        )}
        {task.resolved && (
          <span className="text-hue-teal">
            {" · "}
            {saidBy(task)}
          </span>
        )}
      </p>
      {seals.length > 0 && (
        <div className="mt-2.5 flex flex-wrap gap-1.5">
          {seals.map((seal) => (
            <span
              key={seal.key}
              className={`rounded-full px-2.5 py-0.5 text-[11.5px] text-soft ${seal.look ?? "bg-hover"}`}
            >
              {seal.text}
            </span>
          ))}
        </div>
      )}
    </>
  );
}

function Facts({ task, from }: { task: Task; from: string }) {
  const days = Math.max(0, -daysFrom(from));
  const kept: [string, string][] = [
    [String(days), t("factDays")],
    [`${task.volume?.steps_done ?? 0}/${task.volume?.steps ?? 0}`, t("factSteps")],
    [String(task.volume?.journal ?? 0), t("factJournal")],
    [String(task.volume?.refs ?? 0), t("factRefs")],
  ];
  return (
    <>
      <Section label={t("carries")} />
      <dl className="grid grid-cols-2 gap-1.5">
        {kept.map(([count, said]) => (
          <div key={said} className="rounded-[10px] border border-hair px-2.5 py-1.5">
            <dt className="text-[13px] leading-tight font-semibold tabular-nums">{count}</dt>
            <dd className="text-[10.5px] text-faint">{said}</dd>
          </div>
        ))}
      </dl>
    </>
  );
}

function journalled(task: Task): { count: number; last: string } {
  const kept = (task.log ?? []).filter((entry) => entry.body.trim());
  const last = kept.reduce<(typeof kept)[number] | undefined>(
    (latest, entry) => (latest && latest.at >= entry.at ? latest : entry),
    undefined,
  );
  return { count: kept.length, last: last ? wroteAt(last.at, last.tz) : "" };
}
