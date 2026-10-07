import { ask, open, save } from "@tauri-apps/plugin-dialog";
import {
  type About,
  backUp,
  type Carrying,
  type Freeing,
  freeUp,
  type Holds,
  restore,
  revealed,
  type Settings,
  stopFreeing,
} from "../core";
import { weigh } from "../format";
import { fill, t } from "../locales";
import { saidPlainly } from "../refusal";
import Backup from "./Backup";
import type { Run, Which, Word } from "./Card";
import { Band, Line, mild, off } from "./Rows";
import Syncing from "./Syncing";

const SIZES = [256 * 1024, 1024 * 1024, 5 * 1024 * 1024, 20 * 1024 * 1024, 50 * 1024 * 1024];

interface Props {
  state: Carrying;
  kept: Settings | null;
  holds: string;
  build: About | null;
  freeing: Freeing | null;
  setFreeing: (freeing: Freeing | null) => void;
  busy: Which | null;
  said?: Word;
  trouble?: Word;
  run: Run;
  fail: (word?: Word) => void;
  carry: (way?: "again") => void;
  pickFolder: () => void;
  remember: (next: Settings, card: Which) => void;
  onPack: () => void;
  onUnpack: () => void;
  tell: (word?: Word) => void;
}

export default function Yours({
  state,
  kept,
  holds,
  build,
  freeing,
  setFreeing,
  busy,
  said,
  trouble,
  run,
  fail,
  carry,
  pickFolder,
  remember,
  onPack,
  onUnpack,
  tell,
}: Props) {
  const held = busy !== null;

  const makeBackup = () => {
    if (held) return;
    save({ defaultPath: "tisty-backup.zip", filters: [{ name: "Tisty", extensions: ["zip"] }] })
      .then(
        (at) =>
          typeof at === "string" &&
          run("backup", backUp(at), (bytes) =>
            tell({ card: "backup", text: fill("backupMade", weigh(bytes)) }),
          ),
      )
      .catch((e) => fail({ card: "backup", text: saidPlainly(e) }));
  };

  const takeBackup = () => {
    if (held) return;
    open({ filters: [{ name: "Tisty", extensions: ["zip"] }] })
      .then(async (at) => {
        if (typeof at !== "string") return;
        if (!(await ask(t("restoreSure"), { kind: "warning" }))) return;
        run("restore", restore(at), (files) =>
          tell({ card: "restore", text: fill("restored", String(files)) }),
        );
      })
      .catch((e) => fail({ card: "restore", text: saidPlainly(e) }));
  };

  return (
    <>
      <Syncing
        state={state}
        kept={kept}
        busy={busy}
        said={said}
        trouble={trouble}
        run={run}
        fail={fail}
        carry={carry}
        pickFolder={pickFolder}
      />

      <Band label={t("attachTitle")} />
      <div className="border-t border-hair">
        {kept && (
          <Line
            title={t("attachRow")}
            why={t("attachWhy")}
            which="attach"
            said={said}
            trouble={trouble}
          >
            <select
              aria-label={t("attachUpTo")}
              value={String(kept.attachUpTo)}
              disabled={held}
              onChange={(e) => remember({ ...kept, attachUpTo: Number(e.target.value) }, "attach")}
              className={`rounded-[10px] border border-line bg-bg px-2 py-1 text-[12.5px] ${off}`}
            >
              {SIZES.map((bytes) => (
                <option key={bytes} value={bytes}>
                  {weigh(bytes)}
                </option>
              ))}
            </select>
          </Line>
        )}

        {kept && (
          <Line
            title={t("holdsTitle")}
            why={
              kept.shares ? fill("holdsWhy", weigh(kept.onlySharedAbove)) : t("holdsNeedsShared")
            }
            which="holds"
            said={said}
            trouble={trouble}
            more={
              freeing && (
                <div className="mt-2 flex items-center gap-2.5">
                  <span className="text-[11.5px] leading-relaxed text-soft">
                    {fill(freeing.done ? "holdsFreed" : "holdsFreeing", weigh(freeing.freed))}
                  </span>
                  {!freeing.done && (
                    <button
                      type="button"
                      onClick={() => void stopFreeing()}
                      className="rounded-md border border-line px-2.5 py-0.5 text-[11.5px] text-soft hover:border-urgent hover:text-urgent"
                    >
                      {t("holdsStop")}
                    </button>
                  )}
                </div>
              )
            }
          >
            <select
              aria-label={t("holdsTitle")}
              value={kept.holds}
              disabled={held || !kept.shares}
              onChange={(e) => {
                const holds = e.target.value as Holds;
                remember({ ...kept, holds }, "holds");
                if (holds === "shared") {
                  setFreeing({ gone: 0, freed: 0, done: false });
                  freeUp().catch((e) => {
                    setFreeing(null);
                    fail({ card: "holds", text: saidPlainly(e) });
                  });
                }
                if (holds === "everywhere") {
                  carry();
                }
              }}
              className={`rounded-[10px] border border-line bg-bg px-2 py-1 text-[12.5px] ${off}`}
            >
              <option value="everywhere">{t("holdsEverywhere")}</option>
              <option value="mine">{t("holdsMine")}</option>
              <option value="shared">{t("holdsShared")}</option>
            </select>
          </Line>
        )}
      </div>
      <p className="mt-2 text-[11.5px] leading-relaxed text-faint">{t("attachBig")}</p>

      <Band label={t("bandParcels")} />
      <div className="border-t border-hair">
        <Line
          title={t("packAllPlain")}
          why={t("packAllWhy")}
          which="parcel"
          said={said}
          trouble={trouble}
        >
          <button
            type="button"
            disabled={held}
            onClick={onPack}
            className={`rounded-[10px] border border-line px-2 py-1 text-[12.5px] hover:bg-hover ${off}`}
          >
            {t("packAllDo")}
          </button>
        </Line>
        <Line
          title={t("unpackPlain")}
          why={t("unpackWhy")}
          which="parcel"
          said={said}
          trouble={trouble}
        >
          <button
            type="button"
            disabled={held}
            onClick={onUnpack}
            className={`rounded-[10px] border border-line px-2 py-1 text-[12.5px] hover:bg-hover ${off}`}
          >
            {t("unpackDo")}
          </button>
        </Line>
      </div>

      <Backup
        state={state}
        holds={holds}
        held={held}
        said={said}
        trouble={trouble}
        onMake={makeBackup}
        onTake={takeBackup}
      />

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
      </div>
    </>
  );
}
