import { useEffect, useState } from "react";
import type { Doc } from "../core";
import { docBackTo, type Earlier as Version, whenOf } from "../earlier";
import { settled } from "../saving";
import Earlier from "./Earlier";
import type { CardsShown } from "./liveCards";

interface Asked {
  file?: string;
  title: string;
  page?: boolean;
  writable: boolean;
  cardsShown?: CardsShown;
  /** What is typed reaches the disk first, so it is what going back keeps to come back to; refused, nothing goes back. */
  before: () => Promise<unknown>;
  onWentBack: (said: Doc, when: string) => void | Promise<void>;
  onEmptied?: () => void;
  onError: (problem: unknown) => void;
}

// The words typed are the same, and what they kept from happening is a comparison, not a step back.
const uncompared = (problem: unknown) =>
  (problem as { code?: string } | null)?.code === "unsavedStays"
    ? { code: "unsavedUncompared" }
    : problem;

export const useEarlier = (asked: Asked) => {
  const { file } = asked;
  const [shown, setShown] = useState<string | null>(null);
  // Another document's history is never left open over this one.
  useEffect(() => setShown(null), [file]);

  const back = async (one: Version) => {
    if (!file) return;
    await asked.before();
    await settled();
    const said = await docBackTo(file, one.print);
    setShown(null);
    await asked.onWentBack(said, whenOf(one));
  };

  return {
    show: () => setShown(file ?? null),
    view: file && shown === file && (
      <Earlier
        doc={file}
        title={asked.title}
        page={asked.page}
        cardsShown={asked.cardsShown}
        onBack={asked.writable ? back : undefined}
        ready={async () => {
          await asked.before().catch((e) => Promise.reject(uncompared(e)));
          await settled();
        }}
        onEmptied={asked.onEmptied}
        onClose={() => setShown(null)}
        onError={asked.onError}
      />
    ),
  };
};
