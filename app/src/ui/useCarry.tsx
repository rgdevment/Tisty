import { useEffect, useState } from "react";
import { stillApart, walkThrough } from "../apart";
import { choose } from "../coming";
import { docs, type Kin, syncKin, syncNow, whatWaits, whatWentAmiss } from "../core";
import { decideAll, putOffSaid } from "../deciding";
import { fill, t } from "../locales";
import { saidPlainly } from "../refusal";
import { lastingCause, syncSaid, waitSaid, whatsLeft } from "../syncSaid";
import Apart, { type Door } from "./Apart";
import type { Which, Word } from "./Card";

interface Hands {
  held: boolean;
  cloud?: string;
  setBusy: (which: Which | null) => void;
  tell: (word?: Word) => void;
  fail: (word?: Word) => void;
  look: () => void;
  onChanged: () => void;
}

export function useCarry({ held, cloud, setBusy, tell, fail, look, onChanged }: Hands) {
  const [apart, setApart] = useState<((door: Door | "else" | null) => void) | null>(null);
  const [kin, setKin] = useState<Kin>("unsure");

  useEffect(() => {
    choose(apart !== null);
    return () => choose(false);
  }, [apart]);

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
      const { said, putOff } = await decideAll(answer.undecided, true);
      const amiss = whatWentAmiss(answer);
      const waits = whatWaits(answer);
      const cause = answer.deferred?.reason === "offline" ? lastingCause(answer.deferred) : null;
      if (amiss) {
        fail({ card: "sync", text: t(amiss) });
      } else if (answer.refused) {
        fail({
          card: "sync",
          text: saidPlainly({ code: answer.refused.code, name: answer.refused.name ?? undefined }),
        });
      } else if (said) {
        fail({ card: "sync", text: said });
      } else if (putOff.length) {
        tell({ card: "sync", text: putOffSaid(putOff.length) });
      } else if (answer.adopted) {
        tell({ card: "sync", text: fill("syncAdopted", cloud ?? t("syncTheFolder")) });
      } else if (answer.joined?.length) {
        tell({ card: "sync", text: fill("someJoined", await namedDocs(answer.joined)) });
      } else if (cause) {
        fail({ card: "sync", text: cause });
      } else if (waits) {
        tell({
          card: "sync",
          text: [waitSaid(waits), whatsLeft(answer)].filter(Boolean).join(" "),
        });
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
          cloud={cloud}
          onPick={(door) => closed(door)}
          onElse={() => closed("else")}
          onClose={() => closed(null)}
        />
      )}
    </>
  );

  return { carryNow, shown };
}
