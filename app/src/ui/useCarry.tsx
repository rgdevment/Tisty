import { useState } from "react";
import { stillApart, walkThrough } from "../apart";
import { docs, type Kin, syncKin, syncNow, whatWentAmiss } from "../core";
import { decideAll } from "../deciding";
import { fill, t } from "../locales";
import { saidPlainly } from "../refusal";
import { syncSaid } from "../syncSaid";
import Apart, { type Door } from "./Apart";
import type { Which, Word } from "./Card";

interface Hands {
  held: boolean;
  setBusy: (which: Which | null) => void;
  tell: (word?: Word) => void;
  fail: (word?: Word) => void;
  look: () => void;
  onChanged: () => void;
}

export function useCarry({ held, setBusy, tell, fail, look, onChanged }: Hands) {
  const [apart, setApart] = useState<((door: Door | "else" | null) => void) | null>(null);
  const [kin, setKin] = useState<Kin>("unsure");

  const closed = (door: Door | "else" | null) => {
    apart?.(door);
    setApart(null);
  };

  const namedDocs = async (files: string[]): Promise<string> => {
    const titled = await docs()
      .then((found) => new Map(found.docs.map((one) => [one.file, one.title])))
      .catch(() => new Map<string, string>());
    return files.map((one) => `«${titled.get(one)?.trim() || t("untitledDoc")}»`).join(", ");
  };

  const carryNow = async (way?: "again"): Promise<"done" | "declined" | "failed"> => {
    if (held) return "failed";
    setBusy("sync");
    tell(undefined);
    fail(undefined);
    try {
      const answer = await syncNow(way).catch(async (problem) => {
        if (!stillApart(problem)) throw problem;
        setKin(await syncKin().catch(() => "unsure" as const));
        const door = await new Promise<Door | "else" | null>((settle) => setApart(() => settle));
        if (door === null) return "declined" as const;
        if (!(await walkThrough(door))) return "declined" as const;
        return syncNow();
      });

      if (answer === "declined") {
        fail({ card: "sync", text: t("wouldReset") });
        return "declined";
      }
      const { shut, said } = await decideAll(answer.undecided);
      const amiss = whatWentAmiss(answer);
      if (amiss) {
        fail({ card: "sync", text: t(amiss) });
      } else if (shut.length) {
        fail({ card: "sync", text: fill("someLockedAtOdds", await namedDocs(shut)) });
      } else if (said) {
        fail({ card: "sync", text: said });
      } else if (answer.joined?.length) {
        tell({ card: "sync", text: fill("someJoined", await namedDocs(answer.joined)) });
      } else {
        tell({ card: "sync", text: syncSaid(answer) });
      }
      look();
      onChanged();
      return "done";
    } catch (e) {
      fail({ card: "sync", text: saidPlainly(e) });
      return "failed";
    } finally {
      setBusy(null);
    }
  };

  const shown = (
    <>
      {apart && (
        <Apart
          kin={kin}
          onPick={(door) => closed(door)}
          onElse={() => closed("else")}
          onClose={() => closed(null)}
        />
      )}
    </>
  );

  return { carryNow, shown };
}
