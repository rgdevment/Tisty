import { listen } from "@tauri-apps/api/event";
import { ask, open as pick } from "@tauri-apps/plugin-dialog";
import { useCallback, useEffect, useRef, useState } from "react";
import { AXES } from "./archive";
import { carrying } from "./carrying";
import { heard, play } from "./chime";
import { asPlain } from "./copying";
import {
  attach,
  type Change,
  capture,
  complete,
  DEEPEST,
  discard,
  docAway,
  docCopy,
  docDrop,
  docExport,
  docFile,
  docImport,
  docNew,
  docPage,
  docs,
  dropStep,
  erase,
  type Filed,
  FOLDER_NAME_AT_MOST,
  type Folded,
  type Found,
  fold,
  folderAdd,
  folderDrop,
  folderFile,
  folderLook,
  folderRename,
  markStep,
  owed,
  type Papers,
  type Pick,
  parted,
  patch,
  type Ready,
  type Rift,
  reopen,
  type Snapshot,
  settleIn,
  snapshot,
  sow,
  syncState,
  type Task,
  type Underway,
  updateReady,
  writeLog,
  writeStep,
} from "./core";
import { decideAll, decidesByBlock } from "./deciding";
import { handTo, whenFilesLand } from "./dropped";
import { todayLong } from "./format";
import { adopt, fill, t } from "./locales";
import { saidPlainly } from "./refusal";
import { settled } from "./saving";
import About from "./ui/About";
import CaptureField from "./ui/CaptureField";
import Closing from "./ui/Closing";
import Cover from "./ui/Cover";
import Detail from "./ui/Detail";
import Docs from "./ui/Docs";
import Folder from "./ui/Folder";
import Keeping from "./ui/Keeping";
import Lists from "./ui/Lists";
import Matrix from "./ui/Matrix";
import Menu, { type Choice } from "./ui/Menu";
import Naming from "./ui/Naming";
import Notice from "./ui/Notice";
import Only from "./ui/Only";
import Owed from "./ui/Owed";
import Rifts from "./ui/Rifts";
import Search from "./ui/Search";
import Shelf from "./ui/Shelf";
import Sidebar from "./ui/Sidebar";
import Sightings from "./ui/Sightings";
import Tags from "./ui/Tags";
import TaskList from "./ui/TaskList";
import Welcome from "./ui/Welcome";
import WindowChrome from "./ui/WindowChrome";
import {
  accepts,
  asView,
  axisWord,
  type Chosen,
  invite,
  LAYERS,
  layerCount,
  layerWord,
  nothing,
  SLICES,
  type Slice,
  title,
} from "./views";

export const steady = <T,>(was: T, found: T): T =>
  JSON.stringify(was) === JSON.stringify(found) ? was : found;

type Mode = "columns" | "sheet";

export const kept = (key: string): string[] => {
  try {
    const said: unknown = JSON.parse(localStorage.getItem(key) ?? "[]");
    return Array.isArray(said) ? said.filter((one) => typeof one === "string") : [];
  } catch {
    return [];
  }
};

