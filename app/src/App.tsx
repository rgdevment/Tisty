import { useCallback, useEffect, useRef, useState } from "react";
import type { carrying } from "./carrying";
import {
  type Afoot,
  complete,
  DEEPEST,
  docFile,
  type Filed,
  type Folded,
  type Found,
  folderFile,
  owed,
  type Snapshot,
  settleIn,
  snapshot,
  starDue,
  type Task,
} from "./core";
import { decidesByBlock } from "./deciding";
import { detailOf, erasing } from "./detailing";
import { docChoices, folderChoices, type Hands, hereChoices } from "./docMenus";
import { deep } from "./folders";
import { useNote } from "./glance";
import { useListening } from "./listening";
import { adopt, fill, t } from "./locales";
import { useOnly, useOnlyAlive } from "./only";
import { saidPlainly } from "./refusal";
import { WEEK } from "./ui/Ahead";
import Detail from "./ui/Detail";
import Door from "./ui/Door";
import Layers, { type MenuOpen, type Torn } from "./ui/Layers";
import Owed from "./ui/Owed";
import { useParcels } from "./ui/Parcels";
import Pulse from "./ui/Pulse";
import Sidebar from "./ui/Sidebar";
import Spine from "./ui/Spine";
import Stage from "./ui/Stage";
import Star from "./ui/Star";
import WindowChrome from "./ui/WindowChrome";
import { usePapers } from "./usePapers";
import { asView, type Chosen, type Slice, useReach } from "./views";
import { useAloud, useTight, useUpdates } from "./watching";
import { knowAgents } from "./who";

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
            <Stage
              act={act}
              asking={asking}
              beside={beside}
              bringBack={bringBack}
              byList={byList}
              cameFrom={cameFrom}
              captured={captured}
              carried={carried}
              carries={carries}
              chosen={chosen}
              data={data}
              detailing={detailing}
              docMenu={docMenu}
              dropDoc={dropDoc}
              folderMenu={folderMenu}
              found={found}
              further={further}
              greeted={greeted}
              here={here}
              hereMenu={hereMenu}
              load={load}
              lookPapers={lookPapers}
              marking={marking}
              openDoc={openDoc}
              paging={paging}
              papers={papers}
              papersChanged={papersChanged}
              parcels={parcels}
              ready={ready}
              remember={remember}
              reveal={reveal}
              say={say}
              seen={seen}
              selected={selected}
              setCameFrom={setCameFrom}
              setCaptured={setCaptured}
              setChosen={setChosen}
              setDealing={setDealing}
              setError={setError}
              setFound={setFound}
              setGreet={setGreet}
              setHere={setHere}
              setSelected={setSelected}
              setShowing={setShowing}
              setUnderway={setUnderway}
              sheet={sheet}
              shown={shown}
              shut={shut}
              standing={standing}
              strip={strip}
              taggedDocs={taggedDocs}
              task={task}
              tight={tight}
              told={told}
              underway={underway}
              wholes={wholes}
            />
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
