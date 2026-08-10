import { useCallback, useEffect, useState } from "react";
import {
  attach,
  capture,
  complete,
  discard,
  fold,
  dropStep,
  markStep,
  patch,
  reopen,
  reorder,
  snapshot,
  writeLog,
  writeStep,
  type Change,
  type Snapshot,
  type Task,
} from "./core";
import { handTo, whenFilesLand } from "./dropped";
import { adopt, t } from "./locales";
import { saidPlainly } from "./refusal";
import { accepts, asView, invite, title, type Chosen } from "./views";
import CaptureField from "./ui/CaptureField";
import Detail from "./ui/Detail";
import Notice from "./ui/Notice";
import Search from "./ui/Search";
import Sidebar from "./ui/Sidebar";
import Tags from "./ui/Tags";
import TaskList from "./ui/TaskList";
import WindowChrome from "./ui/WindowChrome";

type Mode = "columns" | "sheet";

export default function App() {
  const [data, setData] = useState<Snapshot | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [selected, setSelected] = useState<string | undefined>();
  const [captured, setCaptured] = useState<Task | undefined>();
  const [reveal, setReveal] = useState<string | undefined>();
  const [mode, setMode] = useState<Mode>(
    () => (localStorage.getItem("detail") as Mode) ?? "columns",
  );
  const [chosen, setChosen] = useState<Chosen>({ named: "today" });
  const [found, setFound] = useState<Task[] | null>(null);
  const [held, setHeld] = useState<Task | undefined>();
  const dismiss = useCallback(() => setCaptured(undefined), []);

  useTheme();

  const load = useCallback(() => {
    snapshot(asView(chosen))
      .then((fresh) => {
        adopt(fresh.locale);
        setData(fresh);
      })
      .catch((e) => setError(saidPlainly(e)));
  }, [chosen]);

  useEffect(() => {
    load();
    window.addEventListener("focus", load);
    return () => window.removeEventListener("focus", load);
  }, [load]);

  useEffect(
    () =>
      whenFilesLand((target, paths) => {
        setError(null);
        Promise.all(paths.map((one) => attach(one)))
          .then((written) => {
            // The field can be gone by the time the copy finishes; saying so
            // beats leaving the file in the store with nothing pointing at it.
            const put = handTo(target, written.join("\n\n"));
            if (!put) setError(t("attachmentLost"));
          })
          .catch((e) => setError(saidPlainly(e)));
      }),
    [],
  );

  if (!data) return null;

  const fresh =
    data.tasks.find((candidate) => candidate.id === selected) ??
    found?.find((candidate) => candidate.id === selected);
  const task = fresh ?? (held?.id === selected ? held : undefined) ?? undefined;
  const open = task !== undefined;
  if (fresh && fresh !== held) setHeld(fresh);

  const remember = (next: Mode) => {
    localStorage.setItem("detail", next);
    setMode(next);
  };

  // An action that pushes the task out of the view leaves `fresh` empty, so
  // the held copy has to come from the answer or the panel shows a stale one.
  const act = (work: Promise<Task>) => {
    setError(null);
    work
      .then((one) => {
        setHeld(one);
        load();
      })
      .catch((e) => setError(saidPlainly(e)));
  };

  return (
    <div
      className="grid h-full font-sans"
      style={{
        gridTemplateColumns: open && mode === "columns" ? "232px minmax(0,1fr) 380px" : "232px minmax(0,1fr)",
      }}
    >
      <WindowChrome />

      {error && (
        <p className="pointer-events-none fixed inset-x-0 top-11 z-40 mx-auto w-fit rounded-md bg-urgent/12 px-3 py-1.5 text-xs text-urgent">
          {error}
        </p>
      )}

      {captured && (
        <Notice
          key={captured.id}
          task={captured}
          lists={data.lists}
          onOpen={() => {
            setSelected(captured.id);
            setReveal(captured.id);
            dismiss();
          }}
          onDismiss={dismiss}
        />
      )}

      <Sidebar
        lists={data.lists}
        counts={data.counts}
        onFile={(task, list) => act(reorder(task, list ? { list } : { inbox: true }))}
        chosen={chosen}
        onChoose={(next) => {
          setChosen(next);
          setSelected(undefined);
          setFound(null);
        }}
      />

      {open && mode === "sheet" ? (
        <Detail
          key={task.id}
          task={task}
          lists={data.lists}
          known={data.tags.map((one) => one.tag)}
          refs={data.refs ?? []}
          expanded
          onExpand={() => remember("sheet")}
          onCollapse={() => remember("columns")}
          onPatch={(change: Change) => act(patch(task.id, change))}
          onStep={(text, step) => act(writeStep(task.id, text, step))}
          onMark={(step, done) => act(markStep(task.id, step, done))}
          onDropStep={(step) => act(dropStep(task.id, step))}
          onLog={(body, entry) => act(writeLog(task.id, body, entry))}
          onDiscard={() => act(discard(task.id))}
          onReopen={() => act(reopen(task.id))}
          onError={(e) => setError(saidPlainly(e))}
        />
      ) : (
        <TaskList
          tasks={found ?? data.tasks}
          lists={data.lists}
          title={title(chosen, data.lists)}
          selected={selected}
          fresh={captured?.id}
          reveal={reveal}
          centred={!open}
          byMonth={chosen.named === "archive" && found === null}
          onSelect={setSelected}
          onComplete={chosen.named === "archive" ? undefined : (id) => act(complete(id))}
          onFold={
            chosen.named === "archive" ? (id, away) => act(fold(id, away)) : undefined
          }
          onDrop={
            // The archive is sorted by when it closed and a search by how it
            // matched; dragging inside either would promise an order it has not.
            chosen.named === "archive" || found !== null
              ? undefined
              : (task, after, before) => act(reorder(task, { after, before }))
          }
          above={
            chosen.named === "archive" && (data.counts.folded || chosen.folded) ? (
              <button
                onClick={() => setChosen({ named: "archive", folded: !chosen.folded })}
                className="px-2.5 pb-1.5 text-xs text-faint hover:text-ink"
              >
                {chosen.folded
                  ? `⊕ ${t("backToArchive")}`
                  : `⊖ ${data.counts.folded} ${t("folded")}`}
              </button>
            ) : chosen.named === "tags" || chosen.tags?.length ? (
              <Tags
                tags={data.tags}
                chosen={chosen.tags ?? []}
                onToggle={(tag) => {
                  const now = chosen.tags ?? [];
                  const next = now.includes(tag)
                    ? now.filter((t) => t !== tag)
                    : [...now, tag];
                  setChosen({ named: "tags", tags: next });
                  setSelected(undefined);
                }}
              />
            ) : undefined
          }
        >
          {chosen.named === "search" ? (
            <Search key="search" onFound={setFound} onError={setError} />
          ) : chosen.named === "archive" ? (
            <Search key="archive" fixed="archived" onFound={setFound} onError={setError} />
          ) : accepts(chosen) ? (
          <CaptureField
            invite={invite(chosen, data.lists)}
            lists={data.lists}
            tags={data.tags}
            onCapture={(written, edits) => {
              setError(null);
              return capture(written, asView(chosen), edits).then((task) => {
                setCaptured(task);
                load();
                return task;
              });
            }}
            onError={setError}
          />
          ) : null}
        </TaskList>
      )}

      {open && mode === "columns" && (
        <Detail
          key={task.id}
          task={task}
          lists={data.lists}
          known={data.tags.map((one) => one.tag)}
          refs={data.refs ?? []}
          expanded={false}
          onExpand={() => remember("sheet")}
          onCollapse={() => remember("columns")}
          onPatch={(change: Change) => act(patch(task.id, change))}
          onStep={(text, step) => act(writeStep(task.id, text, step))}
          onMark={(step, done) => act(markStep(task.id, step, done))}
          onDropStep={(step) => act(dropStep(task.id, step))}
          onLog={(body, entry) => act(writeLog(task.id, body, entry))}
          onDiscard={() => act(discard(task.id))}
          onReopen={() => act(reopen(task.id))}
          onError={(e) => setError(saidPlainly(e))}
        />
      )}
    </div>
  );
}

function useTheme() {
  useEffect(() => {
    const dark = window.matchMedia("(prefers-color-scheme: dark)");
    const paint = () =>
      document.documentElement.setAttribute("data-theme", dark.matches ? "dark" : "light");

    paint();
    dark.addEventListener("change", paint);
    return () => dark.removeEventListener("change", paint);
  }, []);
}
