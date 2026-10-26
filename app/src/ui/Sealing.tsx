import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useState } from "react";
import {
  identityBegin,
  identityPhrase,
  identityShown,
  type SealRoom,
  type SealShown,
  type SealTold,
  sealingBegin,
  sealingCancel,
  sealingResume,
  sealingRoom,
  sealingShown,
  sealingWipe,
} from "../core";
import { weigh } from "../format";
import { cloudNamed, fill, t } from "../locales";
import { plainly, saidPlainly } from "../refusal";
import { sure } from "../sure";
import { causeSaid, waitSaid } from "../syncSaid";
import { LetMeIn, LetThemIn } from "./Admitting";
import Card, { type Which, type Word } from "./Card";
import Entering from "./Entering";
import Modal from "./Modal";
import Phrasing from "./Phrasing";
import { mild, risky, strong } from "./Rows";

interface Props {
  round?: string | null;
  phrased?: number;
  onPhrased?: () => void;
  busy: Which | null;
  said?: Word;
  trouble?: Word;
  tell: (word?: Word) => void;
  fail: (word?: Word) => void;
  onChanged: () => void;
}

type Phrase = { words: string[]; fingerprint: string };

const card = "sealing" as const;

export default function Sealing({
  round,
  phrased,
  onPhrased,
  busy,
  said,
  trouble,
  tell,
  fail,
  onChanged,
}: Props) {
  const [shown, setShown] = useState<SealShown | null>(null);
  const [asking, setAsking] = useState(false);
  const [starting, setStarting] = useState(false);
  const [wiping, setWiping] = useState(false);
  const [phrase, setPhrase] = useState<Phrase | null>(null);
  const [letting, setLetting] = useState<"me" | "them" | "phrase" | null>(null);
  const failed = useCallback((e: unknown) => fail({ card, text: saidPlainly(e) }), [fail]);

  const look = useCallback(() => {
    sealingShown().then(setShown).catch(failed);
  }, [failed]);
  useEffect(look, [look, round, phrased]);
  useEffect(() => {
    if (!shown?.wiping || wiping) return;
    const again = setInterval(look, 5_000);
    return () => clearInterval(again);
  }, [shown?.wiping, wiping, look]);
  useEffect(() => {
    const off = listen<SealTold | null>("sealing", (told) => {
      setShown((was) => (was ? { ...was, turning: told.payload } : was));
      if (!told.payload || told.payload.step === "done") {
        look();
        onChanged();
      }
    });
    return () => {
      void off.then((stop) => stop());
    };
  }, [look, onChanged]);

  const phraseFor = (made: Promise<string>) => {
    made
      .then((said) =>
        identityShown().then((who) =>
          setPhrase({ words: said.split(" "), fingerprint: who.fingerprint ?? "" }),
        ),
      )
      .catch(failed);
  };

  const phrasedHere = () => {
    setPhrase(null);
    look();
    onPhrased?.();
  };

  const begin = (on: boolean) => {
    setAsking(false);
    setStarting(true);
    sealingBegin(on)
      .then(() => sealingShown())
      .then(setShown)
      .catch(failed)
      .finally(() => setStarting(false));
  };

  if (!shown)
    return (
      <Card title={t("sealTitle")} which={card} busy={busy} said={said} trouble={trouble}>
        {trouble?.card === card ? (
          <button type="button" onClick={look} className={mild}>
            {t("tryAgain")}
          </button>
        ) : (
          <p role="status" className="text-[12.5px] text-faint motion-safe:animate-pulse">
            {t("checkingNow")}
          </p>
        )}
      </Card>
    );
  const provider = cloudNamed(shown.cloud);
  const held = busy !== null;

  return (
    <Card title={t("sealTitle")} which={card} busy={busy} said={said} trouble={trouble}>
      {asking && (
        <Before
          shown={shown}
          provider={provider}
          onCreate={() => phraseFor(identityBegin())}
          onConfirm={() => phraseFor(identityPhrase())}
          onClose={() => setAsking(false)}
          onGo={() => begin(true)}
        />
      )}
      {phrase && (
        <Phrasing
          words={phrase.words}
          fingerprint={phrase.fingerprint}
          onClose={phrasedHere}
          onConfirmed={phrasedHere}
        />
      )}
      {letting === "me" && (
        <LetMeIn
          onClose={() => setLetting(null)}
          onIn={() => {
            look();
            onChanged();
          }}
        />
      )}
      {letting === "them" && <LetThemIn onClose={() => setLetting(null)} />}
      {letting === "phrase" && (
        <Entering
          onClose={() => setLetting(null)}
          onIn={() => {
            look();
            onChanged();
          }}
        />
      )}
      {shown.state === "locked" && (
        <>
          <p className="text-[13px] font-semibold">{t("sealLocked")}</p>
          <p className="mt-0.5 text-[12.5px] leading-relaxed text-soft">
            {t(shown.joined ? "sealLockedJoined" : "sealLockedNew")}
          </p>
          <div className="mt-3 flex flex-wrap justify-end gap-2">
            <button
              type="button"
              className={mild}
              disabled={held}
              onClick={() => setLetting("phrase")}
            >
              {t("enterButton")}
            </button>
            {!shown.joined && (
              <button
                type="button"
                className={strong}
                disabled={held}
                onClick={() => setLetting("me")}
              >
                {t("admitAskButton")}
              </button>
            )}
          </div>
        </>
      )}
      {shown.state === "none" && (
        <p className="text-[12.5px] leading-relaxed text-soft">{t("sealNoCloud")}</p>
      )}
      {starting && shown.state !== "turning" && (
        <p
          role="status"
          className="text-[12.5px] leading-relaxed text-soft motion-safe:animate-pulse"
        >
          {t("sealStarting")}
        </p>
      )}
      {shown.state === "plain" && !starting && (
        <>
          <p className="text-[12.5px] leading-relaxed text-soft">{fill("sealWhat", provider)}</p>
          <div className="mt-3 flex justify-end">
            <button
              type="button"
              className={strong}
              disabled={held}
              onClick={() => setAsking(true)}
            >
              {t("sealStart")}
            </button>
          </div>
        </>
      )}
      {shown.state === "turning" && shown.turning && (
        <Progress
          told={shown.turning}
          provider={provider}
          onAgain={() => sealingResume().catch(failed)}
          onCancel={() =>
            sure(t("sealCancelSure"))
              .then((yes) => (yes ? sealingCancel().then(look) : undefined))
              .catch(failed)
          }
        />
      )}
      {shown.state === "sealed" && !starting && (
        <Sealed
          provider={provider}
          held={held}
          wiping={wiping || Boolean(shown.wiping)}
          onWipe={() =>
            sure(fill("sealWipeSure", provider))
              .then((yes) => {
                if (!yes) return;
                setWiping(true);
                return sealingWipe()
                  .then((gone) => tell({ card, text: fill("sealWiped", String(gone)) }))
                  .finally(() => {
                    setWiping(false);
                    look();
                  });
              })
              .catch(failed)
          }
          onLetIn={() => setLetting("them")}
          onOff={() =>
            sure(fill("sealOffSure", provider))
              .then((yes) => (yes ? begin(false) : undefined))
              .catch(failed)
          }
        />
      )}
      {shown.state === "elsewhere" && (
        <>
          <p className="text-[12.5px] leading-relaxed text-soft">{t("sealElsewhere")}</p>
          {shown.orphan && (
            <>
              <p className="mt-1 text-[12.5px] text-urgent">{t("sealOrphan")}</p>
              <div className="mt-3 flex justify-end">
                <button
                  type="button"
                  className={risky}
                  disabled={held}
                  onClick={() =>
                    sure(t("sealCancelSure"))
                      .then((yes) => (yes ? sealingCancel().then(look) : undefined))
                      .catch(failed)
                  }
                >
                  {t("sealOrphanCancel")}
                </button>
              </div>
            </>
          )}
        </>
      )}
    </Card>
  );
}

