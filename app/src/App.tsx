import { useCallback, useEffect, useState } from "react";
import { capture, complete, snapshot, type Snapshot } from "./core";
import { adopt, t } from "./locales";
import Detail from "./ui/Detail";
import Sidebar from "./ui/Sidebar";
import TaskList from "./ui/TaskList";

type Mode = "columns" | "sheet";

export default function App() {
  const [data, setData] = useState<Snapshot | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [text, setText] = useState("");
  const [selected, setSelected] = useState<string | undefined>();
  const [mode, setMode] = useState<Mode>("columns");

  useTheme();

  const load = useCallback(() => {
    snapshot()
      .then((fresh) => {
        adopt(fresh.locale);
        setData(fresh);
      })
      .catch((e) => setError(String(e)));
  }, []);

  useEffect(() => {
    load();
    window.addEventListener("focus", load);
    return () => window.removeEventListener("focus", load);
  }, [load]);

  if (!data) return null;

  const task = data.tasks.find((candidate) => candidate.id === selected);
  const open = task !== undefined;

  const act = (work: Promise<unknown>) => {
    setError(null);
    work.then(load).catch((e) => setError(String(e)));
  };

  return (
    <div
      className="grid h-full font-sans"
      style={{
        gridTemplateColumns: open && mode === "columns" ? "232px minmax(0,1fr) 380px" : "232px minmax(0,1fr)",
      }}
    >
      <Sidebar tasks={data.tasks} lists={data.lists} />

      {open && mode === "sheet" ? (
        <Detail
          task={task}
          lists={data.lists}
          expanded
          onExpand={() => setMode("sheet")}
          onCollapse={() => setMode("columns")}
        />
      ) : (
        <TaskList
          tasks={data.tasks}
          lists={data.lists}
          title={t("today")}
          selected={selected}
          centred={!open}
          onSelect={setSelected}
          onComplete={(id) => act(complete(id))}
        >
          <form
            onSubmit={(e) => {
              e.preventDefault();
              act(capture(text).then(() => setText("")));
            }}
            className="mt-1.5 w-full"
          >
            <input
              value={text}
              onChange={(e) => setText(e.target.value)}
              aria-label={t("capture")}
              className="w-full rounded-[9px] border border-line bg-bg px-3.5 py-2.5 text-sm outline-none focus:border-accent focus:ring-[3px] focus:ring-accent-soft"
            />
          </form>
          {error && <p className="mt-2 px-2.5 text-xs text-urgent">{error}</p>}
        </TaskList>
      )}

      {open && mode === "columns" && (
        <Detail
          task={task}
          lists={data.lists}
          expanded={false}
          onExpand={() => setMode("sheet")}
          onCollapse={() => setMode("columns")}
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
