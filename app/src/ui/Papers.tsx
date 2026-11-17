import { useEffect, useId, useState } from "react";
import { matched } from "tisty-editor/read";
import { useAttended } from "../attended";
import { docs, type Filed } from "../core";
import { named, underOf } from "../jumping";
import { t } from "../locales";
import { optionOf, PickList, usePicker } from "./Picker";

const leftOpen = () => {};

export default function Papers({
  all: given,
  onPick,
  onError,
}: {
  all?: Filed[];
  onPick: (doc: Filed) => void;
  onError?: (problem: unknown) => void;
}) {
  const [found, setFound] = useState<Filed[] | null>(null);
  const [word, setWord] = useState("");

  useEffect(() => {
    if (given) return;
    docs()
      .then((papers) => setFound(papers.docs.filter((one) => !one.away)))
      .catch((problem) => {
        setFound([]);
        onError?.(problem);
      });
  }, [given, onError]);

  const all = given ? given.filter((one) => !one.away) : found;
  const rows = (all ?? [])
    .filter((one) => matched(named(one), word))
    .map((doc) => ({
      key: doc.id,
      glyph: "▤",
      label: named(doc),
      detail: underOf(doc, all ?? []),
      onPick: () => onPick(doc),
    }));
  // Esc belongs to the box around the list, which puts it away.
  const { on, setAt, keyed } = usePicker(rows, word, leftOpen);
  const list = useId();
  const held = useAttended<HTMLInputElement>();

  return (
    <>
      <input
        ref={held}
        value={word}
        role="combobox"
        aria-expanded="true"
        aria-controls={list}
        aria-activedescendant={optionOf(list, on)}
        aria-label={t("pickADocToLink")}
        placeholder={t("pickADocToLink")}
        onChange={(e) => setWord(e.target.value)}
        onKeyDown={(e) => {
          if (e.key !== "Escape" && !e.nativeEvent.isComposing) keyed(e);
        }}
        className="mb-1 w-full rounded-md bg-hover px-2.5 py-1.5 outline-none placeholder:text-faint"
      />
      <div className="scroller max-h-[168px]">
        {all === null && <p className="px-2.5 py-1.5 text-faint">{t("opening")}</p>}
        {all !== null && rows.length === 0 && (
          <p className="px-2.5 py-1.5 text-faint">{all.length ? t("noneHere") : t("noDocsYet")}</p>
        )}
        {rows.length > 0 && (
          <PickList id={list} label={t("foundPapers")} rows={rows} on={on} onHover={setAt} />
        )}
      </div>
    </>
  );
}