function Mark({ ok }: { ok: boolean }) {
  return (
    <span aria-hidden="true" className={`w-4 shrink-0 ${ok ? "text-hue-teal" : "text-urgent"}`}>
      {ok ? "✓" : "✕"}
    </span>
  );
}

function Before({
  shown,
  provider,
  onCreate,
  onConfirm,
  onClose,
  onGo,
}: {
  shown: SealShown;
  provider: string;
  onCreate: () => void;
  onConfirm: () => void;
  onClose: () => void;
  onGo: () => void;
}) {
  const [room, setRoom] = useState<SealRoom | "unread" | null>(null);
  useEffect(() => {
    sealingRoom()
      .then(setRoom)
      .catch(() => setRoom("unread"));
  }, []);
  const roomy =
    room !== null && room !== "unread" && (room.free === null || room.free >= room.needed);
  const phraseOk = shown.held && shown.confirmed;
  const ready = phraseOk && shown.behind.length === 0 && roomy;

  return (
    <Modal title={t("sealTitle")} wide onClose={onClose}>
      <p className="text-[12.5px] leading-relaxed text-soft">{fill("sealWhat", provider)}</p>
      <p className="mt-4 text-[11.5px] font-semibold text-faint uppercase">{t("sealBefore")}</p>
      <ul className="mt-2 flex flex-col gap-2 text-[12.5px]">
        <li className="flex gap-2">
          <Mark ok={phraseOk} />
          <span className="min-w-0 flex-1">
            {phraseOk
              ? t("sealPhraseOk")
              : shown.phraseElsewhere
                ? t("sealPhraseAway")
                : t(shown.held ? "sealPhraseUnconfirmed" : "sealPhraseMissing")}
          </span>
          {!phraseOk && !shown.phraseElsewhere && (
            <button type="button" className={mild} onClick={shown.held ? onConfirm : onCreate}>
              {t(shown.held ? "sealPhraseConfirm" : "sealPhraseCreate")}
            </button>
          )}
        </li>
        <li className="flex gap-2">
          {room === null ? (
            <span role="status" className="text-faint motion-safe:animate-pulse">
              {fill("sealRoomLooking", provider)}
            </span>
          ) : room === "unread" ? (
            <>
              <Mark ok={false} />
              <span className="min-w-0 flex-1">{fill("sealRoomUnread", provider)}</span>
            </>
          ) : (
            <>
              <Mark ok={roomy} />
              <span className="min-w-0 flex-1">
                <span className="block">
                  {fill(roomy ? "sealRoomOk" : "sealRoomShort", provider)}
                </span>
                <span className="block text-soft">
                  {room.free === null
                    ? fill("sealRoomUnknown", provider, weigh(room.needed))
                    : fill("sealRoomWhy", weigh(room.needed), weigh(room.free))}
                </span>
              </span>
            </>
          )}
        </li>
        {shown.behind.length === 0 ? (
          <li className="flex gap-2">
            <Mark ok />
            <span>{t("sealUpToDate")}</span>
          </li>
        ) : (
          shown.behind.map((name) => (
            <li key={name} className="flex gap-2">
              <Mark ok={false} />
              <span className="min-w-0 flex-1">
                <span className="block">{fill("sealBehindOne", name)}</span>
                <span className="block text-soft">{t("sealBehindWhy")}</span>
              </span>
            </li>
          ))
        )}
      </ul>
      <p className="mt-4 text-[11.5px] font-semibold text-faint uppercase">{t("sealKnow")}</p>
      <p className="mt-1 text-[12.5px] leading-relaxed text-soft">
        {fill("sealKnowWhat", provider)}
      </p>
      <div className="mt-4 flex flex-wrap justify-end gap-2">
        <button type="button" className={mild} onClick={onClose}>
          {t("sealNotNow")}
        </button>
        <button type="button" className={strong} disabled={!ready} onClick={onGo}>
          {t("sealGo")}
        </button>
      </div>
    </Modal>
  );
}

