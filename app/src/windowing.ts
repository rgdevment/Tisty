import { useCallback, useEffect, useRef, useState } from "react";
import type { carrying } from "./carrying";
import {
  type Afoot,
  DEEPEST,
  type DoorDue,
  type Filed,
  type Folded,
  type Found,
  starDue,
  type Task,
} from "./core";
import { decidesByBlock, decidesWhole } from "./deciding";
import { deep } from "./folders";
import { useNote } from "./glance";
import { useListening } from "./listening";
import { useMarking } from "./marking";
import { useOnly, useOnlyAlive } from "./only";
import { saidPlainly } from "./refusal";
import { asksShape, type Shape } from "./reshaping";
import { useSnapshot } from "./snapshotting";
import { asksSure } from "./sure";
import { useBeaver } from "./ui/Beaver";
import type { Asked, MenuOpen, Shaping, Torn } from "./ui/Layers";
import { useParcels } from "./ui/Parcels";
import { SETTLING_MS } from "./ui/Sure";
import { usePapers } from "./usePapers";
import { type Chosen, type Slice, useReach } from "./views";
import { useAloud, useTight, useUpdates } from "./watching";

export type Mode = "columns" | "sheet";

export const kept = (key: string): string[] => {
  try {
    const said: unknown = JSON.parse(localStorage.getItem(key) ?? "[]");
    return Array.isArray(said) ? said.filter((one) => typeof one === "string") : [];
  } catch {
    return [];
  }
};

export function useWindow() {
  const { ready, lookAgain, underway, setUnderway, behind, setBehind } = useUpdates();
  const tight = useTight();
  const { aloud, say } = useAloud();
  const [error, setError] = useState<string | null>(null);
  const [selected, setSelected] = useState<string | undefined>();
  const [captured, setCaptured] = useState<Task | undefined>();
  const [cheered, setCheered] = useState<Task | undefined>();
  const peeks = useBeaver();

  const [dealing, setDealing] = useState(false);

  const [reveal, setReveal] = useState<string | undefined>();
  const [returning, setReturning] = useState<string | null>(null);
  const [mode, setMode] = useState<Mode>(() => {
    try {
      return localStorage.getItem("detail") === "sheet" ? "sheet" : "columns";
    } catch {
      return "columns";
    }
  });
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
    dropDocs,
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
    openTask: setSelected,
    noted,
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
  const overlays = useOverlays();
  const { setGreet, setStarring, setOffering, greeted, setLeaving } = overlays;
  const dismiss = useCallback(() => setCaptured(undefined), []);

  const parcels = useParcels({ afoot, setAfoot, setError, noted, said, papersChanged });

  const { asking, marking, strip } = useMarking({
    chosen,
    act: (work) => act(work),
    say,
    setError,
    lookForAStar: () => lookForAStar(),
    cheer: peeks ? setCheered : undefined,
  });
  const { reach, further } = useReach(chosen);
  const { data, load, latest, settling, stuck, setStuck } = useSnapshot({
    seen,
    reach,
    acted,
    setError,
  });
  useOnlyAlive(data?.lists, chosen, setChosen);

  useListening({
    latest,
    lookPapers,
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

  return {
    ...overlays,
    ready,
    lookAgain,
    underway,
    setUnderway,
    behind,
    setBehind,
    tight,
    aloud,
    say,
    error,
    setError,
    selected,
    setSelected,
    captured,
    setCaptured,
    cheered,
    setCheered,
    dealing,
    setDealing,
    reveal,
    setReveal,
    returning,
    setReturning,
    mode,
    setMode,
    chosen,
    setChosen,
    found,
    setFound,
    cameFrom,
    setCameFrom,
    seen,
    byList,
    makingFolder,
    setMakingFolder,
    renaming,
    setRenaming,
    note,
    noted,
    said,
    afoot,
    setAfoot,
    backing,
    setBacking,
    menu,
    setMenu,
    here,
    setHere,
    carries,
    paging,
    papers,
    lookPapers,
    papersChanged,
    newDoc,
    bringIn,
    dropFolder,
    dropDoc,
    dropDocs,
    bringBack,
    openDoc,
    hangIt,
    standing,
    showing,
    setShowing,
    carried,
    setCarried,
    lookForAStar,
    roomBelow,
    told,
    held,
    setHeld,
    acted,
    dismiss,
    parcels,
    asking,
    marking,
    strip,
    reach,
    further,
    data,
    load,
    latest,
    settling,
    stuck,
    setStuck,
    act,
  };
}

function useOverlays() {
  const [greet, setGreet] = useState(false);
  const [starring, setStarring] = useState(false);
  const [offering, setOffering] = useState<DoorDue | null>(null);
  const [greeted, setGreeted] = useState(0);
  const [leaving, setLeaving] = useState(false);
  const [torn, setTorn] = useState<Torn | null>(null);
  const [asked, setAsked] = useState<Asked | null>(null);
  const [shaping, setShaping] = useState<Shaping | null>(null);

  useEffect(() => {
    decidesByBlock((named, rifts) => new Promise((answer) => setTorn({ named, rifts, answer })));
    decidesWhole(
      (named) =>
        new Promise((answer) =>
          setTorn({ named, rifts: [], answer: (picks) => answer(picks?.[0] ?? null) }),
        ),
    );
    let turn: Promise<unknown> = Promise.resolve();
    let turns = 0;
    let answeredAt = Number.NEGATIVE_INFINITY;
    asksSure((said) => {
      const answer = turn.then(
        () =>
          new Promise<boolean>((done) => {
            turns += 1;
            const hurried = performance.now() - answeredAt < SETTLING_MS;
            setAsked({
              said,
              turn: turns,
              hurried,
              done: (yes) => {
                answeredAt = performance.now();
                done(yes);
              },
            });
          }),
      );
      turn = answer.catch(() => undefined);
      return answer;
    });
    // One question at a time, as with being sure: a second drop waits instead of taking the first's place.
    let shapes: Promise<unknown> = Promise.resolve();
    asksShape((said) => {
      const answer = shapes.then(
        () => new Promise<Shape | undefined>((done) => setShaping({ said, done })),
      );
      shapes = answer.catch(() => undefined);
      return answer;
    });
    return () => {
      decidesByBlock(null);
      decidesWhole(null);
      asksSure(null);
      asksShape(null);
    };
  }, []);

  return {
    greet,
    setGreet,
    starring,
    setStarring,
    offering,
    setOffering,
    greeted,
    setGreeted,
    leaving,
    setLeaving,
    torn,
    setTorn,
    asked,
    setAsked,
    shaping,
    setShaping,
  };
}
