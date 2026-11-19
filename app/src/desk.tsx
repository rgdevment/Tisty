import { createContext, useContext } from "react";
import type { Filed, Folded, List, Snapshot, Task } from "./core";
import { detailOf, erasing } from "./detailing";
import { diaryToday } from "./diary";
import { docChoices, folderChoices, type Hands, hereChoices, litFor } from "./docMenus";
import { DIARY, hideChoice } from "./hiding";
import { layoutOf } from "./layout";
import { type ListHands, listChoices } from "./listMenus";
import { t } from "./locales";
import { saidPlainly } from "./refusal";
import type { Chosen } from "./views";
import type { Mode, useWindow } from "./windowing";

type State = ReturnType<typeof useWindow>;

export function deskOf(state: State, data: Snapshot) {
  const {
    tight,
    say,
    error,
    setError,
    selected,
    setSelected,
    setReturning,
    mode,
    setMode,
    chosen,
    setChosen,
    setQuery,
    setCameFrom,
    found,
    setFound,
    setMakingFolder,
    setMakingDiagram,
    setRenaming,
    setEditingList,
    asideShut,
    noted,
    afoot,
    setMenu,
    setHere,
    carries,
    papers,
    board,
    papersChanged,
    newDoc,
    bringIn,
    dropFolder,
    dropDoc,
    bringBack,
    openDoc,
    hangIt,
    showing,
    held,
    setHeld,
    acted,
    greet,
    leaving,
    torn,
    asked,
    parcels,
    asking,
    marking,
    load,
    act,
  } = state;
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

  const { sheet, beside, aside, quiet, papered, lane, room } = layoutOf({
    chosen,
    open,
    mode,
    tight,
    asideShut,
    calm: !asking && !greet && !leaving && !torn && !asked && !afoot && !error && !parcels.asking,
  });

  const remember = (next: Mode) => {
    try {
      localStorage.setItem("detail", next);
    } catch {}
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
    newDiagram: (folder) => setMakingDiagram({ into: folder }),
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
    litFor(folder, papers.folders, (lit) =>
      setMenu({
        at,
        on: folder.id,
        label: t("folderActions"),
        choices: folderChoices(folder, hands, lit),
      }),
    );

  const diaryMenu = (at: { x: number; y: number }) =>
    setMenu({
      at,
      on: DIARY,
      label: t("diary"),
      choices: [{ ...hideChoice(DIARY), apart: false }],
    });

  const docMenu = (doc: Filed, at: { x: number; y: number }) =>
    setMenu({
      at,
      on: doc.id,
      label: t("docActions"),
      choices: docChoices(doc, hands, doc.file === board),
    });

  const listHands: ListHands = {
    edit: setEditingList,
    gone: (list) => {
      if (chosen.list === list) setChosen({ named: "lists" });
      load();
    },
    fail: (e) => setError(saidPlainly(e)),
  };

  const listMenu = (list: List, at: { x: number; y: number }) =>
    setMenu({
      at,
      on: list.id,
      label: t("listActions"),
      choices: listChoices(list, data.counts[list.id] ?? 0, listHands),
    });

  const shown = found?.tasks ?? (chosen.named === "search" ? [] : data.tasks);

  const shut = () => {
    setReturning(selected ?? null);
    setSelected(undefined);
  };

  const wholes = { ...found?.wholes, ...data.wholes };
  const standings = { ...found?.standings, ...data.standings };
  const paces = { ...found?.paces, ...data.paces };
  const detailing = (one: Task) =>
    detailOf(one, {
      data,
      wholes,
      paces,
      act,
      marking,
      wipe,
      shut,
      close: () => setSelected(undefined),
      openDoc,
      opening,
      say,
      fail: (e) => setError(saidPlainly(e)),
      papers: papers.docs,
    });

  const opening = (one: Task) => {
    setHeld(one);
    setSelected(one.id);
  };

  const choose = (next: Chosen) => {
    setChosen(next);
    setQuery("");
    setCameFrom(null);
    setSelected(undefined);
    setFound(null);
    setError(null);
  };

  const toDiary = () =>
    diaryToday()
      .then((today) => {
        papersChanged();
        choose({ named: "docs", doc: today.page });
      })
      .catch((e) => setError(saidPlainly(e)));

  return {
    ...state,
    data,
    task,
    open,
    taggedDocs,
    sheet,
    beside,
    aside,
    quiet,
    papered,
    lane,
    room,
    remember,
    wipe,
    hands,
    listHands,
    listMenu,
    hereMenu,
    folderMenu,
    diaryMenu,
    docMenu,
    shown,
    shut,
    wholes,
    standings,
    paces,
    detailing,
    opening,
    choose,
    toDiary,
  };
}

export const seeing = (one: Task): Chosen =>
  one.list ? { named: "lists", list: one.list } : { named: "tasks", slice: "all" };

export type Desk = ReturnType<typeof deskOf>;

const Shared = createContext<Desk | null>(null);

export const DeskProvider = Shared.Provider;

export const useDesk = (): Desk => {
  const desk = useContext(Shared);
  if (!desk) throw new Error("the desk was read outside the window");
  return desk;
};
