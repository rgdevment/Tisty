import { useEffect, useMemo, useRef } from "react";
import { discard, patch, sow } from "../core";
import { useDesk } from "../desk";
import { fill } from "../locales";
import { saidPlainly } from "../refusal";
import { afterMove } from "../relocation";
import { sure } from "../sure";
import { title } from "../views";
import About from "./About";
import Board from "./Board";
import Detail from "./Detail";
import Docs from "./Docs";
import Folder from "./Folder";
import Help from "./Help";
import Keeping from "./Keeping";
import Lists from "./Lists";
import Matrix from "./Matrix";
import Spread from "./Spread";

export default function Stage() {
  const {
    act,
    beside,
    bringBack,
    carried,
    carries,
    chosen,
    data,
    detailing,
    docMenu,
    dropDoc,
    dropDocs,
    folderMenu,
    greeted,
    here,
    hereMenu,
    load,
    lookPapers,
    openDoc,
    opening,
    paging,
    papers,
    papersChanged,
    parcels,
    ready,
    remember,
    setCameFrom,
    setChosen,
    setDealing,
    setError,
    setGreet,
    setHere,
    setSelected,
    setShowing,
    setUnderway,
    sheet,
    shut,
    standing,
    strip,
    task,
    tight,
    told,
    underway,
  } = useDesk();
  const tagNames = useMemo(() => data.tags.map((one) => one.tag), [data.tags]);
  const refreshed = () => {
    load();
    lookPapers();
    carries.current?.recheck();
    carries.current?.changed();
  };
  const fresh = useRef(refreshed);
  fresh.current = refreshed;
  useEffect(() => afterMove(() => fresh.current()), []);
  return chosen.named === "help" ? (
    <Help topic={chosen.topic} from={chosen.from} />
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
      tags={tagNames}
      onShown={setShowing}
      onDoc={openDoc}
      onDrop={dropDoc}
      onDropAll={(all) => void dropDocs(all)}
      onTask={(one) => {
        if (one.list) setChosen({ named: "lists", list: one.list });
        opening(one);
      }}
      onBack={bringBack}
      fresh={carried}
    />
  ) : chosen.named === "lists" && !chosen.list && !sheet ? (
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
          sure(fill("dropThemSure", String(ids.length)))
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
      tab={chosen.tab ?? "general"}
      onTab={(tab) => setChosen({ named: "keeping", tab })}
      about={
        <About
          ready={ready}
          step={underway}
          onGaveUp={() => setUnderway(null)}
          onError={(e) => setError(saidPlainly(e))}
        />
      }
      move={chosen.move}
      movedAt={chosen.movedAt}
      onPack={() => parcels.packUp([], "tisty")}
      onUnpack={parcels.takeParcel}
      onGreet={() => setGreet(true)}
      onDoc={openDoc}
      onChanged={refreshed}
    />
  ) : sheet && task ? (
    <Detail
      key={task.id}
      {...detailing(task)}
      from={title(chosen, data.lists)}
      onCollapse={() => (tight ? shut() : remember("columns"))}
    />
  ) : (
    <Board />
  );
}
