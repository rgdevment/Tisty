import { useCallback, useEffect, useState } from "react";
import {
  capture,
  complete,
  dropStep,
  markStep,
  patch,
  snapshot,
  writeLog,
  writeStep,
  type Change,
  type Snapshot,
  type Task,
} from "./core";
import { adopt } from "./locales";
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

  const act = (work: Promise<unknown>) => {
    setError(null);
    work.then(load).catch((e) => setError(saidPlainly(e)));
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
          expanded
          onExpand={() => remember("sheet")}
          onCollapse={() => remember("columns")}
          onPatch={(change: Change) => act(patch(task.id, change))}
          onStep={(text, step) => act(writeStep(task.id, text, step))}
          onMark={(step, done) => act(markStep(task.id, step, done))}
          onDropStep={(step) => act(dropStep(task.id, step))}
          onLog={(body, entry) => act(writeLog(task.id, body, entry))}
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
          onSelect={setSelected}
          onComplete={chosen.named === "archive" ? undefined : (id) => act(complete(id))}
          above={
            chosen.named === "tags" || chosen.tags?.length ? (
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
            <Search key="search" onFound={setFound} />
          ) : chosen.named === "archive" ? (
            <Search key="archive" fixed="archived" onFound={setFound} />
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
          expanded={false}
          onExpand={() => remember("sheet")}
          onCollapse={() => remember("columns")}
          onPatch={(change: Change) => act(patch(task.id, change))}
          onStep={(text, step) => act(writeStep(task.id, text, step))}
          onMark={(step, done) => act(markStep(task.id, step, done))}
          onDropStep={(step) => act(dropStep(task.id, step))}
          onLog={(body, entry) => act(writeLog(task.id, body, entry))}
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
