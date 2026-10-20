import { open, save } from "@tauri-apps/plugin-dialog";
import { useState } from "react";
import { type About, backUp, type Carrying, restore, revealed } from "../core";
import { weigh } from "../format";
import { fill, t } from "../locales";
import { saidPlainly } from "../refusal";
import { sure } from "../sure";
import Backup from "./Backup";
import type { Run, Which, Word } from "./Card";
import Leave from "./Leave";
import { Band, Line, mild, risky } from "./Rows";

interface Props {
  state: Carrying;
  holds: string;
  build: About | null;
  busy: Which | null;
  said?: Word;
  trouble?: Word;
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
  said,
  trouble,
  run,
  fail,
  onPack,
  onUnpack,
  tell,
}: Props) {
  const held = busy !== null;
  const [leaving, setLeaving] = useState(false);

  const makeBackup = () => {
    if (held) return;
    save({ defaultPath: "tisty-backup.zip", filters: [{ name: "Tisty", extensions: ["zip"] }] })
      .then(
        (at) =>
          typeof at === "string" &&
          run("backup", backUp(at), (made) =>
            tell({
              card: "backup",
              text: made.leftOut
                ? fill(
                    "backupLeftOut",
                    weigh(made.bytes),
                    `${made.leftOut} · ${weigh(made.leftOutBytes)}`,
                  )
                : fill("backupMade", weigh(made.bytes)),
            }),
          ),
      )
      .catch((e) => fail({ card: "backup", text: saidPlainly(e) }));
  };

  const takeBackup = () => {
    if (held) return;
    open({ filters: [{ name: "Tisty", extensions: ["zip"] }] })
      .then(async (at) => {
        if (typeof at !== "string") return;
        if (!(await sure(t("restoreSure")))) return;
        run("restore", restore(at), (files) =>
          tell({ card: "restore", text: fill("restored", String(files)) }),
        );
      })
      .catch((e) => fail({ card: "restore", text: saidPlainly(e) }));
  };

  return (
    <>
      <Backup
        state={state}
        holds={holds}
        held={held}
        said={said}
        trouble={trouble}
        onMake={makeBackup}
        onTake={takeBackup}
      >
        <Line
          title={t("parcelsTitle")}
          why={t("parcelsWhy")}
          which="parcel"
          said={said}
          trouble={trouble}
        >
          <button type="button" disabled={held} onClick={onPack} className={mild}>
            {t("packAllDo")}
          </button>
          <button type="button" disabled={held} onClick={onUnpack} className={mild}>
            {t("unpackDo")}
          </button>
        </Line>
      </Backup>

      <Band label={t("whereItLives")} />
      <div className="border-t border-hair">
        <Line
          title={t("aboutStore")}
          why={
            <>
              <span className="block font-mono text-[11.5px] break-all">{build?.store ?? "…"}</span>
              <span className="mt-0.5 block">{t("storeFixed")}</span>
            </>
          }
          which="store"
          said={said}
          trouble={trouble}
        >
          <button
            type="button"
            disabled={!build}
            onClick={() =>
              build &&
              revealed(build.store).catch((e) => fail({ card: "store", text: saidPlainly(e) }))
            }
            className={mild}
          >
            {t("aboutReveal")}
          </button>
        </Line>
        <Line title={t("leaveTitle")} why={t("leaveWhy")} which="store">
          <button type="button" disabled={held} onClick={() => setLeaving(true)} className={risky}>
            {t("leaveOpen")}
          </button>
        </Line>
      </div>
      {leaving && (
        <Leave
          cloud={state.mode === "cloud"}
          store={build?.store}
          onClose={() => setLeaving(false)}
        />
      )}
    </>
  );
}
