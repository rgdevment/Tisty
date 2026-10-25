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
  sealingRoom,
  sealingShown,
  sealingWipe,
} from "../core";
import { weigh } from "../format";
import { cloudNamed, fill, t } from "../locales";
import { plainly, saidPlainly } from "../refusal";
import { sure } from "../sure";
import Card, { type Which, type Word } from "./Card";
import Modal from "./Modal";
import Phrasing from "./Phrasing";
import { mild, risky, strong } from "./Rows";

interface Props {
  busy: Which | null;
  said?: Word;
  trouble?: Word;
  tell: (word?: Word) => void;
  fail: (word?: Word) => void;
  onChanged: () => void;
}

type Phrase = { words: string[]; fingerprint: string };

const card = "sealing" as const;

export default function Sealing({ busy, said, trouble, tell, fail, onChanged }: Props) {
  const [shown, setShown] = useState<SealShown | null>(null);
  const [asking, setAsking] = useState(false);
  const [phrase, setPhrase] = useState<Phrase | null>(null);
  const failed = useCallback((e: unknown) => fail({ card, text: saidPlainly(e) }), [fail]);

  const look = useCallback(() => {
    sealingShown().then(setShown).catch(failed);
  }, [failed]);
  useEffect(look, [look]);
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

  const begin = (on: boolean) => {
    setAsking(false);
    sealingBegin(on).then(look).catch(failed);
  };

  if (!shown) return null;
  const provider = cloudNamed(shown.cloud);
  const held = busy !== null;

  return (
    <Card title={t("sealTitle")} which={card} busy={busy} said={said} trouble={trouble}>
      {phrase && (
        <Phrasing
          words={phrase.words}
          fingerprint={phrase.fingerprint}
          onClose={() => {
            setPhrase(null);
            look();
          }}
          onConfirmed={() => {
            setPhrase(null);
            look();
          }}
        />
      )}
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
      {shown.state === "none" && (
        <p className="text-[12.5px] leading-relaxed text-soft">{t("sealNoCloud")}</p>
      )}
      {shown.state === "plain" && (
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
          onCancel={() =>
            sure(t("sealCancelSure"))
              .then((yes) => (yes ? sealingCancel().then(look) : undefined))
              .catch(failed)
          }
        />
      )}
      {shown.state === "sealed" && (
        <Sealed
          provider={provider}
          held={held}
          onWipe={() =>
            sure(fill("sealWipeSure", provider))
              .then((yes) =>
                yes
                  ? sealingWipe().then((gone) =>
                      tell({ card, text: fill("sealWiped", String(gone)) }),
                    )
                  : undefined,
              )
              .catch(failed)
          }
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
  const [room, setRoom] = useState<SealRoom | null>(null);
  useEffect(() => {
    sealingRoom()
      .then(setRoom)
      .catch(() => setRoom({ free: null, needed: 0 }));
  }, []);
  const roomy = room !== null && (room.free === null || room.free >= room.needed);
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
              : t(shown.held ? "sealPhraseUnconfirmed" : "sealPhraseMissing")}
          </span>
          {!phraseOk && (
            <button type="button" className={mild} onClick={shown.held ? onConfirm : onCreate}>
              {t(shown.held ? "sealPhraseConfirm" : "sealPhraseCreate")}
            </button>
          )}
        </li>
        <li className="flex gap-2">
          {room === null ? (
            <span className="text-faint">{fill("sealRoomLooking", provider)}</span>
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

function Progress({ told, onCancel }: { told: SealTold; onCancel: () => void }) {
  const at =
    told.step === "claimed"
      ? 0
      : told.step === "closed"
        ? 1
        : STEPS.indexOf(told.step as (typeof STEPS)[number]);
  const step = (index: number, label: string, detail?: string) => (
    <li className="flex gap-2">
      <span
        aria-hidden="true"
        className={`w-4 shrink-0 ${index < at ? "text-hue-teal" : "text-faint"}`}
      >
        {index < at ? "✓" : index === at ? "…" : "·"}
      </span>
      <span className="min-w-0 flex-1">
        <span className="block">{label}</span>
        {detail && <span className="block text-soft">{detail}</span>}
      </span>
    </li>
  );
  return (
    <>
      <p className="text-[13px] font-semibold">{t(told.on ? "sealTurningOn" : "sealTurningOff")}</p>
      <p className="mt-0.5 text-[12.5px] leading-relaxed text-soft">{t("sealKeepWorking")}</p>
      <ul className="mt-3 flex flex-col gap-2 text-[12.5px]">
        {step(
          0,
          t("sealStepCopy"),
          told.step === "copying"
            ? fill("sealStepCopied", String(told.done), String(told.left))
            : undefined,
        )}
        {step(1, t("sealStepClose"), t("sealStepCloseWhy"))}
        {step(2, t(told.on ? "sealStepSwitchOn" : "sealStepSwitchOff"))}
      </ul>
      {told.refused > 0 && (
        <p className="mt-2 text-[12.5px] text-soft">{fill("sealRefused", String(told.refused))}</p>
      )}
      {told.step === "stopped" && told.why && (
        <p role="alert" className="mt-2 text-[12.5px] text-urgent">
          {plainly({ code: told.why })}
        </p>
      )}
      <p className="mt-3 text-[11.5px] text-faint">{t("sealIntact")}</p>
      <div className="mt-3 flex justify-end">
        <button type="button" className={mild} onClick={onCancel}>
          {t("cancel")}
        </button>
      </div>
    </>
  );
}

function Sealed({
  provider,
  held,
  onWipe,
  onOff,
}: {
  provider: string;
  held: boolean;
  onWipe: () => void;
  onOff: () => void;
}) {
  return (
    <>
      <p className="text-[13px] font-semibold">{t("sealSealed")}</p>
      <p className="mt-0.5 text-[12.5px] leading-relaxed text-soft">
        {fill("sealOldCopy", provider)}
      </p>
      <div className="mt-3 flex flex-wrap justify-end gap-2">
        <button type="button" className={mild} disabled={held} onClick={onOff}>
          {t("sealOff")}
        </button>
        <button type="button" className={strong} disabled={held} onClick={onWipe}>
          {t("sealWipe")}
        </button>
      </div>
    </>
  );
}
