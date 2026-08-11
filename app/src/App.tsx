import { useCallback, useEffect, useRef, useState } from "react";
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
  snapshot,
  writeLog,
  writeStep,
  type Change,
  type Snapshot,
  type Found,
  type Task,
} from "./core";
import { listen } from "@tauri-apps/api/event";
import { heard, play } from "./chime";
import { carrying } from "./carrying";
import { settleIn, syncState } from "./core";
import { handTo, whenFilesLand } from "./dropped";
import { adopt, fill, t } from "./locales";
import { saidPlainly } from "./refusal";
import {
  accepts,
  asView,
  invite,
  nothing,
  SLICES,
  title,
  type Chosen,
  type Slice,
} from "./views";
import CaptureField from "./ui/CaptureField";
import Closing from "./ui/Closing";
import Detail from "./ui/Detail";
import About from "./ui/About";
import Keeping from "./ui/Keeping";
import Notice from "./ui/Notice";
import Search from "./ui/Search";
import Sidebar from "./ui/Sidebar";
import Tags from "./ui/Tags";
import TaskList from "./ui/TaskList";
import Welcome from "./ui/Welcome";
import WindowChrome from "./ui/WindowChrome";

type Mode = "columns" | "sheet";

export default function App() {
  const [data, setData] = useState<Snapshot | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [selected, setSelected] = useState<string | undefined>();
  const [captured, setCaptured] = useState<Task | undefined>();
  // Everything that happens is answered by something moving on screen, which
  // is no answer at all to someone who is not looking at it.
  const [aloud, setAloud] = useState("");
  const twice = useRef(0);
  // A reader skips a live region whose text did not change, so completing two
  // tasks with the same title said it once. The zero-width space alternates.
  const say = (words: string) => {
    twice.current += 1;
    setAloud(words + "\u200b".repeat(twice.current % 2));
  };
  const [reveal, setReveal] = useState<string | undefined>();
  // Closing the panel used to drop the keyboard on the body: the row it was
  // opened from takes it back once the list has drawn without the panel.
  const [returning, setReturning] = useState<string | null>(null);
  const [mode, setMode] = useState<Mode>(
    () => (localStorage.getItem("detail") as Mode) ?? "columns",
  );
  // Opens on today, and on whatever slice was last chosen: «all» is forty rows
  // the moment you arrive, which is the wall «today» existed to avoid.
  const [chosen, setChosen] = useState<Chosen>(() => ({
    named: "tasks",
    slice: (localStorage.getItem("tisty.slice") as Slice) ?? "today",
  }));
  const [found, setFound] = useState<Found | null>(null);
  const [held, setHeld] = useState<Task | undefined>();
  const [greet, setGreet] = useState(false);
  const [leaving, setLeaving] = useState(false);
  const [settling, setSettling] = useState(true);
  const dismiss = useCallback(() => setCaptured(undefined), []);
  const carries = useRef<ReturnType<typeof carrying>>(null);

  useTheme();

  const load = useCallback(() => {
    snapshot(asView(chosen))
      .then((fresh) => {
        adopt(fresh.locale);
        setData(fresh);
      })
      .catch((e) => setError(saidPlainly(e)));
  }, [chosen]);

  // Before the first read: a fresh install may find a store another machine
  // has been writing, and a cache an older version built.
  useEffect(() => {
    settleIn()
      .then((done) => done.brought && latest.current())
      .catch(() => {})
      .finally(() => setSettling(false));
  }, []);

  useEffect(() => {
    load();
    window.addEventListener("focus", load);
    return () => window.removeEventListener("focus", load);
  }, [load]);

  // Through a ref, and started once: a carrier per view would reset its
  // timers, and one holding the first `load` would reload the first view.
  const latest = useRef(load);
  latest.current = load;
  useEffect(() => {
    const carrier = carrying(() => latest.current());
    carries.current = carrier;
    return () => carrier.stop();
  }, []);

  useEffect(() => {
    syncState()
      .then((state) => setGreet(!state.asked))
      .catch(() => {});
  }, []);

  useEffect(() => {
    if (!returning) return;
    document.querySelector<HTMLElement>(`[data-row="${returning}"]`)?.focus();
    setReturning(null);
  }, [returning]);

  useEffect(() => {
    const stop = listen("closing", () => setLeaving(true));
    // Captured from the quick window, which the main one never hears about.
    const caught = listen("captured", () => latest.current());
    const sound = listen<unknown>("chime", (rung) => {
      if (heard(rung.payload)) play(rung.payload);
    });
    return () => {
      stop.then((off) => off()).catch(() => {});
      caught.then((off) => off()).catch(() => {});
      sound.then((off) => off()).catch(() => {});
    };
  }, []);

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

  // Not `null`: with the frame drawn by us, an empty render leaves a window
  // with no way to close it and nothing saying why.
  if (!data) {
    return (
      <div className="grid h-full font-sans" style={{ gridTemplateColumns: "1fr" }}>
        <WindowChrome />
        {error && (
          <p className="mt-16 px-6 text-center text-xs text-urgent">{error}</p>
        )}
      </div>
    );
  }

  const fresh =
    data.tasks.find((candidate) => candidate.id === selected) ??
    found?.tasks.find((candidate) => candidate.id === selected);
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
        carries.current?.changed();
      })
      .catch((e) => setError(saidPlainly(e)));
  };

  const shown = found?.tasks ?? (chosen.named === "search" ? [] : data.tasks);

  const shut = () => {
    setReturning(selected ?? null);
    setSelected(undefined);
  };

  return (
    <div
      className="grid h-full font-sans motion-safe:transition-[grid-template-columns] motion-safe:duration-150"
      style={{
        gridTemplateColumns:
          open && mode === "columns" && chosen.named !== "keeping"
            ? "232px minmax(0,1fr) 380px"
            : "232px minmax(0,1fr)",
      }}
    >
      <WindowChrome />

      <p role="status" aria-live="polite" className="sr-only">
        {aloud}
      </p>

      {error && (
        <div
          role="alert"
          className="fixed inset-x-0 top-11 z-40 mx-auto flex w-fit max-w-[70%] items-start gap-2 rounded-md bg-urgent/12 px-3 py-1.5 text-xs text-urgent"
        >
          {/* Selectable and closable: it used to sit there unreadable and
              unremovable until some later action happened to succeed. */}
          <span className="select-text">{error}</span>
          <button
            type="button"
            aria-label={t("close")}
            onClick={() => setError(null)}
            className="-mr-1 shrink-0 rounded px-1 hover:bg-urgent/15"
          >
            ✕
          </button>
        </div>
      )}

      {settling && !error && (
        <p className="pointer-events-none fixed inset-x-0 top-11 z-40 mx-auto w-fit rounded-md bg-accent-soft px-3 py-1.5 text-xs text-accent">
          {t("settlingIn")}
        </p>
      )}

      {leaving && (
        <Closing
          onDismiss={() => setLeaving(false)}
          onError={(e) => setError(saidPlainly(e))}
        />
      )}

      {greet && (
        <Welcome
          onDone={() => {
            setGreet(false);
            carries.current?.recheck();
          }}
        />
      )}

      {captured && (
        <Notice
          key={captured.id}
          task={captured}
          lists={data.lists}
          elsewhere={!data.tasks.some((one) => one.id === captured.id)}
          onOpen={() => {
            // Taking it to a view that shows it: selecting a task the list is
            // not drawing would open a panel next to a list without it.
            if (!data.tasks.some((one) => one.id === captured.id)) {
              setChosen({ named: "tasks", slice: "all" });
            }
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
          setError(null);
        }}
      />

      {chosen.named === "aboutScreen" ? (
        <About onError={(e) => setError(saidPlainly(e))} />
      ) : chosen.named === "keeping" ? (
        <Keeping
          onChanged={() => {
            load();
            carries.current?.recheck();
          }}
        />
      ) : open && mode === "sheet" ? (
        <Detail
          key={task.id}
          task={task}
          lists={data.lists}
          known={data.tags.map((one) => one.tag)}
          expanded
          from={title(chosen, data.lists)}
          onExpand={() => remember("sheet")}
          onCollapse={() => remember("columns")}
          onPatch={(change: Change) => act(patch(task.id, change))}
          onStep={(text, step) => act(writeStep(task.id, text, step))}
          onMark={(step, done) => act(markStep(task.id, step, done))}
          onDropStep={(step) => act(dropStep(task.id, step))}
          onLog={(body, entry) => act(writeLog(task.id, body, entry))}
          onDiscard={() => {
            act(discard(task.id));
            setSelected(undefined);
          }}
          onReopen={() => act(reopen(task.id))}
          onClose={shut}
          onError={(e) => setError(saidPlainly(e))}
        />
      ) : (
        <TaskList
          tasks={shown}
          lists={data.lists}
          title={title(chosen, data.lists)}
          count={chosen.named === "tasks" ? undefined : shown.length}
          empty={nothing(chosen, found !== null)}
          note={
            found && found.total > found.tasks.length
              ? fill("someOfMany", `${found.tasks.length}/${found.total}`)
              : undefined
          }
          selected={selected}
          fresh={captured?.id}
          reveal={reveal}
          bands={
            found !== null || chosen.list || chosen.named === "tags" || chosen.tags?.length
              ? undefined
              : chosen.named === "archive"
                ? "month"
                : "day"
          }
          onSelect={setSelected}
          onComplete={
            chosen.named === "archive"
              ? undefined
              : (id) => {
                  const one = shown.find((task) => task.id === id);
                  if (one) say(fill("saidDone", one.title));
                  act(complete(id));
                }
          }
          onFold={
            chosen.named === "archive" ? (id, away) => act(fold(id, away)) : undefined
          }
          above={
            chosen.named === "tasks" ? (
              <div className="flex gap-1 px-2.5 pb-1">
                {SLICES.map((slice) => {
                  const on = (chosen.slice ?? "today") === slice;
                  const many = data.counts[slice === "today" ? "tasks" : slice];
                  return (
                    <button
                      key={slice}
                      type="button"
                      aria-pressed={on}
                      onClick={() => {
                        setSelected(undefined);
                        // Remembered, not reset: a filter that forgets what it
                        // was told is a filter people stop reaching for.
                        window.localStorage.setItem("tisty.slice", slice);
                        setChosen({ named: "tasks", slice });
                      }}
                      className={`rounded-full border px-2.5 py-0.5 text-[11.5px] ${
                        on
                          ? "border-ink bg-ink text-bg"
                          : "border-line text-faint hover:text-soft"
                      }`}
                    >
                      {t(sliceWord(slice))}
                      {many ? <span className="ml-1 tabular-nums opacity-70">{many}</span> : null}
                    </button>
                  );
                })}
              </div>
            ) : chosen.named === "archive" && (data.counts.folded || chosen.folded) ? (
              <button
                onClick={() => {
                  setFound(null);
                  setChosen({ named: "archive", folded: !chosen.folded });
                }}
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
                say(fill("saidFiled", task.title));
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

      {open && mode === "columns" && chosen.named !== "keeping" && (
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
          // Same as the full-screen panel: the task leaves the list, so the
          // column that was showing it has nothing left to show.
          onDiscard={() => {
            act(discard(task.id));
            setSelected(undefined);
          }}
          onReopen={() => act(reopen(task.id))}
          onClose={shut}
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

const sliceWord = (slice: Slice) =>
  slice === "today"
    ? ("today" as const)
    : slice === "upcoming"
      ? ("upcoming" as const)
      : slice === "repeating"
        ? ("repeating" as const)
        : ("sliceAll" as const);
