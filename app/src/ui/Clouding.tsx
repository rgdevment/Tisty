import { useEffect, useRef, useState } from "react";
import { letGoUnused } from "../cloudState";
import { type Connected, cloudChoose, cloudConnect, cloudConnectCancel, syncState } from "../core";
import { weigh } from "../format";
import { fill, t } from "../locales";
import { saidPlainly } from "../refusal";
import CloudHealth from "./CloudHealth";

interface Props {
  busy: boolean;
  onTrouble: (said?: string) => void;
  onBack: () => void;
  onDone: (at: string) => void;
  onConnected?: (account: Connected) => void;
}

const row = "flex w-full items-center gap-2.5 rounded-[10px] border px-3 py-2 text-left";
const box = "rounded-[10px] border border-hair px-3 py-2 text-[11.5px] leading-relaxed text-soft";
const heading = "block text-[12.5px] font-semibold text-ink";
const back = "text-faint hover:text-ink disabled:opacity-60";
const go =
  "ml-auto rounded-[10px] bg-accent px-4 py-2 text-[13px] font-medium text-white hover:opacity-90 disabled:opacity-60";

// Only the newest sign-in may let an account go: an older one ending late would take a newer token with it.
let asking = 0;

function Mark({ letter, hue }: { letter: string; hue: string }) {
  return (
    <span
      className={`grid size-6 flex-none place-items-center rounded-md text-[11.5px] font-semibold text-white ${hue}`}
    >
      {letter}
    </span>
  );
}

function Points({ title, points }: { title: string; points: string[] }) {
  return (
    <div className={box}>
      <span className={heading}>{title}</span>
      <ul className="mt-1 list-disc pl-4">
        {points.map((one) => (
          <li key={one}>{one}</li>
        ))}
      </ul>
    </div>
  );
}

export default function Clouding({ busy, onTrouble, onBack, onDone, onConnected }: Props) {
  const [waiting, setWaiting] = useState(false);
  const [held, setHeld] = useState(false);
  const [connected, setConnected] = useState<Connected>();
  const [weight, setWeight] = useState<number>();
  const gone = useRef(false);
  const unused = useRef<string | undefined>(undefined);

  useEffect(() => {
    gone.current = false;
    return () => {
      gone.current = true;
      cloudConnectCancel().catch(() => undefined);
      // Closed with an account connected and never started on: its access is let go of.
      if (unused.current) letGoUnused(unused.current);
    };
  }, []);

  useEffect(() => {
    if (!connected) return;
    syncState()
      .then((state) => !gone.current && setWeight(state.weight))
      .catch(() => undefined);
  }, [connected]);

  const stuck = busy || held || waiting;

  const connect = () => {
    if (stuck) return;
    onTrouble(undefined);
    setWaiting(true);
    const mine = ++asking;
    cloudConnect()
      .then((account) => {
        if (gone.current) return mine === asking ? letGoUnused(account.account) : undefined;
        unused.current = account.account;
        setConnected(account);
      })
      .catch((e) => !gone.current && e?.code !== "cloudDropped" && onTrouble(saidPlainly(e)))
      .finally(() => !gone.current && mine === asking && setWaiting(false));
  };

  const stopWaiting = () => {
    cloudConnectCancel().catch(() => undefined);
    setWaiting(false);
  };

  const leave = () => {
    if (busy || held) return;
    onTrouble(undefined);
    if (connected) letGoUnused(connected.account);
    unused.current = undefined;
    setConnected(undefined);
    if (!connected) onBack();
  };

  const start = () => {
    if (stuck || !connected) return;
    if (onConnected) {
      unused.current = undefined;
      return onConnected(connected);
    }
    setHeld(true);
    onTrouble(undefined);
    cloudChoose(connected.account, connected.email)
      .then(() => {
        unused.current = undefined;
        onDone("cloud");
      })
      .catch((e) => {
        onTrouble(saidPlainly(e));
        setHeld(false);
      });
  };

  if (connected) {
    return (
      <div className="flex flex-col gap-3">
        <p className="text-[13px] font-semibold">{t("cloudConnectedTitle")}</p>
        <div className={`${row} border-accent bg-accent-soft`}>
          <Mark letter="G" hue="bg-[#1a73e8]" />
          <span className="min-w-0 flex-1">
            <span className="block text-[13px] font-medium">{t("cloudGoogle")}</span>
            <span className="block truncate text-[11.5px] text-faint">{connected.email}</span>
          </span>
          <span className="flex-none rounded-md bg-hue-green/10 px-1.5 py-0.5 text-[10.5px] text-hue-green">
            {t("cloudConnectedTag")}
          </span>
        </div>
        <Points
          title={t("cloudFirstUpload")}
          points={[
            ...(weight === undefined ? [] : [fill("cloudToUpload", weigh(weight))]),
            t("cloudResumes"),
            t("cloudAttachmentsStay"),
          ]}
        />
        <div className={`${box} bg-accent-soft`}>
          <span className={heading}>{t("cloudGooglePace")}</span>
          {t("cloudGooglePaceWhy")}
        </div>
        <div className="flex items-center gap-4 text-[11.5px]">
          <button type="button" disabled={busy || held} onClick={leave} className={back}>
            {t("welcomeBack")}
          </button>
          <button type="button" disabled={stuck} onClick={start} className={go}>
            {onConnected ? t("welcomeNext") : t("cloudStart")}
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-3">
      <p className="text-[12.5px] leading-relaxed text-soft">{t("cloudConnectWhy")}</p>
      <button
        type="button"
        disabled={stuck}
        onClick={connect}
        className={`${row} border-line hover:bg-hover disabled:opacity-60`}
      >
        <Mark letter="G" hue="bg-[#1a73e8]" />
        <span className="min-w-0 flex-1">
          <span className="block text-[13px] font-medium">{t("cloudGoogle")}</span>
          <span className="block text-[11.5px] text-faint">{t("cloudGoogleWhy")}</span>
        </span>
        <span className="flex-none text-[11.5px] text-accent">{t("cloudConnectDo")}</span>
      </button>
      {(["cloudOneDrive", "cloudDropbox"] as const).map((one) => (
        <div key={one} aria-disabled="true" className={`${row} border-hair opacity-70`}>
          <Mark letter={t(one).slice(0, 1)} hue="bg-soft" />
          <span className="min-w-0 flex-1">
            <span className="block text-[13px] font-medium">{t(one)}</span>
            <span className="block text-[11.5px] text-faint">{t("cloudComing")}</span>
          </span>
        </div>
      ))}
      <CloudHealth name={t("cloudGoogle")} />
      <Points title={t("cloudWhatHappens")} points={[t("cloudFirstAll"), t("cloudNoPassword")]} />
      <Points
        title={t("cloudLimitIsGoogle")}
        points={[t("cloudLimitIsGoogleWhy"), t("cloudCaps")]}
      />
      {waiting && (
        <p role="status" aria-live="polite" className="text-[11.5px] text-faint">
          {t("cloudWaiting")}{" "}
          <button type="button" onClick={stopWaiting} className="text-accent hover:underline">
            {t("cancel")}
          </button>
        </p>
      )}
      <div className="flex items-center gap-4 text-[11.5px]">
        <button type="button" disabled={busy || held} onClick={leave} className={back}>
          {t("welcomeBack")}
        </button>
        <button type="button" disabled={stuck} onClick={connect} className={go}>
          {t("cloudOpenGoogle")}
        </button>
      </div>
    </div>
  );
}
