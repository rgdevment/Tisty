import { listen } from "@tauri-apps/api/event";
import { ask, open as pick } from "@tauri-apps/plugin-dialog";
import { useCallback, useEffect, useRef, useState } from "react";
import { carrying } from "./carrying";
import { heard, play } from "./chime";
import {
  type Afoot,
  attach,
  capture,
  complete,
  DEEPEST,
  discard,
  docAway,
  docDrop,
  docFile,
  docImport,
  docNew,
  docPage,
  docs,
  docsCatchUp,
  doorDue,
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
  noteTrouble,
  owed,
  type Papers,
  type Pick,
  parted,
  patch,
  type Rift,
  type Snapshot,
  settleIn,
  snapshot,
  sow,
  starDue,
  syncState,
  type Task,
  updateInstall,
} from "./core";
import { decideAll, decidesByBlock } from "./deciding";
import { detailOf, erasing } from "./detailing";
import { docChoices, folderChoices, type Hands, hereChoices } from "./docMenus";
import { handTo, whenFilesLand } from "./dropped";
import { deep } from "./folders";
import { todayLong } from "./format";
import { useNote } from "./glance";
import { adopt, fill, t } from "./locales";
import { useOnly, useOnlyAlive } from "./only";
import { offerMoved, saidPlainly } from "./refusal";
import { settled } from "./saving";
import About from "./ui/About";
import { WEEK } from "./ui/Ahead";
import Axis from "./ui/Axis";
import { Alarm, Progress } from "./ui/Banners";
import BringingBack from "./ui/BringingBack";
import CaptureField from "./ui/CaptureField";
import Closing from "./ui/Closing";
import Cover from "./ui/Cover";
import Detail from "./ui/Detail";
import Docs from "./ui/Docs";
import Door from "./ui/Door";
import Folder from "./ui/Folder";
import Keeping from "./ui/Keeping";
import Lists from "./ui/Lists";
import Matrix from "./ui/Matrix";
import Menu, { type Choice } from "./ui/Menu";
import Naming from "./ui/Naming";
import Notice from "./ui/Notice";
import Only from "./ui/Only";
import Owed from "./ui/Owed";
import { useParcels } from "./ui/Parcels";
import Pulse from "./ui/Pulse";
import Rifts from "./ui/Rifts";
import Search from "./ui/Search";
import Shelf from "./ui/Shelf";
import Sidebar from "./ui/Sidebar";
import Sightings from "./ui/Sightings";
import Spine from "./ui/Spine";
import Spread from "./ui/Spread";
import Star from "./ui/Star";
import Tagged from "./ui/Tagged";
import Tags from "./ui/Tags";
import Tally from "./ui/Tally";
import TaskList from "./ui/TaskList";
import Welcome from "./ui/Welcome";
import WindowChrome from "./ui/WindowChrome";
import {
  accepts,
  asView,
  type Chosen,
  headerCount,
  invite,
  LAYERS,
  layerCount,
  layerWord,
  nothing,
  SLICES,
  type Slice,
  title,
  useReach,
} from "./views";
import { useAloud, useTight, useUpdates } from "./watching";
import { knowAgents } from "./who";

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

const TURNS_OVER = 60 * 1000;

