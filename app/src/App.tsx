import { useCallback, useEffect, useRef, useState } from "react";
import {
  attach,
  capture,
  complete,
  discard,
  docFile,
  docNew,
  docs,
  folderAdd,
  docAway,
  docCopy,
  parted,
  printed,
  docDrop,
  docImport,
  folderFile,
  folderDrop,
  folderLook,
  folderRename,
  type Folded,
  type Filed,
  type Papers,
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
  updateReady,
  type Ready,
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
import Docs from "./ui/Docs";
import Naming from "./ui/Naming";
import Menu, { type Choice } from "./ui/Menu";
import { settled } from "./saving";
import { asPlain } from "./copying";
import { ask, open as pick } from "@tauri-apps/plugin-dialog";
import Lists from "./ui/Lists";
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
  const [aloud, setAloud] = useState("");
  const [ready, setReady] = useState<Ready | null>(null);

  useEffect(() => {
    updateReady()
      .then(setReady)
      .catch(() => {});
  }, []);
  const twice = useRef(0);
  const say = (words: string) => {
    twice.current += 1;
    setAloud(words + "\u200b".repeat(twice.current % 2));
  };
  const [reveal, setReveal] = useState<string | undefined>();
  const [returning, setReturning] = useState<string | null>(null);
  const [mode, setMode] = useState<Mode>(
    () => (localStorage.getItem("detail") as Mode) ?? "columns",
  );
  const [chosen, setChosen] = useState<Chosen>(() => ({
    named: "tasks",
    slice: (localStorage.getItem("tisty.slice") as Slice) ?? "today",
  }));
  const [found, setFound] = useState<Found | null>(null);
  const [papers, setPapers] = useState<Papers>({ folders: [], docs: [] });
  const [makingFolder, setMakingFolder] = useState(false);
  const [renaming, setRenaming] = useState<Folded | null>(null);
  const [note, setNote] = useState<string | null>(null);
  const [menu, setMenu] = useState<{ at: { x: number; y: number }; label: string; choices: Choice[] } | null>(null);
  const [here, setHere] = useState<string | null | undefined>(undefined);
  const [showing, setShowing] = useState<string | null>(null);

  const newDoc = (folder?: string) =>
    docNew(folder)
      .then((made) => {
        lookPapers();
        setChosen({ named: "docs", doc: made.id });
      })
      .catch((e) => setError(saidPlainly(e)));

  const bringIn = (folder?: string) =>
    pick({
      multiple: false,
      filters: [{ name: "Markdown", extensions: ["md", "markdown", "txt"] }],
    })
      .then((at) => (typeof at === "string" ? docImport(at, folder) : null))
      .then((made) => {
        if (!made) return;
        lookPapers();
        setChosen({ named: "docs", doc: made.id });
      })
      .catch((e) => setError(saidPlainly(e)));

  const dropFolder = (folder: Folded) =>
    ask(fill("dropFolderSure", folder.name), { kind: "warning" })
      .then((yes) => {
        if (!yes) return;
        if (here === folder.id) setHere(undefined);
        setReturning(folder.parent ?? "unfiled");
        return folderDrop(folder.id).then(lookPapers);
      })
      .catch((e) => setError(saidPlainly(e)));

  const dropDoc = (doc: Filed) =>
    ask(fill("dropDocSure", doc.title || t("untitledDoc")), { kind: "warning" })
      .then((yes) => {
        if (!yes) return;
        if (chosen.doc === doc.file) setChosen({ named: "docs" });
        setReturning(doc.folder ?? "unfiled");
        return docDrop(doc.id).then(lookPapers);
      })
      .catch((e) => setError(saidPlainly(e)));

  const destinations = (
    skip: string | null,
    land: (folder?: string) => void,
    moving?: Folded,
  ): Choice[] => {
    const under = (at: string): string[] => {
      const kids = papers.folders.filter((one) => one.parent === at);
      return kids.flatMap((one) => [one.id, ...under(one.id)]);
    };
    const forbidden = moving ? new Set([moving.id, ...under(moving.id)]) : new Set<string>();
    const tall = moving ? (under(moving.id).length > 0 ? 2 : 1) : 0;

    return [
      {
        key: "unfiled",
        icon: "↥",
        label: t("unfiled"),
        off: skip === null,
        onPick: () => land(undefined),
      },
      ...papers.folders
        .filter((one) => one.id !== skip && !forbidden.has(one.id))
        .filter((one) => !moving || (one.parent === null ? 1 : 2) + tall <= 2)
        .map((one) => ({
          key: one.id,
          icon: one.parent ? "↳" : "▸",
          label: one.parent
            ? `${papers.folders.find((up) => up.id === one.parent)?.name ?? ""} / ${one.name}`
            : one.name,
          onPick: () => land(one.id),
        })),
    ];
  };

  const roomBelow =
    here != null && !papers.folders.some((one) => one.id === here && one.parent !== null);
  const lookPapers = useCallback(() => {
    docs()
      .then((found) => setPapers(found ?? { folders: [], docs: [] }))
      .catch(() => {});
  }, []);
  useEffect(lookPapers, [lookPapers]);
  const [held, setHeld] = useState<Task | undefined>();
  const [greet, setGreet] = useState(false);
  const [leaving, setLeaving] = useState(false);
  const [settling, setSettling] = useState(true);
  const dismiss = useCallback(() => setCaptured(undefined), []);
  const carries = useRef<ReturnType<typeof carrying>>(null);

  const load = useCallback(() => {
    snapshot(asView(chosen))
      .then((fresh) => {
        adopt(fresh.locale);
        setData(fresh);
      })
      .catch((e) => setError(saidPlainly(e)));
  }, [chosen]);

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
    const off = listen("parting", () => {
      void settled().finally(() => void parted());
    });
    return () => {
      void off.then((stop) => stop());
    };
  }, []);

  useEffect(() => {
    if (!returning) return;
    document.querySelector<HTMLElement>(`[data-row="${returning}"]`)?.focus();
    setReturning(null);
  }, [returning]);

  useEffect(() => {
    const stop = listen("closing", () => setLeaving(true));
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
            const put = handTo(target, written.join("\n\n"));
            if (!put) setError(t("attachmentLost"));
          })
          .catch((e) => setError(saidPlainly(e)));
      }),
    [],
  );

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
            ? "268px minmax(0,1fr) 380px"
            : "268px minmax(0,1fr)",
      }}
    >
      <WindowChrome />

      <p role="status" aria-live="polite" className="sr-only">
        {aloud}
      </p>

      {error && (
        <div
          role="alert"
          className="fixed inset-x-0 top-11 z-[60] mx-auto flex w-fit max-w-[70%] items-start gap-2 rounded-md bg-urgent/12 px-3 py-1.5 text-xs text-urgent"
        >
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
        <p className="pointer-events-none fixed inset-x-0 top-11 z-[60] mx-auto w-fit rounded-md bg-accent-soft px-3 py-1.5 text-xs text-accent">
          {t("settlingIn")}
        </p>
      )}

      {note && !error && (
        <p
          role="status"
          className="pointer-events-none fixed bottom-5 left-1/2 z-[60] w-fit -translate-x-1/2 rounded-lg border border-hair bg-rail px-3.5 py-2 text-xs text-ink shadow-xl"
        >
          {note}
        </p>
      )}

      {leaving && (
        <Closing
          onDismiss={() => setLeaving(false)}
          onError={(e) => setError(saidPlainly(e))}
        />
      )}

      {makingFolder && (
        <Naming
          title={
            roomBelow
              ? fill("newFolderIn", papers.folders.find((one) => one.id === here)?.name ?? "")
              : t("newFolder")
          }
          invite={t("folderName")}
          onClose={() => setMakingFolder(false)}
          onName={(name, icon) =>
            folderAdd(name, roomBelow ? (here ?? undefined) : undefined, icon)
              .then(() => {
                setMakingFolder(false);
                lookPapers();
              })
              .catch((e) => setError(saidPlainly(e)))
          }
        />
      )}

      {renaming && (
        <Naming
          title={t("renameIt")}
          invite={t("folderName")}
          called={renaming.name}
          drawn={renaming.icon ?? undefined}
          action={t("renameIt")}
          onClose={() => setRenaming(null)}
          onName={(name, icon) =>
            Promise.all([
              folderRename(renaming.id, name),
              icon === (renaming.icon ?? undefined)
                ? Promise.resolve()
                : folderLook(renaming.id, icon),
            ])
              .then(() => {
                setRenaming(null);
                lookPapers();
              })
              .catch((e) => setError(saidPlainly(e)))
          }
        />
      )}

      {menu && (
        <Menu at={menu.at} choices={menu.choices} label={menu.label} onClose={() => setMenu(null)} />
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
        papers={papers}
        counts={data.counts}
        chosen={chosen}
        ready={ready !== null}
        here={here}
        onHere={(folder) => setHere(folder ?? null)}
        onMove={(folder, parent) =>
          folderFile(folder, parent)
            .then(lookPapers)
            .catch((e) => setError(saidPlainly(e)))
        }
        onFile={(doc, folder) =>
          docFile(doc, folder)
            .then(lookPapers)
            .catch((e) => setError(saidPlainly(e)))
        }
        onFolderMenu={(folder, at) =>
          setMenu({
            at,
            label: t("folderActions"),
            choices: [
              { key: "newDoc", icon: "+", label: t("newDoc"), onPick: () => newDoc(folder.id) },
              {
                key: "newFolder",
                icon: "+",
                label: t("newFolder"),
                off: folder.parent !== null,
                onPick: () => {
                  setHere(folder.id);
                  setMakingFolder(true);
                },
              },
              {
                key: "rename",
                icon: "✎",
                label: t("rename"),
                apart: true,
                onPick: () => setRenaming(folder),
              },
              {
                key: "move",
                icon: "⇢",
                label: t("moveTo"),
                into: {
                  label: t("moveHere"),
                  choices: destinations(
                    folder.id,
                    (parent) =>
                      folderFile(folder.id, parent)
                        .then(lookPapers)
                        .catch((e) => setError(saidPlainly(e))),
                    folder,
                  ),
                },
              },
              { key: "import", icon: "↧", label: t("importDoc"), onPick: () => bringIn(folder.id) },
              {
                key: "drop",
                icon: "✕",
                label: t("deleteIt"),
                danger: true,
                apart: true,
                onPick: () => dropFolder(folder),
              },
            ],
          })
        }
        onDocMenu={(doc, at) =>
          setMenu({
            at,
            label: t("docActions"),
            choices: [
              {
                key: "move",
                icon: "⇢",
                label: t("moveTo"),
                off: doc.archived,
                into: {
                  label: t("moveHere"),
                  choices: destinations(doc.folder, (folder) =>
                    docFile(doc.id, folder)
                      .then(lookPapers)
                      .catch((e) => setError(saidPlainly(e))),
                  ),
                },
              },
              {
                key: "asPlain",
                icon: "⌘",
                label: t("copyPlain"),
                apart: true,
                onPick: () =>
                  asPlain(doc.file)
                    .then(() => {
                      setNote(t("copied"));
                      setTimeout(() => setNote(null), 3200);
                    })
                    .catch((e) => setError(saidPlainly(e))),
              },
              {
                key: "asPdf",
                icon: "▤",
                label: t("toPdf"),
                off: showing !== doc.file,
                onPick: () => printed().catch((e) => setError(saidPlainly(e))),
              },
              {
                key: "copy",
                icon: "⧉",
                label: t("duplicate"),
                apart: true,
                onPick: () =>
                  docCopy(doc.id)
                    .then((made) => {
                      lookPapers();
                      if (!doc.archived) setChosen({ named: "docs", doc: made.id });
                    })
                    .catch((e) => setError(saidPlainly(e))),
              },
              {
                key: "away",
                icon: doc.archived ? "▢" : "▣",
                label: doc.archived ? t("bringBack") : t("putAway"),
                apart: true,
                onPick: () =>
                  docAway(doc.id, !doc.archived)
                    .then(lookPapers)
                    .catch((e) => setError(saidPlainly(e))),
              },
              {
                key: "drop",
                icon: "✕",
                label: t("deleteIt"),
                danger: true,
                onPick: () => dropDoc(doc),
              },
            ],
          })
        }
        onHereMenu={(at) =>
          setMenu({
            at,
            label: t("docsActions"),
            choices: [
              { key: "newDoc", icon: "+", label: t("newDoc"), onPick: () => newDoc(undefined) },
              { key: "newFolder", icon: "+", label: t("newFolder"), onPick: () => setMakingFolder(true) },
              { key: "import", icon: "↧", label: t("importDoc"), apart: true, onPick: () => bringIn(undefined) },
            ],
          })
        }
        onDocsMenu={(at) =>
          setMenu({
            at,
            label: t("docsActions"),
            choices: [
              { key: "newDoc", icon: "+", label: t("newDoc"), onPick: () => newDoc(here ?? undefined) },
              {
                key: "newFolder",
                icon: "+",
                label: t("newFolder"),
                onPick: () => setMakingFolder(true),
              },
              {
                key: "import",
                icon: "↧",
                label: t("importDoc"),
                apart: true,
                onPick: () => bringIn(here ?? undefined),
              },
            ],
          })
        }
        onChoose={(next) => {
          setChosen(next);
          setSelected(undefined);
          setFound(null);
          setError(null);
        }}
      />

      {chosen.named === "aboutScreen" ? (
        <About ready={ready} onError={(e) => setError(saidPlainly(e))} />
      ) : chosen.named === "docs" ? (
        <Docs
          open={chosen.doc}
          known={papers.docs}
          onKept={lookPapers}
          onError={(e) => setError(saidPlainly(e))}
          onShown={setShowing}
          onDoc={(paper) =>
            papers.docs.some((one) => one.file === paper)
              ? setChosen({ named: "docs", doc: paper })
              : setError(t("goneDoc"))
          }
        />
      ) : chosen.named === "lists" && !chosen.list ? (
        <Lists
          lists={data.lists}
          counts={data.counts}
          onOpen={(id) => setChosen({ named: "lists", list: id })}
          onChanged={load}
          onError={(e) => setError(saidPlainly(e))}
        />
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
          onDoc={(paper) =>
            papers.docs.some((one) => one.file === paper)
              ? setChosen({ named: "docs", doc: paper })
              : setError(t("goneDoc"))
          }
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
          onDiscard={() => {
            act(discard(task.id));
            setSelected(undefined);
          }}
          onReopen={() => act(reopen(task.id))}
          onClose={shut}
          onError={(e) => setError(saidPlainly(e))}
          onDoc={(paper) =>
            papers.docs.some((one) => one.file === paper)
              ? setChosen({ named: "docs", doc: paper })
              : setError(t("goneDoc"))
          }
        />
      )}
    </div>
  );
}

const sliceWord = (slice: Slice) =>
  slice === "today"
    ? ("today" as const)
    : slice === "upcoming"
      ? ("upcoming" as const)
      : slice === "repeating"
        ? ("repeating" as const)
        : ("sliceAll" as const);
