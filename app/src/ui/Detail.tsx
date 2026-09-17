import { useEffect, useRef, useState } from "react";
import { type Change, erasable, type List, readingOf, type Task } from "../core";
import { cadence, daysFrom, stamped, whenLabel, wroteAt } from "../format";
import { fill, t } from "../locales";
import { composed } from "../markdown";
import { placed, said } from "../quadrants";
import { agentNamed } from "../who";
import Composed from "./Composed";
import Fields from "./Fields";
import Journal from "./Journal";
import Left from "./Left";
import Menu, { type Choice } from "./Menu";
import Prose from "./Prose";
import Routine from "./Routine";
import Steps from "./Steps";
import Trail from "./Trail";

interface Props {
  task: Task;
  lists: List[];
  known: string[];
  apart?: string;
  expanded: boolean;
  from?: string;
  onExpand: () => void;
  onCollapse: () => void;
  onPatch: (change: Change) => void;
  onStep: (text: string, step?: string) => void;
  onMark: (step: string, done: boolean) => void;
  onDropStep: (step: string) => void;
  onLog: (body: string, entry?: string) => void;
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
}

export default function Detail({
  task,
  lists,
  known,
  apart,
  expanded,
  from,
  onExpand,
  onCollapse,
  onPatch,
  onStep,
  onMark,
  onDropStep,
  onLog,
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
}: Props) {
  const opened = useRef<HTMLElement>(null);
  useEffect(() => {
    opened.current?.focus({ preventScroll: true });
  }, [task.id]);

  const leave = (event: React.KeyboardEvent) => {
    if (event.key !== "Escape") return;
    const at = event.target as HTMLElement;
    if (at.isContentEditable || at.closest("input, textarea, select")) return;
    event.preventDefault();
    onClose();
  };

  const aside = (
    <>
      <Trail
        task={task.id}
        moved={[
          task.status,
          task.read_as ?? "",
          task.completed_at ?? "",
          task.hidden ? "hidden" : "",
          task.open_to_agents ? "open" : "",
          task.log?.length ?? task.volume?.journal ?? 0,
          task.steps?.length ?? task.volume?.steps ?? 0,
          task.steps?.filter((step) => step.done).length ?? task.volume?.steps_done ?? 0,
          task.description ? "described" : "",
          task.title,
        ].join("|")}
        lists={lists}
        onError={onError}
        heading={<Section label={t("trail")} />}
        before={(from) => <Facts task={task} from={from} />}
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
      {task.status === "open" && agentNamed(task.created_by) && (
        <p className="-mt-1.5 mb-3 text-[11.5px] text-hue-teal">
          {fill("agentWrote", agentNamed(task.created_by) as string)}
        </p>
      )}
      {task.resolved && (
        <p className="mt-3 mb-4 flex items-center gap-2 rounded-md border border-hue-teal/40 bg-hue-teal/10 px-2.5 py-1.5 text-[12.5px] font-medium text-hue-teal">
          <span aria-hidden="true">◆</span>
          {agentNamed(task.resolved.by)
            ? fill("agentNamedSaidDone", agentNamed(task.resolved.by) as string)
            : t("agentSaidDone")}
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

      <Section label={t("description")} />
      <Prose
        value={task.description ?? ""}
        hint={t("describeIt")}
        label={t("description")}
        beside={expanded}
        catches
        onError={onError}
        onDoc={onDoc}
        onWhole={expanded ? undefined : onExpand}
        onWrite={(description) => onPatch({ description })}
      />

      <Section
        label={t("steps")}
        note={
          task.volume?.steps ? `${task.volume.steps_done ?? 0}/${task.volume.steps}` : undefined
        }
      />
      <Steps steps={task.steps ?? []} onWrite={onStep} onMark={onMark} onDrop={onDropStep} />

      <Section
        label={t("journal")}
        note={task.volume?.journal ? String(task.volume.journal) : undefined}
      />
      <Journal
        entries={task.log ?? []}
        steps={task.steps?.map((one) => one.text)}
        onError={onError}
        onDoc={onDoc}
        onWhole={expanded ? undefined : onExpand}
        onWrite={onLog}
      />
    </>
  );

  const sealed = (
    <>
      <h1 className="text-[21px] leading-snug font-semibold">{task.title}</h1>
      <Stamps task={task} lists={lists} />

      {task.description?.trim() && (
        <>
          <Section label={t("description")} />
          <Composed
            label={t("description")}
            onError={onError}
            onDoc={onDoc}
            html={composed(
              task.description,
              task.steps?.map((one) => one.text),
            )}
            className="prose px-1.5 py-1 text-[13px] leading-relaxed"
          />
        </>
      )}

      {task.steps && task.steps.length > 0 && (
        <>
          <Section
            label={t("steps")}
            note={`${task.volume?.steps_done ?? 0}/${task.volume?.steps ?? task.steps.length}`}
          />
          <ul className="flex flex-col gap-1.5">
            {task.steps.map((step) => (
              <li key={step.id} className="flex items-start gap-2.5 text-[13px]">
                <span
                  aria-hidden="true"
                  className={`pt-px text-[12.5px] ${step.done ? "text-accent" : "text-faint"}`}
                >
                  {step.done ? "✓" : "▫"}
                </span>
                <span className={step.done ? "text-faint line-through" : "text-soft"}>
                  {step.text}
                </span>
              </li>
            ))}
          </ul>
        </>
      )}

      {task.log && task.log.length > 0 && (
        <>
          <Section label={t("journal")} note={String(task.volume?.journal ?? task.log.length)} />
          <ul className="flex flex-col gap-3">
            {task.log.map((entry) => (
              <li key={entry.id}>
                <span className="block text-[11.5px] tabular-nums text-faint">
                  {wroteAt(entry.at, entry.tz)}
                </span>
                <Composed
                  label={t("journal")}
                  onError={onError}
                  onDoc={onDoc}
                  html={composed(
                    entry.body,
                    task.steps?.map((one) => one.text),
                  )}
                  className="prose text-[13px] leading-relaxed"
                />
              </li>
            ))}
          </ul>
        </>
      )}

      {(task.repeat || task.after) && (
        <Routine task={task.id} onError={onError} heading={<Section label={t("routine")} />} />
      )}

      {!expanded && aside}
    </>
  );

  const shown = task.status === "open" ? body : sealed;

  if (expanded) {
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
          <div className="mx-auto grid w-fit gap-x-10 rounded-[10px] border border-hair bg-sheet px-8 pt-6 pb-10 shadow-lift @min-[1120px]:grid-cols-[minmax(0,720px)_320px]">
            <div className="min-w-0">{shown}</div>
            <div className="min-w-0">{aside}</div>
          </div>
        </div>
        <Settled
          task={task}
          wide
          onComplete={onComplete}
          onDiscard={onDiscard}
          onReopen={onReopen}
          onStillOpen={onStillOpen}
          onErase={onErase}
          onFold={onFold}
          onReadAs={onReadAs}
          onOpenToAgents={onOpenToAgents}
        />
      </main>
    );
  }

  return (
    <aside
      ref={opened as React.RefObject<HTMLElement>}
      tabIndex={-1}
      onKeyDown={leave}
      className={`absolute top-11 bottom-3 z-20 flex w-[380px] flex-col overflow-hidden rounded-[10px] border border-hair bg-panel shadow-lift outline-none focus-visible:ring-2 focus-visible:ring-accent focus-visible:ring-inset ${apart ?? "right-3"}`}
    >
      <div className="flex items-center justify-end gap-1 px-3 pt-2.5 text-[13px] text-faint">
        <button
          type="button"
          onClick={onExpand}
          title={t("expand")}
          aria-label={t("expand")}
          className="flex h-6 w-6 items-center justify-center rounded-md hover:bg-hover hover:text-accent"
        >
          <span aria-hidden="true">⤢</span>
        </button>
        <button
          type="button"
          onClick={onClose}
          title={t("closePanel")}
          aria-label={t("closePanel")}
          aria-keyshortcuts="Escape"
          className="flex h-6 w-6 items-center justify-center rounded-md hover:bg-hover hover:text-ink"
        >
          <span aria-hidden="true">✕</span>
        </button>
      </div>
      <div className="scroller flex-1 px-5 pt-1.5 pb-7">{shown}</div>
      <Settled
        task={task}
        onComplete={onComplete}
        onDiscard={onDiscard}
        onReopen={onReopen}
        onStillOpen={onStillOpen}
        onErase={onErase}
        onFold={onFold}
        onReadAs={onReadAs}
        onOpenToAgents={onOpenToAgents}
      />
    </aside>
  );
}

function Settled({
  task,
  wide,
  onComplete,
  onDiscard,
  onReopen,
  onStillOpen,
  onErase,
  onFold,
  onReadAs,
  onOpenToAgents,
}: {
  task: Task;
  wide?: boolean;
  onComplete: () => void;
  onDiscard: () => void;
  onReopen: () => void;
  onStillOpen: () => void;
  onErase: () => void;
  onFold: (away: boolean) => void;
  onReadAs: (how: "story" | "trace") => void;
  onOpenToAgents: (open: boolean) => void;
}) {
  const [more, setMore] = useState<{ x: number; y: number } | null>(null);
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
        ...(task.resolved ? [discard] : []),
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
          className={
            wide
              ? "mx-auto flex w-full max-w-[720px] items-center gap-1"
              : "flex items-center gap-1"
          }
        >
          {open ? (
            <>
              <button
                type="button"
                onClick={onComplete}
                className={`${seat} font-medium text-accent`}
              >
                <span aria-hidden="true">✓</span> {t("markDone")}
              </button>
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
              className="ml-auto flex h-6 w-6 shrink-0 items-center justify-center rounded-md text-faint hover:bg-hover hover:text-ink"
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
        {agentNamed(task.created_by) && (
          <span className="text-hue-teal">
            {" · "}
            {fill("agentWrote", agentNamed(task.created_by) as string)}
          </span>
        )}
        {task.resolved && (
          <span className="text-hue-teal">
            {" · "}
            {agentNamed(task.resolved.by)
              ? fill("agentNamedSaidDone", agentNamed(task.resolved.by) as string)
              : t("agentSettled")}
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

function Section({ label, note }: { label: string; note?: string }) {
  return (
    <div className="mt-5 mb-1.5 flex items-center gap-2.5 text-[11.5px] font-semibold tracking-[0.05em] text-faint uppercase">
      <span>{label}</span>
      <span className="h-px flex-1 bg-hair" />
      {note && <span>{note}</span>}
    </div>
  );
}
