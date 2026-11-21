import { open } from "@tauri-apps/plugin-dialog";
import { useState } from "react";
import { typed } from "../clouds";
import { type About, backUp, type Carrying, restore } from "../core";
import { fill, t } from "../locales";
import { saidPlainly } from "../refusal";
import Backup, { backupMadeSaid, whereToBackUp } from "./Backup";
import type { Run, Word } from "./Card";
import { useCareful } from "./Careful";
import { DANGERS } from "./dangers";
import Leave from "./Leave";
import { Line, mild, risky } from "./Rows";
import { held, type Working } from "./working";
import { Zone } from "./Zone";

interface Props {
  state: Carrying;
  holds: string;
  build: About | null;
  busy: Working;
  run: Run;
  fail: (word?: Word) => void;
  onPack: () => void;
  onUnpack: () => void;
  tell: (word?: Word) => void;
}

export default function Yours({
  state,
  holds,
  build,
  busy,
  run,
  fail,
  onPack,
  onUnpack,
  tell,
}: Props) {
  const [leaving, setLeaving] = useState(false);
  const careful = useCareful();

  const makeBackup = () => {
    if (held(busy, "backup")) return;
    whereToBackUp()
      .then(
        (at) =>
          typeof at === "string" &&
          run(
            "backup",
            () => backUp(at),
            (made) => tell({ card: "backup", text: backupMadeSaid(made) }),
          ),
      )
      .catch((e) => fail({ card: "backup", text: saidPlainly(e) }));
  };

  const takeBackup = () => {
    if (held(busy, "restore")) return;
    open({ filters: [{ name: "Tisty", extensions: ["zip"] }] })
      .then(async (at) => {
        if (typeof at !== "string") return;
        if (!(await careful.ask(DANGERS.restore()))) return;
        run(
          "restore",
          () => restore(at),
          (files) => tell({ card: "restore", text: fill("restored", String(files)) }),
        );
      })
      .catch((e) => fail({ card: "restore", text: saidPlainly(e) }));
  };

  return (
    <>
      {careful.shown}
      <Backup state={state} holds={holds} held={held(busy, "backup")} onMake={makeBackup}>
        <Line row="parcels" why={t("parcelsWhy")} which="parcel">
          <button type="button" disabled={held(busy, "parcel")} onClick={onPack} className={mild}>
            {t("packAllDo")}
          </button>
          <button type="button" disabled={held(busy, "parcel")} onClick={onUnpack} className={mild}>
            {t("unpackDo")}
          </button>
        </Line>
      </Backup>

      <Zone>
        {!state.freesSpace && (
          <Line row="restore" why={t("restoreWhat")} which="restore">
            <button
              type="button"
              disabled={held(busy, "restore")}
              onClick={takeBackup}
              className={risky}
            >
              {t("restoreFrom")}
            </button>
          </Line>
        )}
        <Line row="leave" why={t("leaveWhy")} which="store">
          <button
            type="button"
            disabled={held(busy, "store")}
            onClick={() => setLeaving(true)}
            className={risky}
          >
            {t("leaveOpen")}
          </button>
        </Line>
      </Zone>
      {leaving && (
        <Leave
          cloud={state.mode === "cloud"}
          revokes={!typed(state.cloudProvider)}
          store={build?.store}
          onClose={() => setLeaving(false)}
        />
      )}
    </>
  );
}
