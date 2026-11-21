import { save } from "@tauri-apps/plugin-dialog";
import type { ReactNode } from "react";
import type { BackedUp, Carrying } from "../core";
import { stamped, weigh } from "../format";
import { fill, t } from "../locales";
import { Line, mild } from "./Rows";

export const whereToBackUp = () =>
  save({ defaultPath: "tisty-backup.zip", filters: [{ name: "Tisty", extensions: ["zip"] }] });

export const backupMadeSaid = (made: BackedUp): string =>
  made.leftOut
    ? fill("backupLeftOut", weigh(made.bytes), `${made.leftOut} · ${weigh(made.leftOutBytes)}`)
    : fill("backupMade", weigh(made.bytes));

interface Props {
  state: Carrying;
  holds: string;
  held: boolean;
  onMake: () => void;
  children?: ReactNode;
}

export default function Backup({ state, holds, held, onMake, children }: Props) {
  return (
    <div className="mt-3 border-t border-hair">
      <Line
        row="backup"
        why={
          <>
            <span className="block">{t(state.freesSpace ? "backupCloudWhat" : "backupWhat")}</span>
            <span className="mt-0.5 block tabular-nums">
              {[
                holds,
                state.weight > state.carries
                  ? fill("backupPastIt", weigh(state.weight), weigh(state.carries))
                  : fill("backupAbout", weigh(state.weight)),
                state.backedUpAt ? stamped(state.backedUpAt) : t("backupNever"),
              ].join(" · ")}
            </span>
          </>
        }
        which="backup"
      >
        <button
          type="button"
          disabled={held || state.weight > state.carries}
          onClick={onMake}
          className={mild}
        >
          {t("backupMake")}
        </button>
      </Line>

      {children}
    </div>
  );
}
