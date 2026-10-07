import type { ReactNode } from "react";
import {
  FOLDER_NAME_AT_MOST,
  folderAdd,
  folderLook,
  folderRename,
  type Pick,
  type Rift,
  updateInstall,
} from "../core";
import { useDesk } from "../desk";
import { fill, t } from "../locales";
import { offerMoved, saidPlainly } from "../refusal";
import { Alarm, Progress } from "./Banners";
import BringingBack from "./BringingBack";
import Closing from "./Closing";
import Menu, { type Choice } from "./Menu";
import Naming from "./Naming";
import Notice from "./Notice";
import Rifts from "./Rifts";
import Welcome from "./Welcome";

export interface Torn {
  named: string;
  rifts: Rift[];
  answer: (picks: Pick[] | null) => void;
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
    afoot,
    aloud,
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
    menu,
    note,
    openDoc,
    opening,
    papers,
    papersChanged,
    ready,
    renaming,
    roomBelow,
    setBacking,
    setBehind,
    setChosen,
    setError,
    setGreet,
    setGreeted,
    setLeaving,
    setMakingFolder,
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

      {torn && (
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
            setChosen({ named: "keeping" });
          }}
          onInstall={() => {
            setUnderway({ stage: "getting", far: 0 });
            updateInstall().catch((problem) => {
              setUnderway(null);
              setError(saidPlainly(problem));
              if (offerMoved(problem)) {
                lookAgain();
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
