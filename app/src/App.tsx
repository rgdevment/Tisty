import { useCallback, useEffect, useRef, useState } from "react";
import type { carrying } from "./carrying";
import {
  type Afoot,
  DEEPEST,
  docFile,
  type Filed,
  type Folded,
  type Found,
  folderFile,
  starDue,
  type Task,
} from "./core";
import { decidesByBlock } from "./deciding";
import { detailOf, erasing } from "./detailing";
import { docChoices, folderChoices, type Hands, hereChoices } from "./docMenus";
import { deep } from "./folders";
import { useNote } from "./glance";
import { layoutOf } from "./layout";
import { useListening } from "./listening";
import { t } from "./locales";
import { useMarking } from "./marking";
import { useOnly, useOnlyAlive } from "./only";
import { saidPlainly } from "./refusal";
import { useSnapshot } from "./snapshotting";
import { WEEK } from "./ui/Ahead";
import Layers, { type MenuOpen, type Torn } from "./ui/Layers";
import Margins from "./ui/Margins";
import { useParcels } from "./ui/Parcels";
import Sidebar from "./ui/Sidebar";
import Spine from "./ui/Spine";
import Stage from "./ui/Stage";
import WindowChrome from "./ui/WindowChrome";
import { usePapers } from "./usePapers";
import { type Chosen, type Slice, useReach } from "./views";
import { useAloud, useTight, useUpdates } from "./watching";

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
  const [torn, setTorn] = useState<Torn | null>(null);

  useEffect(() => {
    decidesByBlock((named, rifts) => new Promise((answer) => setTorn({ named, rifts, answer })));
    return () => decidesByBlock(null);
  }, []);
  const dismiss = useCallback(() => setCaptured(undefined), []);

  const parcels = useParcels({ afoot, setAfoot, setError, noted, said, papersChanged });

  const { asking, marking, strip } = useMarking({
    chosen,
    act: (work) => act(work),
    say,
    setError,
    lookForAStar: () => lookForAStar(),
  });
  const { reach, further } = useReach(chosen);
  const { data, load, latest, settling, stuck, setStuck } = useSnapshot({
    seen,
    reach,
    acted,
    setError,
  });
  useOnlyAlive(data?.lists, chosen, setChosen);

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

  const { sheet, beside, aside, quiet, papered, lane } = layoutOf({
    chosen,
    open,
    mode,
    tight,
    calm: !asking && !greet && !leaving && !torn && !afoot && !error && !parcels.asking,
  });

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

          <Margins
            aside={aside}
            beside={beside}
            dealing={dealing}
            task={task}
            detailing={detailing}
            remember={remember}
            data={data}
            papers={papers}
            chosen={chosen}
            setChosen={setChosen}
            setSelected={setSelected}
            offering={offering}
            setOffering={setOffering}
            starring={starring}
            setStarring={setStarring}
            quiet={quiet}
            setError={setError}
          />
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
