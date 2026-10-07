import { save as intoFile, open as pick } from "@tauri-apps/plugin-dialog";
import { useState } from "react";
import { type Afoot, docsPack, docsTakeOut, docsUnpack, spelled } from "../core";
import { SHOWN } from "../glance";
import { fill, t } from "../locales";
import { saidPlainly } from "../refusal";
import Digits, { HOW_MANY } from "./Digits";
import Modal from "./Modal";

const PARCEL = "tistyx";

interface Hands {
  afoot: Afoot | null;
  setAfoot: (afoot: Afoot | null) => void;
  setError: (text: string) => void;
  setNote: (text: string | null) => void;
  said: (text: string, since: number) => void;
  papersChanged: () => void;
}

export function useParcels({ afoot, setAfoot, setError, setNote, said, papersChanged }: Hands) {
  const [whoFor, setWhoFor] = useState<string | null>(null);
  const [movingTo, setMovingTo] = useState<string | null>(null);
  const [locked, setLocked] = useState<string | null>(null);
  const [number, setNumber] = useState("");
  const [wrong, setWrong] = useState(false);

  // One at a time: starting a second one paints over the first one's progress and the backend
  // refuses it anyway, leaving the bar gone and the first one's success unsaid.
  // Everything at once is the one that may be a move rather than a hand-over, so it asks first;
  // a single document is always somebody else's to keep.
  const packUp = (which: string[], named: string) => {
    if (afoot) return Promise.resolve();
    if (which.length) return packing(which, named);
    setWhoFor(named);
    return Promise.resolve();
  };

  const packing = (which: string[], named: string, number?: string) => {
    const since = Date.now();
    return spelled(named)
      .catch(() => "tisty")
      .then((safe) =>
        intoFile({
          defaultPath: `${safe}.${PARCEL}`,
          filters: [{ name: "Tisty", extensions: [PARCEL] }],
        }),
      )
      .then((at) => {
        if (typeof at !== "string") return null;
        setAfoot({ stage: "packing", far: 0, done: 0, whole: 0 });
        return docsPack(which, at, number);
      })
      .then((packed) => {
        setAfoot(null);
        if (!packed) return;
        const many = packed.docs + packed.pages;
        if (packed.missed > 0 || packed.left > 0) {
          setError(
            packed.missed > 0
              ? fill("packedShort", String(many), String(packed.missed))
              : fill("packedLess", String(many), String(packed.left)),
          );
          return;
        }
        said(
          many === 1 ? t("packedOne") : many ? fill("packed", String(many)) : t("packedAlone"),
          since,
        );
      })
      .catch((e) => {
        setAfoot(null);
        setError(saidPlainly(e));
      });
  };

  const takeOutAll = () =>
    afoot
      ? Promise.resolve()
      : pick({ directory: true })
          .then((at) => {
            if (typeof at !== "string") return null;
            setAfoot({ stage: "takingOut", far: 0, done: 0, whole: 0 });
            return docsTakeOut([], at);
          })
          .then((took) => {
            setAfoot(null);
            if (!took) return;
            const many = took.docs;
            if (took.missed > 0 || took.left > 0) {
              setError(
                took.missed > 0
                  ? fill("packedShort", String(many), String(took.missed))
                  : fill("packedLess", String(many), String(took.left)),
              );
              return;
            }
            setNote(
              many === 1
                ? t("tookOutOne")
                : took.folders
                  ? fill("tookOutAll", String(many), String(took.folders))
                  : fill("tookOutAllFlat", String(many)),
            );
            setTimeout(() => setNote(null), SHOWN);
          })
          .catch((e) => {
            setAfoot(null);
            setError(saidPlainly(e));
          });

  const takeParcel = () =>
    afoot
      ? Promise.resolve()
      : pick({ multiple: false, filters: [{ name: "Tisty", extensions: [PARCEL] }] }).then((at) =>
          typeof at === "string" ? landing(at) : undefined,
        );

  const landing = (at: string, number?: string) => {
    const since = Date.now();
    return Promise.resolve()
      .then(() => {
        setAfoot({ stage: "landing", far: 0, done: 0, whole: 0 });
        return docsUnpack(at, number);
      })
      .then((landed) => {
        setAfoot(null);
        if (!landed) return;
        papersChanged();
        const many = landed.docs + landed.pages;
        if (landed.missed > 0 && many > 0) {
          setError(fill("landedShort", String(landed.missed)));
          return;
        }
        if (many === 0) {
          setError(
            landed.missed > 0 ? fill("landedNoneOfIt", String(landed.missed)) : t("landedNone"),
          );
          return;
        }
        said(
          many === 1
            ? t("landedOne")
            : landed.folders
              ? fill("landedIn", String(many), String(landed.folders))
              : fill("landedAlone", String(many)),
          since,
        );
      })
      .catch((e) => {
        setAfoot(null);
        // Locked is not a failure: it is the parcel asking whether this is the machine it was
        // packed for, and only the number answers that.
        const why = (e as { code?: string } | undefined)?.code;
        if (why === "parcelLocked" || why === "wrongNumber") {
          setWrong(why === "wrongNumber");
          setNumber("");
          setLocked(at);
          return;
        }
        setError(saidPlainly(e));
      });
  };

  const lockAndPack = () => {
    if (movingTo === null || number.length < HOW_MANY) return;
    const named = movingTo;
    const said = number;
    setMovingTo(null);
    setNumber("");
    void packing([], named, said);
  };

  const openLocked = () => {
    if (locked === null || number.length < HOW_MANY) return;
    const at = locked;
    const said = number;
    setLocked(null);
    setNumber("");
    void landing(at, said);
  };

  const shown = (
    <>
      {whoFor !== null && (
        <Modal title={t("packWho")} onClose={() => setWhoFor(null)}>
          <p className="mt-3 text-[12.5px] leading-relaxed text-soft">{t("packWhoWhy")}</p>
          <div className="mt-5 flex flex-wrap items-center justify-end gap-2 text-[12.5px]">
            <button
              type="button"
              onClick={() => setWhoFor(null)}
              className="cursor-pointer rounded-[10px] px-3 py-1.5 text-faint hover:text-ink"
            >
              {t("cancel")}
            </button>
            <button
              type="button"
              onClick={() => {
                const named = whoFor;
                setWhoFor(null);
                void packing([], named);
              }}
              className="cursor-pointer rounded-[10px] border border-line px-3 py-1.5 text-ink hover:bg-line/40"
            >
              {t("packToShare")}
            </button>
            <button
              type="button"
              onClick={() => {
                setNumber("");
                setMovingTo(whoFor);
                setWhoFor(null);
              }}
              className="cursor-pointer rounded-[10px] bg-accent px-3.5 py-1.5 text-bg"
            >
              {t("packToMove")}
            </button>
          </div>
        </Modal>
      )}

      {movingTo !== null && (
        <Modal
          title={t("packToMove")}
          onClose={() => {
            setMovingTo(null);
            setNumber("");
          }}
        >
          <p className="mt-3 text-[12.5px] text-soft">{t("packNumber")}</p>
          <Digits
            label={t("packNumber")}
            value={number}
            onChange={setNumber}
            onDone={lockAndPack}
          />
          <p className="mt-3 text-[11.5px] leading-relaxed text-faint">{t("packNumberWhy")}</p>
          <div className="mt-5 flex items-center justify-end gap-2 text-[12.5px]">
            <button
              type="button"
              onClick={() => {
                setMovingTo(null);
                setNumber("");
              }}
              className="cursor-pointer rounded-[10px] px-3 py-1.5 text-faint hover:text-ink"
            >
              {t("cancel")}
            </button>
            <button
              type="button"
              disabled={number.length < HOW_MANY}
              onClick={lockAndPack}
              className="cursor-pointer rounded-[10px] bg-accent px-3.5 py-1.5 text-bg disabled:opacity-60"
            >
              {t("packLockIt")}
            </button>
          </div>
        </Modal>
      )}

      {locked !== null && (
        <Modal
          title={t("parcelShut")}
          onClose={() => {
            setLocked(null);
            setNumber("");
          }}
        >
          <p className="mt-3 text-[12.5px] leading-relaxed text-soft">{t("parcelLocked")}</p>
          <p className="mt-4 text-[12.5px] text-soft">{t("openNumber")}</p>
          <Digits
            label={t("openNumber")}
            value={number}
            onChange={(said) => {
              setWrong(false);
              setNumber(said);
            }}
            onDone={openLocked}
          />
          {wrong && (
            <p role="alert" className="mt-3 text-[11.5px] text-urgent">
              {t("wrongNumber")}
            </p>
          )}
          <div className="mt-5 flex items-center justify-end gap-2 text-[12.5px]">
            <button
              type="button"
              onClick={() => {
                setLocked(null);
                setNumber("");
              }}
              className="cursor-pointer rounded-[10px] px-3 py-1.5 text-faint hover:text-ink"
            >
              {t("cancel")}
            </button>
            <button
              type="button"
              disabled={number.length < HOW_MANY}
              onClick={openLocked}
              className="cursor-pointer rounded-[10px] bg-accent px-3.5 py-1.5 text-bg disabled:opacity-60"
            >
              {t("openLocked")}
            </button>
          </div>
        </Modal>
      )}
    </>
  );

  return {
    packUp,
    takeOutAll,
    takeParcel,
    landing,
    asking: whoFor !== null || movingTo !== null || locked !== null,
    shown,
  };
}
