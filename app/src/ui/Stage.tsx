import { useEffect, useMemo, useRef } from "react";
import { agentsTree, folderShown, isAgentsFolder } from "../agentsSync";
import { taskStepMarked } from "../citing";
import { discard, patch, sow } from "../core";
import { seeing, useDesk } from "../desk";
import { fill } from "../locales";
import { saidPlainly } from "../refusal";
import { afterMove } from "../relocation";
import { sure } from "../sure";
import { shown } from "../tongues";
import { title } from "../views";
import About from "./About";
import Board from "./Board";
import BrainstyView, { openedHere } from "./BrainstyView";
import { AGENTS_HUE } from "./brainsty/looks";
import Detail from "./Detail";
import Docs from "./Docs";
import Folder from "./Folder";
import Help from "./Help";
import HiddenFolders from "./HiddenFolders";
import Keeping from "./Keeping";
import Lists from "./Lists";
import Matrix from "./Matrix";
import Review from "./Review";
import Spread from "./Spread";

export default function Stage() {
  const desk = useDesk();
  const {
    act,
    beside,
    bringBack,
    cameFrom,
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
    markingAway,
    noted,
    openDoc,
    opening,
    paging,
    papers,
    papersChanged,
    parcels,
    ready,
    remember,
    setAsideShut,
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
  } = desk;
  const tagNames = useMemo(() => data.tags.map((one) => one.tag), [data.tags]);
  const agentsFolders = useMemo(() => agentsTree(papers.folders), [papers.folders]);
  const groups = useMemo(
    () => [
      ...papers.folders.map((one) => ({
        ...one,
        name: folderShown(one),
        color: isAgentsFolder(one) ? AGENTS_HUE : one.color,
        kind: "folder" as const,
        agents: agentsFolders.has(one.id),
      })),
      ...(data.every ?? data.lists).map((one) => ({ ...one, kind: "list" as const })),
    ],
    [papers.folders, data.every, data.lists, agentsFolders],
  );
  const refreshed = () => {
    load();
    lookPapers();
    carries.current?.recheck();
    carries.current?.changed();
  };
  const finishCited = (id: string, title: string) =>
    markingAway(id, title, (one) => {
      const at = seeing(one);
      setChosen(at);
      opening(one);
      return at;
    });
  const fresh = useRef(refreshed);
  fresh.current = refreshed;
  useEffect(() => afterMove(() => fresh.current()), []);
  return chosen.named === "help" ? (
    <Help topic={chosen.topic} from={chosen.from} />
  ) : chosen.named === "brainsty" ? (
    <BrainstyView
      near={chosen.near}
      groups={groups}
      onOpen={(one) => {
        setCameFrom(null);
        openedHere(one, desk);
      }}
      onLeave={() => setChosen(cameFrom ?? { named: "tasks" })}
    />
  ) : chosen.named === "docs" && !chosen.doc && here !== undefined ? (
    <Folder
      folder={standing ?? null}
      folders={papers.folders}
      docs={shown(papers.docs)}
      onOpen={(doc) => setChosen({ named: "docs", doc: doc.file })}
      onHere={(folder) => setHere(folder ?? null)}
      onMenu={folderMenu}
      onHereMenu={hereMenu}
      onDocMenu={docMenu}
    />
  ) : chosen.named === "docs" ? (
    <Docs
      open={chosen.doc}
      visit={chosen}
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
      onNote={noted}
      tags={tagNames}
      onShown={setShowing}
      onDoc={openDoc}
      onDrop={dropDoc}
      onDropAll={(all) => void dropDocs(all)}
      onTask={(one) => {
        if (one.list) setChosen({ named: "lists", list: one.list });
        opening(one);
      }}
      onSeeTask={(one) => {
        setChosen(seeing(one));
        opening(one);
      }}
      onTaskDone={finishCited}
      onTaskStep={(id, step, done) => act(taskStepMarked(id, step, done))}
      heard={data.tasks}
      onBack={bringBack}
      onAsideShut={setAsideShut}
      fresh={carried}
    />
  ) : chosen.named === "lists" && !chosen.list && !sheet ? (
    <Lists
      lists={data.every ?? data.lists}
      counts={data.counts}
      soonest={data.soonest}
      standings={data.standings}
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
  ) : chosen.named === "weekly" && !sheet ? (
    <Review />
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
      hidden={<HiddenFolders papers={papers} />}
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
