import { ask } from "@tauri-apps/plugin-dialog";
import { useCallback, useEffect, useRef, useState } from "react";
import type { carrying } from "./carrying";
import {
  type Afoot,
  capture,
  complete,
  DEEPEST,
  discard,
  docFile,
  docPage,
  type Filed,
  type Folded,
  type Found,
  fold,
  folderFile,
  owed,
  patch,
  type Snapshot,
  settleIn,
  snapshot,
  sow,
  starDue,
  type Task,
} from "./core";
import { decidesByBlock } from "./deciding";
import { detailOf, erasing } from "./detailing";
import { docChoices, folderChoices, type Hands, hereChoices } from "./docMenus";
import { deep } from "./folders";
import { todayLong } from "./format";
import { useNote } from "./glance";
import { useListening } from "./listening";
import { adopt, fill, t } from "./locales";
import { useOnly, useOnlyAlive } from "./only";
import { saidPlainly } from "./refusal";
import About from "./ui/About";
import { WEEK } from "./ui/Ahead";
import { ArchiveBar, SliceBar } from "./ui/Bars";
import CaptureField from "./ui/CaptureField";
import Detail from "./ui/Detail";
import Docs from "./ui/Docs";
import Door from "./ui/Door";
import Folder from "./ui/Folder";
import Keeping from "./ui/Keeping";
import Layers, { type MenuOpen, type Torn } from "./ui/Layers";
import Lists from "./ui/Lists";
import Matrix from "./ui/Matrix";
import Owed from "./ui/Owed";
import { useParcels } from "./ui/Parcels";
import Pulse from "./ui/Pulse";
import Search from "./ui/Search";
import Shelf from "./ui/Shelf";
import Sidebar from "./ui/Sidebar";
import Sightings from "./ui/Sightings";
import Spine from "./ui/Spine";
import Spread from "./ui/Spread";
import Star from "./ui/Star";
import Tagged from "./ui/Tagged";
import Tags from "./ui/Tags";
import TaskList from "./ui/TaskList";
import WindowChrome from "./ui/WindowChrome";
import { usePapers } from "./usePapers";
import {
  accepts,
  asView,
  type Chosen,
  headerCount,
  invite,
  nothing,
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

  const [makingFolder, setMakingFolder] = useState(false);
  const [renaming, setRenaming] = useState<Folded | null>(null);
  const { note, noted, said } = useNote();
  const [afoot, setAfoot] = useState<Afoot | null>(null);
  const [backing, setBacking] = useState<Filed | null>(null);
  const [menu, setMenu] = useState<MenuOpen | null>(null);
  const [here, setHere] = useState<string | null | undefined>(undefined);
  const carries = useRef<ReturnType<typeof carrying>>(null);
  const paging = useRef<((page: Filed) => boolean) | null>(null);
  const {
    papers,
    lookPapers,
    papersChanged,
    newDoc,
    bringIn,
    dropFolder,
    dropDoc,
    bringBack,
    openDoc,
    hangIt,
  } = usePapers({
    chosen,
    setChosen,
    here,
    setHere,
    setReturning,
    setBacking,
    setError,
    lookForAStar: () => lookForAStar(),
    carries,
    paging,
  });
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

  const roomBelow = here != null && deep(papers.folders, here) < DEEPEST;
  const told = useCallback((problem: unknown) => setError(saidPlainly(problem)), []);

  const [held, setHeld] = useState<Task | undefined>();
  const acted = useRef<string | null>(null);
  const [greet, setGreet] = useState(false);
  const [starring, setStarring] = useState(false);
  const [offering, setOffering] = useState(false);
  const [greeted, setGreeted] = useState(0);
  const [leaving, setLeaving] = useState(false);
  const [settling, setSettling] = useState(true);
  const [stuck, setStuck] = useState(false);
  const [torn, setTorn] = useState<Torn | null>(null);

  useEffect(() => {
    decidesByBlock((named, rifts) => new Promise((answer) => setTorn({ named, rifts, answer })));
    return () => decidesByBlock(null);
  }, []);
  const dismiss = useCallback(() => setCaptured(undefined), []);

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

  useListening({
    latest,
    papersAgain,
    lookPapers,
    docs: papers.docs,
    chosen,
    greeted,
    returning,
    setReturning,
    setCarried,
    setError,
    noted,
    carries,
    setGreet,
    setLeaving,
    setStarring,
    setOffering,
    setAfoot,
  });

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

      <Layers
        afoot={afoot}
        aloud={aloud}
        backing={backing}
        behind={behind}
        captured={captured}
        carries={carries}
        data={data}
        dismiss={dismiss}
        error={error}
        greet={greet}
        here={here}
        leaving={leaving}
        load={load}
        lookAgain={lookAgain}
        lookPapers={lookPapers}
        makingFolder={makingFolder}
        menu={menu}
        note={note}
        openDoc={openDoc}
        opening={opening}
        papers={papers}
        papersChanged={papersChanged}
        parcels={parcels.shown}
        ready={ready}
        renaming={renaming}
        roomBelow={roomBelow}
        setBacking={setBacking}
        setBehind={setBehind}
        setChosen={setChosen}
        setError={setError}
        setGreet={setGreet}
        setGreeted={setGreeted}
        setLeaving={setLeaving}
        setMakingFolder={setMakingFolder}
        setMenu={setMenu}
        setRenaming={setRenaming}
        setReveal={setReveal}
        setStuck={setStuck}
        setTorn={setTorn}
        setUnderway={setUnderway}
        settling={settling}
        stuck={stuck}
        torn={torn}
        underway={underway}
      />

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
                        <SliceBar
                          chosen={chosen}
                          counts={data.counts}
                          lists={byList ? [] : data.lists}
                          setChosen={setChosen}
                          setSelected={setSelected}
                        />
                      ) : chosen.named === "archive" ? (
                        <ArchiveBar
                          chosen={chosen}
                          counts={data.counts}
                          found={found}
                          setChosen={setChosen}
                          setSelected={setSelected}
                          setFound={setFound}
                          fail={(e) => setError(saidPlainly(e))}
                        />
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
