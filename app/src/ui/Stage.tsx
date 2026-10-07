import { ask } from "@tauri-apps/plugin-dialog";
import type { MutableRefObject, ReactNode } from "react";
import type { carrying } from "../carrying";
import {
  discard,
  docPage,
  type Filed,
  type Folded,
  type Found,
  type Papers,
  patch,
  type Ready,
  type Snapshot,
  sow,
  type Task,
  type Underway,
} from "../core";
import type { detailOf } from "../detailing";
import { fill } from "../locales";
import { saidPlainly } from "../refusal";
import { type Chosen, title } from "../views";
import About from "./About";
import Board from "./Board";
import Detail from "./Detail";
import Docs from "./Docs";
import Folder from "./Folder";
import Keeping from "./Keeping";
import Lists from "./Lists";
import Matrix from "./Matrix";
import Spread from "./Spread";

interface Props {
  act: (work: Promise<Task>) => void;
  asking: { id: string; title: string; days: string[] } | null;
  beside: boolean;
  bringBack: (doc: Filed) => void;
  byList: boolean;
  cameFrom: Chosen | null;
  captured: Task | undefined;
  carried: number;
  carries: MutableRefObject<ReturnType<typeof carrying> | null>;
  chosen: Chosen;
  data: Snapshot;
  detailing: (one: Task) => ReturnType<typeof detailOf>;
  docMenu: (doc: Filed, at: { x: number; y: number }) => void;
  dropDoc: (doc: Filed) => unknown;
  folderMenu: (folder: Folded, at: { x: number; y: number }) => void;
  found: Found | null;
  further: () => void;
  greeted: number;
  here: string | null | undefined;
  hereMenu: (at: { x: number; y: number }) => void;
  load: () => void;
  lookPapers: () => void;
  marking: (id: string, title: string) => void;
  openDoc: (paper: string) => void;
  paging: MutableRefObject<((page: Filed) => boolean) | null>;
  papers: Papers;
  papersChanged: () => void;
  parcels: { packUp: (which: string[], named: string) => unknown; takeParcel: () => unknown };
  ready: Ready | null;
  remember: (next: "columns" | "sheet") => void;
  reveal: string | undefined;
  say: (words: string) => void;
  seen: Chosen;
  selected: string | undefined;
  setCameFrom: (from: Chosen | null) => void;
  setCaptured: (task: Task | undefined) => void;
  setChosen: (chosen: Chosen) => void;
  setDealing: (dealing: boolean) => void;
  setError: (text: string | null) => void;
  setFound: (found: Found | null) => void;
  setGreet: (greet: boolean) => void;
  setHere: (here: string | null | undefined) => void;
  setSelected: (id: string | undefined) => void;
  setShowing: (file: string | null) => void;
  setUnderway: (underway: Underway | null) => void;
  sheet: boolean;
  shown: Task[];
  shut: () => void;
  standing: Folded | undefined;
  strip: ReactNode;
  taggedDocs: Filed[];
  task: Task | undefined;
  tight: boolean;
  told: (problem: unknown) => void;
  underway: Underway | null;
  wholes: NonNullable<Snapshot["wholes"]>;
}

export default function Stage({
  act,
  asking,
  beside,
  bringBack,
  byList,
  cameFrom,
  captured,
  carried,
  carries,
  chosen,
  data,
  detailing,
  docMenu,
  dropDoc,
  folderMenu,
  found,
  further,
  greeted,
  here,
  hereMenu,
  load,
  lookPapers,
  marking,
  openDoc,
  paging,
  papers,
  papersChanged,
  parcels,
  ready,
  remember,
  reveal,
  say,
  seen,
  selected,
  setCameFrom,
  setCaptured,
  setChosen,
  setDealing,
  setError,
  setFound,
  setGreet,
  setHere,
  setSelected,
  setShowing,
  setUnderway,
  sheet,
  shown,
  shut,
  standing,
  strip,
  taggedDocs,
  task,
  tight,
  told,
  underway,
  wholes,
}: Props) {
  return chosen.named === "aboutScreen" ? (
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
    // One child, one track: a fragment of two would push the board into a column nought pixels wide.
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
  ) : sheet && task ? (
    <Detail
      key={task.id}
      {...detailing(task)}
      expanded
      from={title(chosen, data.lists)}
      onExpand={() => remember("sheet")}
      onCollapse={() => (tight ? shut() : remember("columns"))}
    />
  ) : (
    <Board
      act={act}
      asking={asking}
      byList={byList}
      cameFrom={cameFrom}
      captured={captured}
      chosen={chosen}
      data={data}
      found={found}
      further={further}
      load={load}
      marking={marking}
      openDoc={openDoc}
      reveal={reveal}
      say={say}
      seen={seen}
      selected={selected}
      setCameFrom={setCameFrom}
      setCaptured={setCaptured}
      setChosen={setChosen}
      setError={setError}
      setFound={setFound}
      setSelected={setSelected}
      shown={shown}
      strip={strip}
      taggedDocs={taggedDocs}
      wholes={wholes}
    />
  );
}
