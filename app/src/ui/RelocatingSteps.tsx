import type { Stage } from "../coming";
import type { Carrying, MovePlan, Moving, Toward } from "../core";
import { goesOnAt, weigh } from "../format";
import { fill, t } from "../locales";
import { waitSaid } from "../syncSaid";

export interface Along {
  stage: Stage;
  done: number;
  whole: number;
}

const STAGE = {
  log: "comingLog",
  papers: "comingPapers",
  attachments: "comingAttachments",
  uploading: "comingUploading",
} as const;

const WAITS = {
  budget: "deferredBudget",
  limit: "deferredLimit",
  busy: "deferredBusy",
  changed: "deferredChanged",
  offline: "deferredOffline",
  paused: "deferredPaused",
  troubled: "deferredTroubled",
  unsent: "deferredUnsent",
  unsettled: "moveUnsettled",
} as const;

const row =
  "flex w-full items-center gap-2.5 rounded-[10px] border border-line px-3 py-2 text-left hover:bg-hover disabled:opacity-50";
const box = "rounded-[10px] border border-hair px-3 py-2 text-[11.5px] leading-relaxed text-soft";
export const go =
  "rounded-[10px] bg-accent px-4 py-2 text-[13px] font-medium text-white hover:opacity-90 disabled:opacity-60";
export const quiet = "text-[11.5px] text-faint hover:text-ink disabled:opacity-60";

export const placeOf = (state: Carrying): string =>
  state.mode === "cloud"
    ? `Google Drive · ${state.cloud ?? ""}`
    : state.mode === "folder"
      ? (state.chosen ?? "")
      : t("welcomeAlone");

export const towardOf = (toward: Toward): string =>
  toward.kind === "cloud"
    ? `Google Drive · ${toward.email}`
    : toward.kind === "folder"
      ? toward.at
      : t("welcomeAlone");

export const doneSaid = (toward: Toward): string =>
  toward.kind === "alone"
    ? t("moveDoneAlone")
    : fill("moveDone", toward.kind === "cloud" ? t("cloudGoogle") : toward.at);

function Option({ said, why, onPick }: { said: string; why: string; onPick: () => void }) {
  return (
    <button type="button" className={row} onClick={onPick}>
      <span className="min-w-0 flex-1">
        <span className="block text-[13px] font-medium">{said}</span>
        <span className="block text-[11.5px] text-faint">{why}</span>
      </span>
    </button>
  );
}

export function Pick({
  state,
  onAlone,
  onCloud,
  onFolder,
}: {
  state: Carrying;
  onAlone: () => void;
  onCloud: () => void;
  onFolder: () => void;
}) {
  return (
    <div className="flex flex-col gap-2">
      <p className="mb-1 text-[12.5px] leading-relaxed text-soft">{t("moveWhy")}</p>
      {state.mode !== "alone" && (
        <Option said={t("welcomeAlone")} why={t("welcomeAloneWhy")} onPick={onAlone} />
      )}
      {state.cloudOffered && (
        <Option said={t("chooseCloud")} why={t("chooseCloudWhy")} onPick={onCloud} />
      )}
      <Option said={t("chooseFolder")} why={t("chooseFolderWhy")} onPick={onFolder} />
    </div>
  );
}

export function Plan({
  state,
  toward,
  plan,
  agreed,
  busy,
  onAgree,
  onCancel,
  onMove,
}: {
  state: Carrying;
  toward: Toward;
  plan?: MovePlan;
  agreed: boolean;
  busy: boolean;
  onAgree: (agreed: boolean) => void;
  onCancel: () => void;
  onMove: () => void;
}) {
  return (
    <div className="flex flex-col gap-3">
      <p className="text-[12.5px] leading-relaxed text-soft">{t("moveWhy")}</p>
      <div className="grid grid-cols-[1fr_auto_1fr] items-center gap-2.5">
        <div className="min-w-0 rounded-[10px] border border-hair px-3 py-2">
          <span className="block text-[10.5px] text-faint">{t("moveNow")}</span>
          <span className="block truncate text-[13px] font-medium">{placeOf(state)}</span>
        </div>
        <span aria-hidden="true" className="text-faint">
          →
        </span>
        <div className="min-w-0 rounded-[10px] border border-accent bg-accent-soft px-3 py-2">
          <span className="block text-[10.5px] text-faint">{t("moveThen")}</span>
          <span className="block truncate text-[13px] font-medium">{towardOf(toward)}</span>
        </div>
      </div>
      {plan && (
        <ul className={`${box} list-disc pl-7`}>
          {plan.down.files > 0 && <li>{fill("moveDown", weigh(plan.down.bytes))}</li>}
          {plan.down.files === 0 && <li>{t("moveHomeFirst")}</li>}
          {plan.toCloud && plan.up.files > 0 && <li>{fill("moveUp", weigh(plan.up.bytes))}</li>}
          {plan.toCloud && <li>{t("moveOnDemand")}</li>}
          {plan.toCloud && state.mode === "folder" && <li>{t("moveOldFolderSpace")}</li>}
          <li>{t("moveResumes")}</li>
          {plan.fromCloud && <li>{t("moveKeepsDrive")}</li>}
          {plan.fromCloud && toward.kind === "folder" && <li>{t("moveDesktopTip")}</li>}
          {plan.others > 0 && <li>{t("moveOthersFollow")}</li>}
        </ul>
      )}
      {plan && plan.short > 0 && (
        <p role="alert" className="text-[12.5px] leading-relaxed text-ink">
          {fill("moveNoRoom", weigh(plan.short), weigh(plan.free ?? 0))}
        </p>
      )}
      <label className="flex items-start gap-2 text-[12.5px] text-soft">
        <input
          type="checkbox"
          checked={agreed}
          onChange={(e) => onAgree(e.target.checked)}
          className="mt-0.5"
        />
        {t("moveAgree")}
      </label>
      <div className="flex items-center gap-4">
        <button type="button" onClick={onCancel} className={quiet}>
          {t("cancel")}
        </button>
        <button
          type="button"
          disabled={!agreed || !plan || busy || plan.short > 0}
          onClick={onMove}
          className={`ml-auto ${go}`}
        >
          {t("moveGo")}
        </button>
      </div>
    </div>
  );
}

