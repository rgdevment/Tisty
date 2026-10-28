import type { ReactNode } from "react";
import {
  FOLDER_NAME_AT_MOST,
  folderAdd,
  folderLook,
  folderRename,
  type Pick,
  type Rift,
  reopen,
  updateInstall,
} from "../core";
import { useDesk } from "../desk";
import { fill, t } from "../locales";
import { offerMoved, saidPlainly } from "../refusal";
import type { ShapeChoice } from "../reshaping";
import { Alarm, Progress } from "./Banners";
import BringingBack from "./BringingBack";
import Cheer from "./Cheer";
import Closing from "./Closing";
import DiagramMaking from "./DiagramMaking";
import Menu, { type Choice } from "./Menu";
import Naming from "./Naming";
import Notice from "./Notice";
import Raised from "./Raised";
import Reshape from "./Reshape";
import Rifts from "./Rifts";
import Sure from "./Sure";
import Versions from "./Versions";
import Welcome from "./Welcome";

export interface Torn {
  named: string;
  rifts: Rift[];
  answer: (picks: Pick[] | null) => void;
}

export interface Asked {
  said: string;
  turn: number;
  hurried: boolean;
  done: (yes: boolean) => void;
}

export interface Shaping {
  said: string;
  done: (chosen: ShapeChoice) => void;
}

export interface MenuOpen {
  at: { x: number; y: number };
  label: string;
  choices: Choice[];
  /// Which row it was opened on, so the tree can say so while it stands.
  on?: string;
}

export default function Layers({ parcels }: { parcels: ReactNode }) {
  const {
    act,
    cheered,
    setCheered,
    afoot,
    aloud,
    asked,
    shaping,
    setShaping,
    backing,
    behind,
    captured,
    carries,
    data,
    dismiss,
    error,
    greet,
    here,
    leaving,
    load,
    lookAgain,
    lookPapers,
    makingFolder,
    makingDiagram,
    menu,
    newDiagram,
    note,
    openDoc,
    opening,
    papers,
    papersChanged,
    ready,
    renaming,
    roomBelow,
    setBacking,
    setAsked,
    setBehind,
    setChosen,
    setError,
    setGreet,
    setGreeted,
    setLeaving,
    setMakingFolder,
    setMakingDiagram,
    setMenu,
    setRenaming,
    setReveal,
    setStuck,
    setTorn,
    setUnderway,
    settling,
    stuck,
    torn,
    underway,
  } = useDesk();
  return (
    <>
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

      {parcels}

      <p role="status" aria-live="polite" className="sr-only">
        {aloud}
      </p>

      {asked && (
        <Sure
          key={asked.turn}
          said={asked.said}
          hurried={asked.hurried}
          onAnswer={(yes) => {
            asked.done(yes);
            setAsked(null);
          }}
        />
      )}

      {shaping && (
        <Reshape
          said={shaping.said}
          onAnswer={(chosen) => {
            shaping.done(chosen);
            setShaping(null);
          }}
        />
      )}

      {torn && torn.rifts.length === 0 && (
        <Versions
          named={torn.named}
          onPick={(pick) => {
            torn.answer(pick ? [pick] : null);
            setTorn(null);
          }}
        />
      )}

      {torn && torn.rifts.length > 0 && (
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
            setChosen({ named: "keeping", tab: "sync", movedAt: Date.now() });
          }}
          onInstall={() => {
            setUnderway({ stage: "getting", far: 0 });
            updateInstall().catch((problem) => {
              setUnderway(null);
              setError(saidPlainly(problem));
              if (offerMoved(problem)) {
                void lookAgain();
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
      <Raised />

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

      {makingDiagram && (
        <DiagramMaking
          onClose={() => setMakingDiagram(null)}
          onMake={(name, kind) => {
            setMakingDiagram(null);
            void newDiagram(name, kind, makingDiagram.into);
          }}
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

      {cheered && (
        <Cheer
          key={`cheer:${cheered.id}`}
          task={cheered}
          onUndo={() => {
            act(reopen(cheered.id));
            setCheered(undefined);
          }}
          onDismiss={() => setCheered(undefined)}
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
    </>
  );
}
