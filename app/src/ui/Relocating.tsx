import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef, useState } from "react";
import { letGoUnused } from "../cloudState";
import {
  type Carrying,
  cloudForget,
  cloudWipe,
  type MovePlan,
  type Moving,
  moveDropped,
  movePlan,
  moveTo,
  type Toward,
} from "../core";
import { weigh } from "../format";
import { fill, t } from "../locales";
import { saidPlainly } from "../refusal";
import { sure } from "../sure";
import type { Heading } from "../views";
import Clouding from "./Clouding";
import Keepers from "./Keepers";
import Modal from "./Modal";

interface Props {
  state: Carrying;
  heading?: Heading;
  resume?: Toward;
  onClose: () => void;
  onMoved: () => void;
}

type Step = "pick" | "folder" | "cloud" | "plan" | "moving" | "waiting" | "done";

interface Along {
  stage: "log" | "papers" | "attachments";
  done: number;
  whole: number;
}

const STAGE = {
  log: "comingLog",
  papers: "comingPapers",
  attachments: "comingAttachments",
} as const;

const WAITS = {
  budget: "deferredBudget",
  limit: "deferredLimit",
  busy: "deferredLimit",
  changed: "deferredLimit",
  offline: "deferredOffline",
  paused: "deferredPaused",
  unsettled: "moveUnsettled",
} as const;

const row =
  "flex w-full items-center gap-2.5 rounded-[10px] border border-line px-3 py-2 text-left hover:bg-hover disabled:opacity-50";
const box = "rounded-[10px] border border-hair px-3 py-2 text-[11.5px] leading-relaxed text-soft";
const go =
  "rounded-[10px] bg-accent px-4 py-2 text-[13px] font-medium text-white hover:opacity-90 disabled:opacity-60";
const quiet = "text-[11.5px] text-faint hover:text-ink disabled:opacity-60";

export const placeOf = (state: Carrying): string =>
  state.mode === "cloud"
    ? `Google Drive · ${state.cloud ?? ""}`
    : state.mode === "folder"
      ? (state.chosen ?? "")
      : t("welcomeAlone");

const towardOf = (toward: Toward): string =>
  toward.kind === "cloud"
    ? `Google Drive · ${toward.email}`
    : toward.kind === "folder"
      ? toward.at
      : t("welcomeAlone");

const firstStep = (heading?: Heading, resume?: Toward): Step =>
  resume ? "plan" : heading === "cloud" ? "cloud" : heading === "folder" ? "folder" : "pick";

