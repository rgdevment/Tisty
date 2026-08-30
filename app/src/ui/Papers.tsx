import { useEffect, useState } from "react";
import { docs, type Filed } from "../core";
import { matched } from "../finding";
import { t } from "../locales";
import Row from "./Row";

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
      .then((papers) => setFound(papers.docs.filter((one) => !one.archived)))
      .catch((problem) => {
        setFound([]);
        onError?.(problem);
      });
  }, [given, onError]);

  const all = given ? given.filter((one) => !one.archived) : found;

  const named = (doc: Filed) => doc.title.trim() || t("untitledDoc");
  const shown = (all ?? []).filter((one) => matched(named(one), word));

  return (
    <>
      <input
        autoFocus
        value={word}
        aria-label={t("pickADocToLink")}
        placeholder={t("pickADocToLink")}
        onChange={(e) => setWord(e.target.value)}
        className="mb-1 w-full rounded-md bg-hover px-2.5 py-1.5 outline-none placeholder:text-faint"
      />
      <div className="scroller max-h-[168px]">
        {all === null && <p className="px-2.5 py-1.5 text-faint">{t("opening")}</p>}
        {all !== null && shown.length === 0 && (
          <p className="px-2.5 py-1.5 text-faint">{all.length ? t("noneHere") : t("noDocsYet")}</p>
        )}
        {shown.map((doc) => (
          <Row key={doc.id} glyph="▤" onPick={() => onPick(doc)}>
            <span className="min-w-0 truncate">{named(doc)}</span>
          </Row>
        ))}
      </div>
    </>
  );
}
