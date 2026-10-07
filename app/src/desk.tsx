import { createContext, useContext } from "react";
import type { Filed, Folded, Snapshot, Task } from "./core";
import { detailOf, erasing } from "./detailing";
import { docChoices, folderChoices, type Hands, hereChoices } from "./docMenus";
import { layoutOf } from "./layout";
import { t } from "./locales";
import { saidPlainly } from "./refusal";
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
    found,
    setFound,
    setMakingFolder,
    setRenaming,
    noted,
    afoot,
    setMenu,
    setHere,
    carries,
    papers,
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
    remember,
    wipe,
    hands,
    hereMenu,
    folderMenu,
    docMenu,
    shown,
    shut,
    wholes,
    detailing,
    opening,
  };
}

export type Desk = ReturnType<typeof deskOf>;

const Shared = createContext<Desk | null>(null);

export const DeskProvider = Shared.Provider;

export const useDesk = (): Desk => {
  const desk = useContext(Shared);
  if (!desk) throw new Error("the desk was read outside the window");
  return desk;
};