export function Underway({
  along,
  pausedUntil,
  onHide,
}: {
  along?: Along;
  pausedUntil?: number;
  onHide: () => void;
}) {
  const share = along?.whole ? Math.min(100, Math.round((along.done / along.whole) * 100)) : 10;
  return (
    <div role="status" aria-live="polite" className="flex flex-col gap-2">
      <p className="text-[13px] font-semibold">{t("moveMoving")}</p>
      <p className="text-[12.5px] leading-relaxed text-soft">{t("moveMovingWhy")}</p>
      {pausedUntil !== undefined && (
        <p className="text-[11.5px] text-faint tabular-nums">{goesOnAt(pausedUntil)}</p>
      )}
      {along && pausedUntil === undefined && (
        <p className="text-[11.5px] text-faint tabular-nums">
          {t(STAGE[along.stage])} · {fill("comingOf", String(along.done), String(along.whole))}
        </p>
      )}
      <span aria-hidden="true" className="block h-1 overflow-hidden rounded-full bg-line">
        <span
          className="block h-full rounded-full bg-accent motion-safe:transition-[width]"
          style={{ width: `${share}%` }}
        />
      </span>
      <button type="button" onClick={onHide} className={`mt-1 self-end ${go}`}>
        {t("moveHide")}
      </button>
    </div>
  );
}

export function Waiting({
  result,
  busy,
  onDrop,
  onLater,
  onAgain,
}: {
  result: Extract<Moving, { state: "waiting" }>;
  busy: boolean;
  onDrop: () => void;
  onLater: () => void;
  onAgain: () => void;
}) {
  const why =
    result.reason && result.reason in WAITS
      ? waitSaid(WAITS[result.reason as keyof typeof WAITS])
      : t("moveStuck");
  return (
    <div className="flex flex-col gap-3">
      <div className={box}>
        <span className="block text-[12.5px] font-semibold text-ink">{t("moveWaiting")}</span>
        {why}{" "}
        {result.left.files > 0 &&
          (result.left.files === 1
            ? fill("moveLeftOne", "1", weigh(result.left.bytes))
            : fill("moveLeft", String(result.left.files), weigh(result.left.bytes)))}
      </div>
      <div className="flex items-center gap-4">
        <button type="button" onClick={onDrop} className={quiet}>
          {t("moveDrop")}
        </button>
        <button type="button" onClick={onLater} className={`ml-auto ${quiet}`}>
          {t("moveLater")}
        </button>
        <button type="button" disabled={busy} onClick={onAgain} className={go}>
          {t("moveAgain")}
        </button>
      </div>
    </div>
  );
}

export function Done({
  toward,
  untold,
  wiped,
  offersWipe,
  busy,
  onKeep,
  onWipe,
  onClose,
}: {
  toward: Toward;
  untold: boolean;
  wiped?: string;
  offersWipe: boolean;
  busy: boolean;
  onKeep: () => void;
  onWipe: () => void;
  onClose: () => void;
}) {
  return (
    <div className="flex flex-col gap-3">
      <p className="text-[13px] font-semibold">{doneSaid(toward)}</p>
      <p className="text-[12.5px] leading-relaxed text-soft">
        {t(untold ? "moveUntold" : "moveDoneWhy")}
      </p>
      {wiped && <p className="text-[11.5px] text-faint">{wiped}</p>}
      <div className="flex flex-wrap items-center gap-2.5">
        {offersWipe && !wiped && (
          <>
            <button type="button" disabled={busy} onClick={onKeep} className={quiet}>
              {t("moveWipeKeep")}
            </button>
            <button
              type="button"
              disabled={busy}
              onClick={onWipe}
              className="rounded-[10px] border border-urgent/40 px-3 py-1.5 text-[12.5px] text-urgent hover:bg-urgent/10 disabled:opacity-60"
            >
              {t("moveWipe")}
            </button>
          </>
        )}
        <button type="button" disabled={busy} onClick={onClose} className={`ml-auto ${go}`}>
          {t("close")}
        </button>
      </div>
    </div>
  );
}