const STEPS = ["copying", "settling", "done"] as const;

const waitingSaid = (on: NonNullable<SealTold["waiting"]>, provider: string): string => {
  if (on === "local") return t("sealWaitLocal");
  if (on === "busy") return t("deferredBusy");
  if (on === "budget") return waitSaid("deferredBudget");
  if (on === "paused") return t("deferredPaused");
  if (on === "cloud") return fill("sealWaitCloud", provider);
  if (on === "offline") return fill("sealWaitOffline", provider);
  return causeSaid(on, provider) ?? fill("sealWaitOffline", provider);
};

function Progress({
  told,
  provider,
  onCancel,
  onAgain,
}: {
  told: SealTold;
  provider: string;
  onCancel: () => void;
  onAgain: () => void;
}) {
  const at = told.step === "claimed" ? 0 : told.step === "closed" ? 1 : STEPS.indexOf(told.step);
  const step = (index: number, label: string, detail?: string) => (
    <li className="flex gap-2">
      <span
        aria-hidden="true"
        className={`w-4 shrink-0 ${index < at ? "text-hue-teal" : "text-faint"} ${index === at && !told.stopped ? "motion-safe:animate-pulse" : ""}`}
      >
        {index < at ? "✓" : index === at ? "…" : "·"}
      </span>
      <span className="min-w-0 flex-1">
        <span className="block">{label}</span>
        {detail && <span className="block text-soft">{detail}</span>}
      </span>
    </li>
  );
  const going = !told.stopped && !told.waiting;
  return (
    <>
      <p className="text-[13px] font-semibold">{t(told.on ? "sealTurningOn" : "sealTurningOff")}</p>
      <p className="mt-0.5 text-[12.5px] leading-relaxed text-soft">{t("sealKeepWorking")}</p>
      <ul className="mt-3 flex flex-col gap-2 text-[12.5px]">
        {step(
          0,
          t("sealStepCopy"),
          told.step === "copying" && told.done + told.left > 0
            ? fill("sealStepCopied", String(told.done), String(told.left))
            : at === 0 && going
              ? t("sealStepCounting")
              : undefined,
        )}
        {step(
          1,
          t("sealStepClose"),
          told.step === "settling" && told.left > 0
            ? fill("sealStepSettlingLeft", String(told.left))
            : t(at === 1 && going ? "sealStepSettling" : "sealStepCloseWhy"),
        )}
        {step(2, t(told.on ? "sealStepSwitchOn" : "sealStepSwitchOff"))}
      </ul>
      {told.refused > 0 && (
        <p className="mt-2 text-[12.5px] text-soft">{fill("sealRefused", String(told.refused))}</p>
      )}
      {told.waiting && !told.stopped && (
        <p role="status" className="mt-2 text-[12.5px] text-soft">
          {waitingSaid(told.waiting, provider)}
        </p>
      )}
      {told.stopped && (
        <p role="alert" className="mt-2 text-[12.5px] text-urgent">
          {told.why ? plainly({ code: told.why }) : t("sealStopped")}
        </p>
      )}
      <p className="mt-3 text-[11.5px] text-faint">{t("sealIntact")}</p>
      <div className="mt-3 flex flex-wrap justify-end gap-2">
        <button type="button" className={mild} onClick={onCancel}>
          {t("cancel")}
        </button>
        {told.stopped && (
          <button type="button" className={strong} onClick={onAgain}>
            {t("tryAgain")}
          </button>
        )}
      </div>
    </>
  );
}

function Sealed({
  provider,
  held,
  wiping,
  onWipe,
  onLetIn,
  onOff,
}: {
  provider: string;
  held: boolean;
  wiping: boolean;
  onWipe: () => void;
  onLetIn: () => void;
  onOff: () => void;
}) {
  return (
    <>
      <p className="text-[13px] font-semibold">{t("sealSealed")}</p>
      <p className="mt-0.5 text-[12.5px] leading-relaxed text-soft">
        {fill("sealOldCopy", provider)}
      </p>
      {wiping && (
        <p role="status" className="mt-2 text-[12.5px] text-soft motion-safe:animate-pulse">
          {fill("sealWiping", provider)}
        </p>
      )}
      <div className="mt-3 flex flex-wrap justify-end gap-2">
        <button type="button" className={mild} disabled={held || wiping} onClick={onOff}>
          {t("sealOff")}
        </button>
        <button type="button" className={mild} disabled={held || wiping} onClick={onLetIn}>
          {t("admitLetInButton")}
        </button>
        <button type="button" className={strong} disabled={held || wiping} onClick={onWipe}>
          {t("sealWipe")}
        </button>
      </div>
    </>
  );
}