export default function App() {
  const [data, setData] = useState<Snapshot | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [selected, setSelected] = useState<string | undefined>();
  const [captured, setCaptured] = useState<Task | undefined>();
  const [aloud, setAloud] = useState("");
  const [ready, setReady] = useState<Ready | null>(null);
  const [underway, setUnderway] = useState<Underway | null>(null);

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
    lists: kept("tisty.only"),
  }));
  const [found, setFound] = useState<Found | null>(null);

  useEffect(() => {
    const wanted = chosen.lists;
    if (!data || !wanted?.length) return;
    const alive = wanted.filter((id) => data.lists.some((one) => one.id === id));
    if (alive.length === wanted.length) return;
    window.localStorage.setItem("tisty.only", JSON.stringify(alive));
    setChosen((was) => ({ ...was, lists: alive }));
  }, [data, chosen.lists]);

  const [papers, setPapers] = useState<Papers>({ folders: [], docs: [] });
  const [makingFolder, setMakingFolder] = useState(false);
  const [renaming, setRenaming] = useState<Folded | null>(null);
  const [note, setNote] = useState<string | null>(null);
  const [menu, setMenu] = useState<{
    at: { x: number; y: number };
    label: string;
    choices: Choice[];
  } | null>(null);
  const [here, setHere] = useState<string | null | undefined>(undefined);
  const standing = here ? papers.folders.find((one) => one.id === here) : undefined;
  const [showing, setShowing] = useState<string | null>(null);
  const [carried, setCarried] = useState(0);
  const [asking, setAsking] = useState<{ id: string; title: string; days: string[] } | null>(null);
  const asked = useRef(0);

  const newDoc = (folder?: string, pageOf?: string) =>
    docNew(folder, pageOf)
      .then((made) => {
        lookPapers();
        setChosen({ named: "docs", doc: made.id });
      })
      .catch((e) => setError(saidPlainly(e)));

  const bringIn = (folder?: string) =>
    pick({
      multiple: false,
      filters: [
        { name: "Markdown", extensions: ["md", "markdown", "txt"] },
        { name: t("anyFile"), extensions: ["*"] },
      ],
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
    ask(
      fill(
        papers.docs.some((one) => one.pageOf === doc.id) ? "dropPagesSure" : "dropDocSure",
        doc.title || t("untitledDoc"),
      ),
      { kind: "warning" },
    )
      .then((yes) => {
        if (!yes) return;
        if (chosen.doc === doc.file) setChosen({ named: "docs" });
        setReturning(doc.folder ?? "unfiled");
        return docDrop(doc.id).then(lookPapers);
      })
      .catch((e) => setError(saidPlainly(e)));

  const deep = (at: string | null | undefined): number => {
    let steps = 0;
    const seen = new Set<string>();
    for (let up = at; up && !seen.has(up); ) {
      seen.add(up);
      steps += 1;
      up = papers.folders.find((one) => one.id === up)?.parent ?? null;
    }
    return steps;
  };

  const trail = (at: string): string => {
    const names: string[] = [];
    const seen = new Set<string>();
    for (let up: string | null | undefined = at; up && !seen.has(up); ) {
      seen.add(up);
      const one = papers.folders.find((each) => each.id === up);
      if (!one) break;
      names.unshift(one.name);
      up = one.parent;
    }
    return names.join(" / ");
  };

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
    const tallest = (at: string): number =>
      1 +
      papers.folders
        .filter((one) => one.parent === at)
        .reduce((most, one) => Math.max(most, tallest(one.id)), 0);
    const tall = moving ? tallest(moving.id) : 0;

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
        .filter((one) => !moving || deep(one.id) + tall <= DEEPEST)
        .map((one) => ({
          key: one.id,
          icon: one.parent ? "↳" : "▸",
          label: trail(one.id),
          onPick: () => land(one.id),
        })),
    ];
  };

  const roomBelow = here != null && deep(here) < DEEPEST;
  const openDoc = (paper: string) => {
    if (papers.docs.some((one) => one.file === paper)) {
      return setChosen({ named: "docs", doc: paper });
    }
    docs()
      .then((found) => {
        setPapers((was) => steady(was, found ?? { folders: [], docs: [] }));
        if (found?.docs.some((one) => one.file === paper)) {
          setChosen({ named: "docs", doc: paper });
        } else {
          setError(t("goneDoc"));
        }
      })
      .catch(() => setError(t("goneDoc")));
  };

  const told = useCallback((problem: unknown) => setError(saidPlainly(problem)), []);

  const lookPapers = useCallback(() => {
    docs()
      .then((found) => setPapers((was) => steady(was, found ?? { folders: [], docs: [] })))
      .catch(() => {});
  }, []);
  useEffect(lookPapers, [lookPapers]);
  const [held, setHeld] = useState<Task | undefined>();
  const acted = useRef<string | null>(null);
  const [greet, setGreet] = useState(false);
  const [greeted, setGreeted] = useState(0);
  const [leaving, setLeaving] = useState(false);
  const [settling, setSettling] = useState(true);
  const [stuck, setStuck] = useState(false);
  const [torn, setTorn] = useState<{
    named: string;
    rifts: Rift[];
    answer: (picks: Pick[] | null) => void;
  } | null>(null);

  useEffect(() => {
    decidesByBlock((named, rifts) => new Promise((answer) => setTorn({ named, rifts, answer })));
    return () => decidesByBlock(null);
  }, []);
  const dismiss = useCallback(() => setCaptured(undefined), []);
  const carries = useRef<ReturnType<typeof carrying>>(null);
  const wasAwry = useRef<string | null>(null);

  useEffect(() => {
    /// A slow answer must not open a strip over the view the person moved on to.
    asked.current += 1;
    setAsking(null);
  }, [chosen]);

  const load = useCallback(() => {
    snapshot(asView(chosen))
      .then((fresh) => {
        adopt(fresh.locale);
        setData(fresh);
        acted.current = null;
      })
      .catch((e) => setError(saidPlainly(e)));
  }, [chosen]);

  useEffect(() => {
    settleIn()
      .then((done) => {
        if (done.stuck) {
          const apart = done.stuck.code === "wouldReset" || done.stuck.code === "otherStore";
          setError(apart ? t("stuckApart") : saidPlainly(done.stuck));
          setStuck(apart);
        }
        return done.brought && latest.current();
      })
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
  const papersAgain = useRef(lookPapers);
  papersAgain.current = lookPapers;
  useEffect(() => {
    const carrier = carrying(
      () => {
        setCarried((was) => was + 1);
        latest.current();
        papersAgain.current();
      },
      (ids) => {
        decideAll(ids).finally(() => latest.current());
      },
      (why) => {
        const now = why?.why ?? null;
        if (now === wasAwry.current) return;
        wasAwry.current = now;
        if (why?.why === "broke") {
          setNote(why.said);
          setTimeout(() => setNote(null), 6000);
        }
      },
    );
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
    const off = listen<Underway>("updating", (said) => setUnderway(said.payload));
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
    // The snapshot carries no documents, so a paper written from outside the window — by an
    // assistant, or by the terminal — would go unseen until the next launch.
    const stirred = listen("stirred", () => {
      latest.current();
      lookPapers();
      setCarried((was) => was + 1);
    });
    const sound = listen<unknown>("chime", (rung) => {
      if (heard(rung.payload)) play(rung.payload);
    });
    return () => {
      stop.then((off) => off()).catch(() => {});
      caught.then((off) => off()).catch(() => {});
      stirred.then((off) => off()).catch(() => {});
      sound.then((off) => off()).catch(() => {});
    };
  }, [lookPapers]);

  const where = useRef(chosen);
  where.current = chosen;

  useEffect(
    () =>
      whenFilesLand((target, paths, at) => {
        setError(null);
        Promise.all(paths.map((one) => attach(one, undefined, where.current.named === "docs")))
          .then((written) => {
            const put = handTo(target, written.join("\n\n"), at);
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
        {error && <p className="mt-16 px-6 text-center text-xs text-urgent">{error}</p>}
      </div>
    );
  }

  const fresh =
    data.tasks.find((candidate) => candidate.id === selected) ??
    found?.tasks.find((candidate) => candidate.id === selected);
  const task = fresh ?? (held?.id === selected ? held : undefined) ?? undefined;
  const open = task !== undefined;
  if (fresh && fresh !== held && acted.current !== fresh.id) setHeld(fresh);

  const remember = (next: Mode) => {
    localStorage.setItem("detail", next);
    setMode(next);
  };

  const wipe = (task: Task) => {
    ask(fill("eraseSure", task.title), { kind: "warning" })
      .then((yes) => {
        if (!yes) return;
        setError(null);
        return erase(task.id).then(() => {
          setSelected(undefined);
          setFound(null);
          say(t("erased"));
          load();
          carries.current?.changed();
        });
      })
      .catch((e) => setError(saidPlainly(e)));
  };

  const marking = (id: string, title: string) => {
    setError(null);
    const mine = ++asked.current;
    owed(id)
      .then((days) => {
        // A slow answer must not open a strip over the task the person moved on to.
        if (mine !== asked.current) return;
        if (!days.length) {
          say(fill("saidDone", title));
          act(complete(id));
          return;
        }
        setAsking({ id, title, days });
      })
      .catch((e) => setError(saidPlainly(e)));
  };

  const strip = asking ? (
    <Owed
      days={asking.days}
      onConfirm={(days) => {
        say(fill("saidDone", asking.title));
        act(complete(asking.id, days));
        setAsking(null);
      }}
    />
  ) : null;

  const hereMenu = (at: { x: number; y: number }) =>
    setMenu({
      at,
      label: t("docsActions"),
      choices: [
        { key: "newDoc", icon: "+", label: t("newDoc"), onPick: () => newDoc(undefined) },
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
          onPick: () => bringIn(undefined),
        },
      ],
    });

  const folderMenu = (folder: Folded, at: { x: number; y: number }) =>
    setMenu({
      at,
      label: t("folderActions"),
      choices: [
        { key: "newDoc", icon: "+", label: t("newDoc"), onPick: () => newDoc(folder.id) },
        {
          key: "newFolder",
          icon: "+",
          label: t("newFolder"),
          off: deep(folder.id) >= DEEPEST,
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
    });

  const docMenu = (doc: Filed, at: { x: number; y: number }) =>
    setMenu({
      at,
      label: t("docActions"),
      choices: [
        {
          key: "newPage",
          icon: "+",
          label: t("newPage"),
          off: doc.archived || !!doc.pageOf,
          onPick: () => newDoc(undefined, doc.id),
        },
        {
          key: "ownDoc",
          icon: "⇤",
          label: t("ownDoc"),
          off: !doc.pageOf,
          onPick: () =>
            docPage(doc.id)
              .then(lookPapers)
              .catch((e) => setError(saidPlainly(e))),
        },
        {
          key: "move",
          icon: "⇢",
          label: t("moveTo"),
          off: doc.archived || !!doc.pageOf,
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
          key: "takeOut",
          icon: "⇪",
          label: t("takeOut"),
          onPick: () =>
            pick({ directory: true })
              .then((at) => (typeof at === "string" ? docExport(doc.file, at) : null))
              .then((taken) => {
                if (taken === null) return;
                setNote(taken ? fill("takenOut", String(taken)) : t("takenOutAlone"));
                setTimeout(() => setNote(null), 3200);
              })
              .catch((e) => setError(saidPlainly(e))),
        },
        {
          key: "seePdf",
          icon: "▤",
          label: t("seePdf"),
          off: showing !== doc.file,
          onPick: () => window.dispatchEvent(new CustomEvent("tisty:see-pdf")),
        },
        {
          key: "asPdf",
          icon: "⇩",
          label: t("toPdf"),
          off: showing !== doc.file,
          onPick: () => window.dispatchEvent(new CustomEvent("tisty:to-pdf")),
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
    });

  const act = (work: Promise<Task>) => {
    setError(null);
    work
      .then((one) => {
        setHeld(one);
        acted.current = one?.id ?? null;
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
    <div className="grid h-full bg-rail font-sans [grid-template-columns:300px_minmax(0,1fr)] min-[1440px]:[grid-template-columns:340px_minmax(0,1fr)]">
      <WindowChrome />

      <p role="status" aria-live="polite" className="sr-only">
        {aloud}
      </p>

      {torn && (
        <Rifts
          named={torn.named}
          rifts={torn.rifts}
          onDone={(picks) => {
            torn.answer(picks);
            setTorn(null);
          }}
          onClose={() => {
            torn.answer(null);
            setTorn(null);
          }}
        />
      )}

      {error && (
        <div
          role="alert"
          className="shadow-lift fixed inset-x-0 top-11 z-[60] mx-auto flex w-fit max-w-[70%] items-start gap-2.5 rounded-[10px] border border-urgent/45 bg-bg px-3.5 py-2 text-[12.5px] leading-snug text-urgent"
        >
          <span className="select-text">{error}</span>
          {stuck && (
            <button
              type="button"
              onClick={() => {
                setStuck(false);
                setError(null);
                setChosen({ named: "keeping" });
              }}
              className="shrink-0 rounded border border-urgent/45 px-1.5 py-0.5 hover:bg-urgent/15"
            >
              {t("stuckTakeMe")}
            </button>
          )}
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
        <Closing onDismiss={() => setLeaving(false)} onError={(e) => setError(saidPlainly(e))} />
      )}

      {makingFolder && (
        <Naming
          title={
            roomBelow
              ? fill("newFolderIn", papers.folders.find((one) => one.id === here)?.name ?? "")
              : t("newFolder")
          }
          invite={t("folderName")}
          most={FOLDER_NAME_AT_MOST}
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
          most={FOLDER_NAME_AT_MOST}
          called={renaming.name}
          drawn={renaming.icon ?? undefined}
          painted={renaming.color ?? undefined}
          action={t("renameIt")}
          onClose={() => setRenaming(null)}
          onName={(name, icon, colour) =>
            Promise.all([
              folderRename(renaming.id, name),
              icon === (renaming.icon ?? undefined) && colour === (renaming.color ?? undefined)
                ? Promise.resolve()
                : folderLook(renaming.id, icon, colour),
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
        <Menu
          at={menu.at}
          choices={menu.choices}
          label={menu.label}
          onClose={() => setMenu(null)}
        />
      )}

      {greet && (
        <Welcome
          onDone={(paper) => {
            setGreet(false);
            setGreeted((n) => n + 1);
            load();
            lookPapers();
            carries.current?.recheck();
            if (paper) openDoc(paper);
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
        onHere={(folder) => {
          setHere(folder ?? null);
          setChosen({ named: "docs" });
        }}
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
        onFolderMenu={folderMenu}
        onDocMenu={docMenu}
        onHereMenu={hereMenu}
        onDocsMenu={(at) =>
          setMenu({
            at,
            label: t("docsActions"),
            choices: [
              {
                key: "newDoc",
                icon: "+",
                label: t("newDoc"),
                onPick: () => newDoc(here ?? undefined),
              },
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

      <div
        className="my-2 mr-2 grid overflow-hidden rounded-[10px] border border-hair bg-bg shadow-lift motion-safe:transition-[grid-template-columns] motion-safe:duration-150"
        style={{
          gridTemplateColumns:
            open && mode === "columns" && chosen.named !== "keeping"
              ? "minmax(0,1fr) 380px"
              : "minmax(0,1fr)",
        }}
      >
        {chosen.named === "aboutScreen" ? (
          <About
            ready={ready}
            step={underway}
            onGaveUp={() => setUnderway(null)}
            onError={(e) => setError(saidPlainly(e))}
          />
        ) : chosen.named === "docs" && !chosen.doc && here !== undefined ? (
          <Folder
            folder={standing ?? null}
            folders={papers.folders}
            docs={papers.docs}
            onOpen={(doc) => setChosen({ named: "docs", doc: doc.file })}
            onHere={(folder) => setHere(folder ?? null)}
            onMenu={folderMenu}
            onHereMenu={hereMenu}
            onDocMenu={docMenu}
          />
        ) : chosen.named === "docs" ? (
          <Docs
            open={chosen.doc}
            known={papers.docs}
            onKept={lookPapers}
            onError={told}
            onShown={setShowing}
            onDoc={openDoc}
            fresh={carried}
          />
        ) : chosen.named === "lists" && !chosen.list ? (
          <Lists
            lists={data.lists}
            counts={data.counts}
            onOpen={(id) => setChosen({ named: "lists", list: id })}
            onChanged={load}
            onError={(e) => setError(saidPlainly(e))}
          />
        ) : chosen.named === "quadrants" && !(open && mode === "sheet") ? (
          <>
            {strip && <div className="shrink-0 px-5 pt-2">{strip}</div>}
            <Matrix
              tasks={data.tasks}
              lists={data.lists}
              beside={open && mode === "columns"}
              onPlace={(id, where) => act(patch(id, { priority: where }))}
              onOpen={(one) => setSelected(one.id)}
              onSow={(where) => {
                sow(where).catch((e: unknown) => setError(saidPlainly(e)));
              }}
              onDiscardAll={(ids) => {
                ask(fill("dropThemSure", String(ids.length)), { kind: "warning" })
                  .then((yes) => {
                    if (!yes) return;
                    setError(null);
                    return Promise.all(ids.map((id) => discard(id))).then(() => {
                      load();
                      carries.current?.changed();
                    });
                  })
                  .catch((e) => setError(saidPlainly(e)));
              }}
            />
          </>
        ) : chosen.named === "keeping" ? (
          <Keeping
            greeted={greeted}
            onGreet={() => setGreet(true)}
            onChanged={() => {
              load();
              lookPapers();
              carries.current?.recheck();
              carries.current?.changed();
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
            onComplete={() => {
              marking(task.id, task.title);
              setSelected(undefined);
            }}
            onDiscard={() => {
              act(discard(task.id));
              setSelected(undefined);
            }}
            onReopen={() => act(reopen(task.id))}
            onErase={() => wipe(task)}
            onClose={shut}
            onError={(e) => setError(saidPlainly(e))}
            onDoc={openDoc}
          />
        ) : (
          <TaskList
            tasks={shown}
            lists={data.lists}
            title={title(chosen, data.lists)}
            when={chosen.named === "tasks" ? todayLong() : undefined}
            count={
              chosen.named === "tasks"
                ? undefined
                : chosen.named === "archive" && !chosen.folded && chosen.layer === "routine"
                  ? data.counts.routines
                  : shown.length
            }
            onBack={
              chosen.list
                ? () => {
                    setChosen({ named: "lists" });
                    setSelected(undefined);
                  }
                : chosen.named === "tags"
                  ? () => {
                      setChosen({ named: "tasks" });
                      setSelected(undefined);
                    }
                  : undefined
            }
            empty={
              found?.papers.length && !shown.length
                ? t("onlyPapers")
                : nothing(chosen, found !== null)
            }
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
            axis={found === null && chosen.named === "archive" ? chosen.axis : undefined}
            dense={
              found === null &&
              chosen.named === "archive" &&
              !chosen.folded &&
              chosen.layer === "trace"
            }
            onSelect={setSelected}
            onComplete={
              chosen.named === "archive"
                ? undefined
                : (id) => {
                    const one = shown.find((task) => task.id === id);
                    marking(id, one?.title ?? "");
                    if (id === selected) setSelected(undefined);
                  }
            }
            onFold={chosen.named === "archive" ? (id, away) => act(fold(id, away)) : undefined}
            closing={asking?.id}
            ask={(id) => (asking?.id === id ? strip : null)}
            below={
              found?.papers.length ? (
                <Sightings papers={found.papers} onOpen={openDoc} />
              ) : undefined
            }
            instead={
              chosen.named === "archive" &&
              !chosen.folded &&
              chosen.layer === "routine" &&
              found === null ? (
                <Shelf
                  lists={data.lists}
                  onOpen={setSelected}
                  onError={(e) => setError(saidPlainly(e))}
                />
              ) : undefined
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
                          setChosen({ named: "tasks", slice, lists: chosen.lists });
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
                  <Only
                    lists={data.lists}
                    chosen={chosen.lists ?? []}
                    onChange={(lists) => {
                      setSelected(undefined);
                      window.localStorage.setItem("tisty.only", JSON.stringify(lists));
                      setChosen({ ...chosen, named: "tasks", lists });
                    }}
                  />
                </div>
              ) : chosen.named === "archive" ? (
                <>
                  {found === null && !chosen.folded && (
                    <Cover onError={(e) => setError(saidPlainly(e))} />
                  )}
                  <div className="flex flex-wrap items-center gap-1 px-2.5 pb-1">
                    {LAYERS.map((layer) => {
                      const on = !chosen.folded && (chosen.layer ?? "story") === layer;
                      const many = data.counts[layerCount(layer)];
                      return (
                        <button
                          key={layer}
                          type="button"
                          aria-pressed={on}
                          onClick={() => {
                            setSelected(undefined);
                            setFound(null);
                            setChosen({ named: "archive", layer });
                          }}
                          className={`rounded-full border px-2.5 py-0.5 text-[11.5px] ${
                            on
                              ? "border-ink bg-ink text-bg"
                              : "border-line text-faint hover:text-soft"
                          }`}
                        >
                          {t(layerWord(layer))}
                          {many ? (
                            <span className="ml-1 tabular-nums opacity-70">{many}</span>
                          ) : null}
                        </button>
                      );
                    })}
                    <span className="mx-1 h-3.5 w-px bg-hair" />
                    {AXES.map((axis) => {
                      const on = (chosen.axis ?? "time") === axis;
                      return (
                        <button
                          key={axis}
                          type="button"
                          aria-pressed={on}
                          onClick={() => {
                            setSelected(undefined);
                            setChosen({ ...chosen, named: "archive", axis, folded: false });
                          }}
                          className={`rounded-full px-2 py-0.5 text-[11.5px] ${
                            on ? "bg-active font-semibold text-ink" : "text-faint hover:text-soft"
                          }`}
                        >
                          {t(axisWord(axis))}
                        </button>
                      );
                    })}
                    {data.counts.folded || chosen.folded ? (
                      <button
                        type="button"
                        aria-pressed={chosen.folded === true}
                        onClick={() => {
                          setSelected(undefined);
                          setFound(null);
                          setChosen({ named: "archive", folded: !chosen.folded });
                        }}
                        className="ml-1 text-xs text-faint hover:text-ink"
                      >
                        {chosen.folded
                          ? `⊕ ${t("backToArchive")}`
                          : `⊖ ${data.counts.folded} ${t("folded")}`}
                      </button>
                    ) : null}
                  </div>
                </>
              ) : chosen.named === "tags" || chosen.tags?.length ? (
                <Tags
                  tags={data.tags}
                  chosen={chosen.tags ?? []}
                  onToggle={(tag) => {
                    const now = chosen.tags ?? [];
                    const next = now.includes(tag) ? now.filter((t) => t !== tag) : [...now, tag];
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
            onComplete={() => {
              marking(task.id, task.title);
              setSelected(undefined);
            }}
            onDiscard={() => {
              act(discard(task.id));
              setSelected(undefined);
            }}
            onReopen={() => act(reopen(task.id))}
            onErase={() => wipe(task)}
            onClose={shut}
            onError={(e) => setError(saidPlainly(e))}
            onDoc={openDoc}
          />
        )}
      </div>
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
