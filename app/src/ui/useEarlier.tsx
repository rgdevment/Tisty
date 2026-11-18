import { useEffect, useState } from "react";
import type { Carded } from "tisty-editor/read";
import type { Doc } from "../core";
import { docBackTo, type Earlier as Version, whenOf } from "../earlier";
import { settled } from "../saving";
import Earlier from "./Earlier";

interface Asked {
  file?: string;
  title: string;
  page?: boolean;
  writable: boolean;
  card?: (file: string) => Carded;
  /** What is typed reaches the disk first, so it is what going back keeps to come back to. */
  before: () => void;
  onWentBack: (said: Doc, when: string) => void | Promise<void>;
  onEmptied?: () => void;
  onError: (problem: unknown) => void;
}

export const useEarlier = (asked: Asked) => {
  const { file } = asked;
  const [shown, setShown] = useState<string | null>(null);
  // Another document's history is never left open over this one.
  useEffect(() => setShown(null), [file]);

  const back = async (one: Version) => {
    if (!file) return;
    asked.before();
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
        card={asked.card}
        onBack={asked.writable ? back : undefined}
        onEmptied={asked.onEmptied}
        onClose={() => setShown(null)}
        onError={asked.onError}
      />
    ),
  };
};
