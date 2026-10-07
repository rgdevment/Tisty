import { save as intoFile, open as pick } from "@tauri-apps/plugin-dialog";
import { useState } from "react";
import { type Afoot, docsPack, docsTakeOut, docsUnpack, spelled } from "../core";
import { fill, t } from "../locales";
import { saidPlainly } from "../refusal";
import { HOW_MANY } from "./Digits";
import ParcelAsks from "./ParcelAsks";

const PARCEL = "tistyx";

interface Hands {
  afoot: Afoot | null;
  setAfoot: (afoot: Afoot | null) => void;
  setError: (text: string) => void;
  noted: (text: string) => void;
  said: (text: string, since: number) => void;
  papersChanged: () => void;
}

export function useParcels({ afoot, setAfoot, setError, noted, said, papersChanged }: Hands) {
  const [whoFor, setWhoFor] = useState<string | null>(null);
  const [movingTo, setMovingTo] = useState<string | null>(null);
  const [locked, setLocked] = useState<string | null>(null);
  const [number, setNumber] = useState("");
  const [wrong, setWrong] = useState(false);

  // One at a time, and only everything at once may be a move, so only that one asks first.
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
            noted(
              many === 1
                ? t("tookOutOne")
                : took.folders
                  ? fill("tookOutAll", String(many), String(took.folders))
                  : fill("tookOutAllFlat", String(many)),
            );
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
        // Locked is the parcel asking whether this is its machine, which only the number answers.
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
    <ParcelAsks
      asks={{ whoFor, movingTo, locked, number, wrong }}
      onWho={setWhoFor}
      onMoving={setMovingTo}
      onLocked={setLocked}
      onNumber={setNumber}
      onWrong={setWrong}
      onShare={(named) => void packing([], named)}
      onLockAndPack={lockAndPack}
      onOpenLocked={openLocked}
    />
  );

  return {
    packUp,
    takeOutAll,
    takeParcel,
    asking: whoFor !== null || movingTo !== null || locked !== null,
    shown,
  };
}