export default function App() {
  const [data, setData] = useState<Snapshot | null>(null);
  const { ready, lookAgain, underway, setUnderway, behind, setBehind } = useUpdates();
  const tight = useTight();
  const { aloud, say } = useAloud();
  const [error, setError] = useState<string | null>(null);
  const [selected, setSelected] = useState<string | undefined>();
  const [captured, setCaptured] = useState<Task | undefined>();

  const [dealing, setDealing] = useState(false);

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
  /// Where a tag was pressed, so leaving it goes back there rather than to a fixed screen.
  const [cameFrom, setCameFrom] = useState<Chosen | null>(null);
  useOnlyAlive(data?.lists, chosen, setChosen);
  const [seen, byList] = useOnly(chosen);

  const [papers, setPapers] = useState<Papers>({ folders: [], docs: [] });
  const [makingFolder, setMakingFolder] = useState(false);
  const [renaming, setRenaming] = useState<Folded | null>(null);
  const { note, noted, said } = useNote();
  const [afoot, setAfoot] = useState<Afoot | null>(null);
  const [backing, setBacking] = useState<Filed | null>(null);
  const [menu, setMenu] = useState<{
    at: { x: number; y: number };
    label: string;
    choices: Choice[];
    /// Which row it was opened on, so the tree can say so while it stands.
    on?: string;
  } | null>(null);
  const [here, setHere] = useState<string | null | undefined>(undefined);
  const standing = here ? papers.folders.find((one) => one.id === here) : undefined;
  const [showing, setShowing] = useState<string | null>(null);
  const [carried, setCarried] = useState(0);
  const [asking, setAsking] = useState<{ id: string; title: string; days: string[] } | null>(null);
  const asked = useRef(0);

  const lookForAStar = () => {
    starDue()
      .then((due) => setStarring((was) => was || due))
      .catch(() => {});
  };

  const newDoc = (folder?: string, pageOf?: string) =>
    docNew(folder, pageOf)
      .then((made) => {
        papersChanged();
        setChosen({ named: "docs", doc: made.id });
        if (!pageOf) lookForAStar();
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
        papersChanged();
        setChosen({ named: "docs", doc: made.id });
      })
      .catch((e) => setError(saidPlainly(e)));

  const dropFolder = (folder: Folded) =>
    ask(fill("dropFolderSure", folder.name), { kind: "warning" })
      .then((yes) => {
        if (!yes) return;
        if (here === folder.id) setHere(undefined);
        setReturning(folder.parent ?? "unfiled");
        return folderDrop(folder.id).then(papersChanged);
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
        const going = [
          doc.file,
          ...papers.docs.filter((one) => one.pageOf === doc.id).map((one) => one.file),
        ];
        if (chosen.doc && going.includes(chosen.doc)) setChosen({ named: "docs" });
        setReturning(doc.pageOf ?? doc.folder ?? "unfiled");
        return docDrop(doc.id).then(papersChanged);
      })
      .catch((e) => setError(saidPlainly(e)));

  const bringBack = (doc: Filed) => {
    if (doc.pageOf) {
      docAway(doc.id, false)
        .then(papersChanged)
        .catch((e) => setError(saidPlainly(e)));
      return;
    }
    setBacking(doc);
  };

  const roomBelow = here != null && deep(papers.folders, here) < DEEPEST;
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
          void noteTrouble("goneDoc", paper);
          setError(t("goneDoc"));
        }
      })
      .catch((e) => setError(saidPlainly(e)));
  };

  const told = useCallback((problem: unknown) => setError(saidPlainly(problem)), []);

  const caught = useRef(false);
  const lookPapers = useCallback(() => {
    docs()
      .then((found) => {
        const now = found ?? { folders: [], docs: [] };
        setPapers((was) => steady(was, now));
        if (caught.current || now.docs.every((one) => one.told !== false)) return;
        caught.current = true;
        return docsCatchUp()
          .then((all) => {
            setPapers((was) => steady(was, { folders: was.folders, docs: all }));
            if (all.some((one) => one.told === false && !one.gone)) caught.current = false;
          })
          .catch(() => {
            caught.current = false;
          });
      })
      .catch(() => {});
  }, []);
  useEffect(lookPapers, [lookPapers]);
  const [held, setHeld] = useState<Task | undefined>();
  const acted = useRef<string | null>(null);
  const [greet, setGreet] = useState(false);
  const [starring, setStarring] = useState(false);
  const [offering, setOffering] = useState(false);
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
  const paging = useRef<((page: Filed) => boolean) | null>(null);
  const wasAwry = useRef<string | null>(null);

  const papersChanged = useCallback(() => {
    lookPapers();
    carries.current?.changed();
  }, [lookPapers]);

  const parcels = useParcels({ afoot, setAfoot, setError, noted, said, papersChanged });

  useEffect(() => {
    /// A slow answer must not open a strip over the view the person moved on to.
    asked.current += 1;
    setAsking(null);
  }, [chosen]);

  const { reach, further } = useReach(chosen);
  const load = useCallback(() => {
    snapshot(asView(seen, reach))
      .then((fresh) => {
        adopt(fresh.locale);
        knowAgents(fresh.agents, {
          tag: fresh.agent_tag,
          hosts: fresh.hosts,
          machines: fresh.machines,
          here: fresh.machine_here,
          clients: fresh.clients,
        });
        setData(fresh);
        acted.current = null;
      })
      .catch((e) => setError(saidPlainly(e)));
  }, [seen, reach]);

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
      .catch((problem) => setError(saidPlainly(problem)))
      .finally(() => setSettling(false));
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  const latest = useRef(load);
  latest.current = load;
  const papersAgain = useRef(lookPapers);
  papersAgain.current = lookPapers;

  useEffect(() => {
    const again = () => {
      latest.current();
      papersAgain.current();
    };
    const shownAgain = () => {
      if (document.visibilityState === "visible") again();
    };
    window.addEventListener("focus", again);
    document.addEventListener("visibilitychange", shownAgain);
    return () => {
      window.removeEventListener("focus", again);
      document.removeEventListener("visibilitychange", shownAgain);
    };
  }, []);

  useEffect(() => {
    let day = new Date().getDate();
    const turned = setInterval(() => {
      const now = new Date().getDate();
      if (now === day) return;
      day = now;
      latest.current();
    }, TURNS_OVER);
    return () => clearInterval(turned);
  }, []);
  const papersNow = useRef(papers.docs);
  papersNow.current = papers.docs;
  useEffect(() => {
    const carrier = carrying(
      () => {
        setCarried((was) => was + 1);
        latest.current();
        papersAgain.current();
      },
      (ids) => {
        decideAll(ids)
          .then((shut) => {
            if (!shut.length) return;
            const named = shut
              .map((one) => papersNow.current.find((doc) => doc.file === one))
              .map((one) => `«${one?.title?.trim() || t("untitledDoc")}»`)
              .join(", ");
            setError(fill("someLockedAtOdds", named));
          })
          .catch((problem) => setError(saidPlainly(problem)))
          .finally(() => latest.current());
      },
      (why) => {
        const now = why?.why ?? null;
        if (now === wasAwry.current) return;
        wasAwry.current = now;
        if (why?.why === "broke" || why?.why === "amiss") {
          noted(why.said, 6000);
        }
      },
    );
    carries.current = carrier;
    return () => carrier.stop();
  }, [noted]);

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
    document.querySelector<HTMLElement>(`[data-task="${returning}"]`)?.focus();
    setReturning(null);
  }, [returning]);

  useEffect(() => {
    const stop = listen("closing", () => setLeaving(true));
    const gone = listen("withdrawn", () => {
      setStarring(false);
      setOffering(false);
    });
    const caught = listen("captured", () => latest.current());
    // The snapshot carries no documents, so a paper written from outside the window — by an
    // assistant, or by the terminal — would go unseen until the next launch.
    const stirred = listen("stirred", () => {
      latest.current();
      lookPapers();
      setCarried((was) => was + 1);
    });
    const landed = listen<string>("carried", (far) => {
      latest.current();
      if (far.payload === "papers") lookPapers();
      setCarried((was) => was + 1);
    });
    const sound = listen<unknown>("chime", (rung) => {
      if (heard(rung.payload)) play(rung.payload);
    });
    const along = listen<Afoot>("carrying", (step) => {
      setAfoot((was) => (was ? step.payload : was));
    });
    return () => {
      stop.then((off) => off()).catch(() => {});
      gone.then((off) => off()).catch(() => {});
      caught.then((off) => off()).catch(() => {});
      stirred.then((off) => off()).catch(() => {});
      landed.then((off) => off()).catch(() => {});
      sound.then((off) => off()).catch(() => {});
      along.then((off) => off()).catch(() => {});
    };
  }, [lookPapers]);

  const where = useRef(chosen);
  where.current = chosen;

  useEffect(() => {
    setStarring(false);
    setOffering(false);
  }, [chosen]);

  useEffect(() => {
    doorDue()
      .then((due) => setOffering((was) => was || due))
      .catch(() => {});
  }, [greeted]);

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
        {error && <p className="mt-16 px-6 text-center text-[11.5px] text-urgent">{error}</p>}
      </div>
    );
  }

  const fresh =
    data.tasks.find((candidate) => candidate.id === selected) ??
    found?.tasks.find((candidate) => candidate.id === selected) ??
    data.ahead?.find((candidate) => candidate.task.id === selected)?.task ??
    data.routines?.find((candidate) => candidate.task.id === selected)?.task;
  const task = fresh ?? (held?.id === selected ? held : undefined) ?? undefined;
  const open = task !== undefined;
  if (fresh && fresh !== held && acted.current !== fresh.id) setHeld(fresh);

  const wanted = chosen.tags ?? [];
  const taggedDocs = wanted.length
    ? papers.docs.filter(
        (one) => !one.away && wanted.every((tag) => (one.tags ?? []).includes(tag)),
      )
    : [];

  const outside =
    chosen.named === "docs" || chosen.named === "keeping" || chosen.named === "aboutScreen";
  const sheet = open && !outside && (mode === "sheet" || tight);
  const beside = open && !outside && !sheet;
  const aside =
    (chosen.named === "tasks" || chosen.named === "tags" || chosen.list !== undefined) && !sheet;
  const quiet =
    !asking && !greet && !open && !leaving && !torn && !afoot && !error && !parcels.asking;
  const papered =
    chosen.named === "tasks" ||
    chosen.named === "tags" ||
    chosen.named === "archive" ||
    chosen.named === "lists" ||
    chosen.named === "spread" ||
    chosen.list !== undefined ||
    sheet;
  const lane =
    chosen.named === "spread"
      ? ""
      : beside
        ? aside
          ? "@min-[964px]:pr-[404px] @min-[1536px]:pr-[716px]"
          : "@min-[964px]:pr-[404px]"
        : aside
          ? "@min-[884px]:pr-[324px]"
          : "";

  const remember = (next: Mode) => {
    localStorage.setItem("detail", next);
    setMode(next);
  };

  const wipe = (task: Task) =>
    erasing(task, {
      clear: () => setError(null),
      gone: () => {
        setSelected(undefined);
        setFound(null);
        say(t("erased"));
        load();
        carries.current?.changed();
      },
      fail: (e) => setError(saidPlainly(e)),
    });

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
          lookForAStar();
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
        lookForAStar();
      }}
    />
  ) : null;

  const hangIt = async (doc: string, pageOf: string) => {
    const named = (id: string) =>
      papers.docs.find((one) => one.id === id)?.title || t("untitledDoc");
    if (!(await ask(fill("pageOfSure", named(doc), named(pageOf)), { kind: "warning" }))) return;
    const under = papers.docs.find((one) => one.id === pageOf);
    const page = papers.docs.find((one) => one.id === doc);
    docPage(doc, pageOf)
      .then(() => {
        // The line goes in through the editor that holds the book, so its own save carries it and
        // nothing is written behind it. A book that is not open leaves the page in the loose half.
        if (page && under && chosen.doc === under.file) {
          const put = paging.current;
          if (!put) setError(t("leafWaitsInIndex"));
          else if (put(page) === false) setError(t("leafNeedsTitle"));
        }
      })
      .then(papersChanged)
      .catch((e) => setError(saidPlainly(e)));
  };

  const hands: Hands = {
    papers,
    showing,
    newDoc,
    bringIn,
    makeFolder: () => setMakingFolder(true),
    makeFolderIn: (folder) => {
      setHere(folder);
      setMakingFolder(true);
    },
    rename: setRenaming,
    parcels,
    changed: papersChanged,
    fail: (e) => setError(saidPlainly(e)),
    failWith: setError,
    noted: (text) => noted(text),
    open: (doc) => setChosen({ named: "docs", doc }),
    dropFolder,
    dropDoc,
    bringBack,
    hangIt,
  };

  const hereMenu = (at: { x: number; y: number }) =>
    setMenu({ at, label: t("docsActions"), choices: hereChoices(hands) });

  const folderMenu = (folder: Folded, at: { x: number; y: number }) =>
    setMenu({
      at,
      on: folder.id,
      label: t("folderActions"),
      choices: folderChoices(folder, hands),
    });

  const docMenu = (doc: Filed, at: { x: number; y: number }) =>
    setMenu({ at, on: doc.id, label: t("docActions"), choices: docChoices(doc, hands) });

  const act = (work: Promise<Task>) => {
    setError(null);
    work
      .then((one) => {
        setHeld(one);
        acted.current = one?.id ?? null;
        // A hit still shows in the search results it came from, so what came back replaces it
        // there too, or the detail would go on reading the copy from before.
        setFound((was) =>
          was && one
            ? { ...was, tasks: was.tasks.map((hit) => (hit.id === one.id ? one : hit)) }
            : was,
        );
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

  const wholes = { ...found?.wholes, ...data.wholes };
  const detailing = (one: Task) =>
    detailOf(one, {
      data,
      wholes,
      act,
      marking,
      wipe,
      shut,
      close: () => setSelected(undefined),
      openDoc,
      opening,
      say,
      fail: (e) => setError(saidPlainly(e)),
    });

  const opening = (one: Task) => {
    setHeld(one);
    setSelected(one.id);
  };

  return (
    <div className="grid h-full bg-rail font-sans [grid-template-columns:336px_minmax(0,1fr)] min-[1440px]:[grid-template-columns:380px_minmax(0,1fr)]">
      <WindowChrome />

      {backing !== null && (
        <BringingBack
          key={backing.id}
          doc={backing}
          papers={papers}
          onClose={() => setBacking(null)}
          onDone={papersChanged}
          fail={(e) => setError(saidPlainly(e))}
        />
      )}

      {parcels.shown}

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
        <Alarm
          error={error}
          stuck={stuck}
          offer={behind ? ready : null}
          underway={underway}
          onTakeMe={() => {
            setStuck(false);
            setError(null);
            setChosen({ named: "keeping" });
          }}
          onInstall={() => {
            setUnderway({ stage: "getting", far: 0 });
            updateInstall().catch((problem) => {
              setUnderway(null);
              setError(saidPlainly(problem));
              if (offerMoved(problem)) {
                lookAgain();
              }
            });
          }}
          onClose={() => {
            setError(null);
            setBehind(false);
          }}
        />
      )}

      <Progress settling={settling && !error} note={!error && !afoot ? note : null} afoot={afoot} />

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
          onName={(name, icon, colour) =>
            folderAdd(name, roomBelow ? (here ?? undefined) : undefined, icon, colour)
              .then(() => {
                setMakingFolder(false);
                papersChanged();
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
                papersChanged();
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
            opening(captured);
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
        waiting={ready ? ready.version || t("updateWaiting") : undefined}
        here={here}
        acting={menu?.on ?? null}
        onHere={(folder) => {
          setHere(folder ?? null);
          setChosen({ named: "docs" });
        }}
        onMove={(folder, parent, before) =>
          folderFile(folder, parent, before)
            .then(papersChanged)
            .catch((e) => setError(saidPlainly(e)))
        }
        onFile={(doc, folder, before) =>
          docFile(doc, folder, before)
            .then(papersChanged)
            .catch((e) => setError(saidPlainly(e)))
        }
        onPage={(doc, pageOf) => hangIt(doc, pageOf)}
        onFolderMenu={folderMenu}
        onDocMenu={docMenu}
        onHereMenu={hereMenu}
        onDocsMenu={(at) =>
          setMenu({ at, label: t("docsActions"), choices: hereChoices(hands, here ?? undefined) })
        }
        onChoose={(next) => {
          setChosen(next);
          setCameFrom(null);
          setSelected(undefined);
          setFound(null);
          setError(null);
        }}
      />

      <div
        className={`@container my-2 mr-2 flex min-w-0 flex-col overflow-hidden rounded-[10px] border border-hair shadow-lift ${
          papered ? "bg-desk" : "bg-bg"
        }`}
      >
        <div className="relative flex min-h-0 w-full min-w-0 flex-1 overflow-hidden">
          <div
            className={`grid min-h-0 min-w-0 flex-1 grid-rows-[minmax(0,1fr)] overflow-hidden motion-safe:transition-[padding] motion-safe:duration-150 ${lane}`}
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
                onPaging={(put) => {
                  paging.current = put;
                }}
                known={papers.docs}
                folders={papers.folders}
                onFolder={(id) => {
                  setHere(id);
                  setChosen({ named: "docs" });
                }}
                onKept={papersChanged}
                onTag={(tag) => {
                  setSelected(undefined);
                  setCameFrom(chosen);
                  setChosen({ named: "tags", tags: [tag] });
                }}
                onError={told}
                onShown={setShowing}
                onDoc={openDoc}
                onOwned={(id) =>
                  docPage(id)
                    .then(papersChanged)
                    .catch((e) => setError(saidPlainly(e)))
                }
                onDrop={dropDoc}
                onBack={bringBack}
                fresh={carried}
              />
            ) : chosen.named === "lists" && !chosen.list ? (
              <Lists
                lists={data.every ?? data.lists}
                counts={data.counts}
                soonest={data.soonest}
                onOpen={(id) => setChosen({ named: "lists", list: id })}
                onChanged={load}
                onError={(e) => setError(saidPlainly(e))}
              />
            ) : chosen.named === "quadrants" && !sheet ? (
              // One child, one track: a fragment of two would push the board into the next column,
              // which is nought pixels wide whenever nothing is open beside it.
              <div className="flex min-w-0 flex-col overflow-hidden">
                {strip && <div className="shrink-0 px-5 pt-2">{strip}</div>}
                <Matrix
                  tasks={data.tasks}
                  counts={data.counts}
                  lists={data.lists}
                  beside={beside}
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
              </div>
            ) : chosen.named === "spread" && !sheet ? (
              <div className="flex min-w-0 flex-col overflow-hidden">
                <Spread
                  onCarrying={setDealing}
                  tasks={data.tasks}
                  counts={data.counts}
                  onPlace={(id, on) => act(patch(id, on ? { date: on } : { noDate: true }))}
                  onOpen={(one) => setSelected(one.id)}
                />
              </div>
            ) : chosen.named === "keeping" ? (
              <Keeping
                greeted={greeted}
                start={chosen.tab}
                onPack={() => parcels.packUp([], "tisty")}
                onUnpack={parcels.takeParcel}
                onGreet={() => setGreet(true)}
                onDoc={openDoc}
                onChanged={() => {
                  load();
                  lookPapers();
                  carries.current?.recheck();
                  carries.current?.changed();
                }}
              />
            ) : sheet ? (
              <Detail
                key={task.id}
                {...detailing(task)}
                expanded
                from={title(chosen, data.lists)}
                onExpand={() => remember("sheet")}
                onCollapse={() => (tight ? shut() : remember("columns"))}
              />
            ) : (
              <div className="flex min-w-0">
                <div className="flex min-w-0 flex-1 flex-col overflow-hidden">
                  <TaskList
                    tasks={shown}
                    lists={data.every ?? data.lists}
                    wholes={wholes}
                    title={title(chosen, data.lists)}
                    when={chosen.named === "tasks" ? todayLong() : undefined}
                    count={headerCount(chosen, found, data.counts, data.total)}
                    onBack={
                      chosen.list
                        ? () => {
                            setChosen({ named: "lists" });
                            setSelected(undefined);
                          }
                        : chosen.named === "tags"
                          ? () => {
                              setChosen(cameFrom ?? { named: "tasks" });
                              setCameFrom(null);
                              setSelected(undefined);
                            }
                          : undefined
                    }
                    empty={
                      found?.papers.length && !shown.length
                        ? t("onlyPapers")
                        : nothing(seen, found !== null, data.counts.tracesHidden ?? 0)
                    }
                    onReach={!found && data.total > data.tasks.length ? further : undefined}
                    note={
                      found && found.total > found.tasks.length
                        ? fill("someOfMany", `${found.tasks.length}/${found.total}`)
                        : undefined
                    }
                    selected={selected}
                    fresh={captured?.id}
                    reveal={reveal}
                    bands={
                      found !== null ||
                      chosen.list ||
                      chosen.named === "tags" ||
                      chosen.tags?.length
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
                    onFold={
                      chosen.named === "archive" ? (id, away) => act(fold(id, away)) : undefined
                    }
                    closing={asking?.id}
                    ask={(id) => (asking?.id === id ? strip : null)}
                    below={
                      found?.papers.length ? (
                        <Sightings papers={found.papers} onOpen={openDoc} />
                      ) : chosen.tags?.length ? (
                        <Tagged docs={taggedDocs} onOpen={openDoc} />
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
                                {many ? (
                                  <span className="ml-1 tabular-nums opacity-70">{many}</span>
                                ) : null}
                              </button>
                            );
                          })}
                          <Only
                            lists={byList ? [] : data.lists}
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
                          {found === null && !chosen.folded && (
                            <Tally counts={data.counts} onError={(e) => setError(saidPlainly(e))} />
                          )}
                          <fieldset className="flex flex-wrap items-center gap-1 px-2.5 pb-1">
                            <legend className="sr-only">{t("archiveShowing")}</legend>
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
                            {data.counts.folded || chosen.folded ? (
                              <button
                                type="button"
                                aria-pressed={chosen.folded === true}
                                title={chosen.folded ? t("backToArchive") : undefined}
                                onClick={() => {
                                  setSelected(undefined);
                                  setFound(null);
                                  setChosen({ named: "archive", folded: !chosen.folded });
                                }}
                                className={`rounded-full border px-2.5 py-0.5 text-[11.5px] ${
                                  chosen.folded
                                    ? "border-ink bg-ink text-bg"
                                    : "border-line text-faint hover:text-soft"
                                }`}
                              >
                                {t("hiddenOnes")}
                                {data.counts.folded ? (
                                  <span className="ml-1 tabular-nums opacity-70">
                                    {data.counts.folded}
                                  </span>
                                ) : null}
                              </button>
                            ) : null}
                            {!chosen.folded && (chosen.layer ?? "story") !== "routine" && (
                              <>
                                <span className="mx-1.5 h-3.5 w-px bg-hair" />
                                <Axis
                                  axis={chosen.axis ?? "time"}
                                  onChange={(axis) => {
                                    setSelected(undefined);
                                    setChosen({ ...chosen, named: "archive", axis, folded: false });
                                  }}
                                />
                              </>
                            )}
                          </fieldset>
                        </>
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
                      <Search
                        key="archive"
                        fixed="archived"
                        onFound={setFound}
                        onError={setError}
                      />
                    ) : accepts(chosen) ? (
                      <CaptureField
                        invite={invite(chosen, data.lists)}
                        lists={data.lists}
                        tags={data.tags}
                        onCapture={(written, edits) => {
                          setError(null);
                          return capture(written, asView(seen), edits).then((task) => {
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
                </div>
              </div>
            )}
          </div>

          {beside && (
            <Detail
              apart={`${aside ? "right-3 @min-[1536px]:right-[324px]" : "right-3"} ${
                dealing ? "pointer-events-none opacity-10" : ""
              }`}
              key={task.id}
              {...detailing(task)}
              expanded={false}
              onExpand={() => remember("sheet")}
              onCollapse={() => remember("columns")}
            />
          )}
          {aside && (
            <Pulse
              apart={beside ? "hidden @min-[1536px]:flex" : "hidden @min-[884px]:flex"}
              counts={data.counts}
              lists={data.lists}
              ahead={data.ahead ?? []}
              routines={data.routines ?? []}
              papers={papers.docs.filter((one) => !one.pageOf).length}
              onOpen={(id) => setSelected(id)}
              onList={(id) => {
                setSelected(undefined);
                setChosen({ named: "lists", list: id });
              }}
              onQuadrants={() => {
                setSelected(undefined);
                setChosen({ named: "quadrants" });
              }}
            />
          )}

          {offering && quiet && (aside || chosen.named === "docs") && (
            <Door
              apart={
                aside ? "right-3 @min-[884px]:right-[324px]" : "right-3 @min-[1440px]:right-[344px]"
              }
              onSettled={() => setOffering(false)}
              onOpen={() => setChosen({ named: "keeping", tab: "agents" })}
              onError={(problem) => setError(saidPlainly(problem))}
            />
          )}

          {starring && !offering && quiet && (aside || chosen.named === "docs") && (
            <Star
              apart={
                aside ? "right-3 @min-[884px]:right-[324px]" : "right-3 @min-[1440px]:right-[344px]"
              }
              onSettled={() => setStarring(false)}
              onError={(problem) => setError(saidPlainly(problem))}
            />
          )}
        </div>

        {aside && (
          <div className={beside ? "hidden @max-[1536px]:block" : "hidden @max-[884px]:block"}>
            <Spine
              coming={data.ahead ?? []}
              routines={data.routines ?? []}
              days={WEEK}
              onOpen={(id) => setSelected(id)}
            />
          </div>
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