export default function Relocating({ state, heading, resume, onClose, onMoved }: Props) {
  const [step, setStep] = useState<Step>(firstStep(heading, resume));
  const [toward, setToward] = useState<Toward | undefined>(resume);
  const [plan, setPlan] = useState<MovePlan>();
  const [agreed, setAgreed] = useState(false);
  const [along, setAlong] = useState<Along>();
  const [result, setResult] = useState<Moving>();
  const [trouble, setTrouble] = useState<string>();
  const [told, setTold] = useState<string>();
  const [busy, setBusy] = useState(false);
  const settled = useRef(false);

  useEffect(() => {
    if (step !== "plan" || !toward) return;
    setPlan(undefined);
    movePlan(toward)
      .then(setPlan)
      .catch((e) => setTrouble(saidPlainly(e)));
  }, [step, toward]);

  useEffect(() => {
    if (step !== "moving") return;
    const off = listen<Along>("bringing", ({ payload }) => setAlong(payload));
    return () => {
      void off.then((stop) => stop());
    };
  }, [step]);

  const chose = (next: Toward) => {
    setTrouble(undefined);
    setToward(next);
    setStep("plan");
  };

  const leftCloud = result?.state === "done" && result.told ? result.leftCloud : undefined;
  const untold = result?.state === "done" && !result.told;

  const close = () => {
    if (busy) return;
    if (toward?.kind === "cloud" && !result) letGoUnused(toward.account);
    if (leftCloud && !settled.current) void cloudForget(leftCloud.account).catch(() => undefined);
    if (result?.state === "done") onMoved();
    onClose();
  };

  const move = () => {
    if (!toward || busy) return;
    setBusy(true);
    setTrouble(undefined);
    setAlong(undefined);
    setStep("moving");
    moveTo(toward)
      .then((now) => {
        setResult(now);
        setStep(now.state === "done" ? "done" : "waiting");
      })
      .catch((e) => {
        setTrouble(saidPlainly(e));
        setStep("plan");
      })
      .finally(() => setBusy(false));
  };

  const drop = () => {
    void moveDropped().catch(() => undefined);
    if (toward?.kind === "cloud") letGoUnused(toward.account);
    if (result?.state === "done") onMoved();
    onClose();
  };

  const wipe = async () => {
    if (!leftCloud || busy || !(await sure(t("moveWipeSure")))) return;
    setBusy(true);
    cloudWipe(leftCloud.account, leftCloud.email)
      .then((gone) => {
        settled.current = true;
        setTold(fill("moveWiped", String(gone)));
      })
      .catch((e) => setTrouble(saidPlainly(e)))
      .finally(() => setBusy(false));
  };

  const keep = () => {
    if (!leftCloud) return;
    settled.current = true;
    void cloudForget(leftCloud.account).catch(() => undefined);
    close();
  };

  return (
    <Modal title={t("moveTitle")} wide onClose={close}>
      {step === "pick" && (
        <div className="flex flex-col gap-2">
          <p className="mb-1 text-[12.5px] leading-relaxed text-soft">{t("moveWhy")}</p>
          {state.mode !== "alone" && (
            <button type="button" className={row} onClick={() => chose({ kind: "alone" })}>
              <span className="min-w-0 flex-1">
                <span className="block text-[13px] font-medium">{t("welcomeAlone")}</span>
                <span className="block text-[11.5px] text-faint">{t("welcomeAloneWhy")}</span>
              </span>
            </button>
          )}
          {state.cloudOffered && (
            <button type="button" className={row} onClick={() => setStep("cloud")}>
              <span className="min-w-0 flex-1">
                <span className="block text-[13px] font-medium">{t("chooseCloud")}</span>
                <span className="block text-[11.5px] text-faint">{t("chooseCloudWhy")}</span>
              </span>
            </button>
          )}
          <button type="button" className={row} onClick={() => setStep("folder")}>
            <span className="min-w-0 flex-1">
              <span className="block text-[13px] font-medium">{t("chooseFolder")}</span>
              <span className="block text-[11.5px] text-faint">{t("chooseFolderWhy")}</span>
            </span>
          </button>
        </div>
      )}

      {step === "folder" && (
        <div className="flex flex-col gap-3">
          <Keepers
            busy={busy}
            pickOnly
            onTrouble={(said) => setTrouble(said)}
            onDone={(at) => chose({ kind: "folder", at })}
          />
          <button type="button" onClick={() => setStep("pick")} className={`self-start ${quiet}`}>
            {t("welcomeBack")}
          </button>
        </div>
      )}

      {step === "cloud" && (
        <Clouding
          busy={busy}
          onTrouble={setTrouble}
          onBack={() => setStep("pick")}
          onDone={() => undefined}
          onConnected={(account) =>
            chose({ kind: "cloud", account: account.account, email: account.email })
          }
        />
      )}

      {step === "plan" && toward && (
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
              {plan.toCloud && plan.up.files > 0 && <li>{fill("moveUp", weigh(plan.up.bytes))}</li>}
              <li>{t("moveResumes")}</li>
              {plan.fromCloud && <li>{t("moveKeepsDrive")}</li>}
              {plan.fromCloud && toward.kind === "folder" && <li>{t("moveDesktopTip")}</li>}
            </ul>
          )}
          <label className="flex items-start gap-2 text-[12.5px] text-soft">
            <input
              type="checkbox"
              checked={agreed}
              onChange={(e) => setAgreed(e.target.checked)}
              className="mt-0.5"
            />
            {t("moveAgree")}
          </label>
          <div className="flex items-center gap-4">
            <button type="button" onClick={close} className={quiet}>
              {t("cancel")}
            </button>
            <button
              type="button"
              disabled={!agreed || !plan || busy}
              onClick={move}
              className={`ml-auto ${go}`}
            >
              {t("moveGo")}
            </button>
          </div>
        </div>
      )}

      {step === "moving" && (
        <div role="status" aria-live="polite" className="flex flex-col gap-2">
          <p className="text-[13px] font-semibold">{t("moveMoving")}</p>
          <p className="text-[12.5px] leading-relaxed text-soft">{t("moveMovingWhy")}</p>
          {along && (
            <p className="text-[11.5px] text-faint tabular-nums">
              {t(STAGE[along.stage])} · {fill("comingOf", String(along.done), String(along.whole))}
            </p>
          )}
          <span aria-hidden="true" className="block h-1 overflow-hidden rounded-full bg-line">
            <span
              className="block h-full rounded-full bg-accent motion-safe:transition-[width]"
              style={{
                width: `${along?.whole ? Math.min(100, Math.round((along.done / along.whole) * 100)) : 10}%`,
              }}
            />
          </span>
        </div>
      )}

      {step === "waiting" && result?.state === "waiting" && (
        <div className="flex flex-col gap-3">
          <div className={box}>
            <span className="block text-[12.5px] font-semibold text-ink">{t("moveWaiting")}</span>
            {result.reason && result.reason in WAITS
              ? t(WAITS[result.reason as keyof typeof WAITS])
              : t("moveStuck")}{" "}
            {result.left.files > 0 &&
              fill("moveLeft", String(result.left.files), weigh(result.left.bytes))}
          </div>
          <div className="flex items-center gap-4">
            <button type="button" onClick={drop} className={quiet}>
              {t("moveDrop")}
            </button>
            <button type="button" onClick={close} className={`ml-auto ${quiet}`}>
              {t("moveLater")}
            </button>
            <button type="button" disabled={busy} onClick={move} className={go}>
              {t("moveAgain")}
            </button>
          </div>
        </div>
      )}

      {step === "done" && toward && (
        <div className="flex flex-col gap-3">
          <p className="text-[13px] font-semibold">{fill("moveDone", towardOf(toward))}</p>
          <p className="text-[12.5px] leading-relaxed text-soft">
            {t(untold ? "moveUntold" : "moveDoneWhy")}
          </p>
          {told && <p className="text-[11.5px] text-faint">{told}</p>}
          <div className="flex flex-wrap items-center gap-2.5">
            {leftCloud && !told && (
              <>
                <button type="button" disabled={busy} onClick={keep} className={quiet}>
                  {t("moveWipeKeep")}
                </button>
                <button
                  type="button"
                  disabled={busy}
                  onClick={wipe}
                  className="rounded-[10px] border border-urgent/40 px-3 py-1.5 text-[12.5px] text-urgent hover:bg-urgent/10 disabled:opacity-60"
                >
                  {t("moveWipe")}
                </button>
              </>
            )}
            <button type="button" disabled={busy} onClick={close} className={`ml-auto ${go}`}>
              {t("close")}
            </button>
          </div>
        </div>
      )}

      {trouble && (
        <p role="alert" className="mt-3 text-[11.5px] text-urgent">
          {trouble}
        </p>
      )}
    </Modal>
  );
}
