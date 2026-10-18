import { useEffect, useRef, useState } from "react";
import { letGoUnused } from "../cloudState";
import {
  type Connected,
  cloudChoose,
  cloudConnect,
  cloudConnectCancel,
  placePeek,
  syncState,
} from "../core";
import { weigh } from "../format";
import { fill, said, speakOf, t, type Word } from "../locales";
import { saidPlainly } from "../refusal";
import CloudHealth from "./CloudHealth";
import CloudMark from "./CloudMark";

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

const CLOUDS: { provider: string; name: Word; why: Word }[] = [
  { provider: "google", name: "cloudGoogle", why: "cloudGoogleWhy" },
  { provider: "onedrive", name: "cloudOneDrive", why: "cloudOneDriveWhy" },
  { provider: "dropbox", name: "cloudDropbox", why: "cloudComing" },
];

const nameOf = (provider: string) =>
  t(CLOUDS.find((one) => one.provider === provider)?.name ?? "cloudGoogle");

// Only the newest sign-in may let an account go: an older one ending late would take a newer token with it.
let asking = 0;

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
  const [holds, setHolds] = useState(false);
  const [offered, setOffered] = useState<string[]>();
  const [picked, setPicked] = useState<string>();
  const gone = useRef(false);
  const unused = useRef<Connected | undefined>(undefined);

  useEffect(() => {
    gone.current = false;
    // A window that cannot say which clouds it has offers Google alone, as every build before OneDrive did.
    const learned = (clouds?: string[]) => {
      if (gone.current) return;
      const known = clouds?.length ? clouds : ["google"];
      setOffered(known);
      setPicked((was) => (was && known.includes(was) ? was : known[0]));
    };
    syncState()
      .then((state) => learned(state?.clouds))
      .catch(() => learned());
    return () => {
      gone.current = true;
      cloudConnectCancel().catch(() => undefined);
      // Closed with an account connected and never started on: its access is let go of.
      if (unused.current) letGoUnused(unused.current);
    };
  }, []);

  useEffect(() => {
    if (!connected) return;
    setHolds(false);
    syncState()
      .then((state) => !gone.current && setWeight(state.weight))
      .catch(() => undefined);
    if (onConnected) return;
    placePeek({
      kind: "cloud",
      account: connected.account,
      email: connected.email,
      provider: connected.provider,
    })
      // A computer with a history of its own goes through how the two come together, never «bring it».
      .then((peeked) => !gone.current && setHolds(peeked.fresh && peeked.holds === true))
      .catch(() => undefined);
  }, [connected, onConnected]);

  const stuck = busy || held || waiting || !picked;

  const connect = (provider: string) => {
    if (stuck) return;
    onTrouble(undefined);
    setPicked(provider);
    setWaiting(true);
    const mine = ++asking;
    cloudConnect(provider)
      .then((account) => {
        if (gone.current) return mine === asking ? letGoUnused(account) : undefined;
        unused.current = account;
        speakOf(account.provider);
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
    if (connected) letGoUnused(connected);
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
    cloudChoose(connected.account, connected.email, connected.provider)
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
    const of = (key: Word) => said(key, connected.provider);
    return (
      <div className="flex flex-col gap-3">
        <p className="text-[13px] font-semibold">{t("cloudConnectedTitle")}</p>
        <div className={`${row} border-accent bg-accent-soft`}>
          <CloudMark provider={connected.provider} />
          <span className="min-w-0 flex-1">
            <span className="block text-[13px] font-medium">{nameOf(connected.provider)}</span>
            <span className="block truncate text-[11.5px] text-faint">{connected.email}</span>
          </span>
          <span className="flex-none rounded-md bg-hue-green/10 px-1.5 py-0.5 text-[10.5px] text-hue-green">
            {t("cloudConnectedTag")}
          </span>
        </div>
        {holds ? (
          <div className={box}>
            <span className={heading}>{fill("cloudHolds", nameOf(connected.provider))}</span>
            {t("cloudHoldsWhy")}
          </div>
        ) : (
          <Points
            title={t("cloudFirstUpload")}
            points={[
              ...(weight === undefined ? [] : [fill("cloudToUpload", weigh(weight))]),
              t("cloudResumes"),
              t("cloudAttachmentsStay"),
            ]}
          />
        )}
        <div className={`${box} bg-accent-soft`}>
          <span className={heading}>{of("cloudGooglePace")}</span>
          {of("cloudGooglePaceWhy")}
        </div>
        <div className="flex items-center gap-4 text-[11.5px]">
          <button type="button" disabled={busy || held} onClick={leave} className={back}>
            {t("welcomeBack")}
          </button>
          <button type="button" disabled={stuck} onClick={start} className={go}>
            {onConnected ? t("welcomeNext") : holds ? t("cloudBring") : t("cloudStart")}
          </button>
        </div>
      </div>
    );
  }

  const of = (key: Word) => said(key, picked ?? "google");
  return (
    <div className="flex flex-col gap-3">
      <p className="text-[12.5px] leading-relaxed text-soft">{of("cloudConnectWhy")}</p>
      {CLOUDS.map((one) =>
        offered?.includes(one.provider) ? (
          <button
            key={one.provider}
            type="button"
            disabled={stuck}
            onClick={() => connect(one.provider)}
            className={`${row} ${one.provider === picked ? "border-accent" : "border-line"} hover:bg-hover disabled:opacity-60`}
          >
            <CloudMark provider={one.provider} />
            <span className="min-w-0 flex-1">
              <span className="block text-[13px] font-medium">{t(one.name)}</span>
              <span className="block text-[11.5px] text-faint">{t(one.why)}</span>
            </span>
            <span className="flex-none text-[11.5px] text-accent">{t("cloudConnectDo")}</span>
          </button>
        ) : (
          <div key={one.provider} aria-disabled="true" className={`${row} border-hair opacity-70`}>
            <CloudMark named={t(one.name)} />
            <span className="min-w-0 flex-1">
              <span className="block text-[13px] font-medium">{t(one.name)}</span>
              <span className="block text-[11.5px] text-faint">{t("cloudComing")}</span>
            </span>
          </div>
        ),
      )}
      {picked && <CloudHealth name={nameOf(picked)} provider={picked} />}
      <Points title={t("cloudWhatHappens")} points={[t("cloudFirstAll"), of("cloudNoPassword")]} />
      <Points
        title={of("cloudLimitIsGoogle")}
        points={[of("cloudLimitIsGoogleWhy"), t("cloudCaps")]}
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
        <button
          type="button"
          disabled={stuck}
          onClick={() => picked && connect(picked)}
          className={go}
        >
          {of("cloudOpenGoogle")}
        </button>
      </div>
    </div>
  );
}
