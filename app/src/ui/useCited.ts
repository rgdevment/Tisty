import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import type { Citer, Panel } from "tisty-editor/read";
import { type Citing, docCitedBy } from "../citing";
import { fill, t } from "../locales";

const CLOSED = new Set(["done", "dropped", "archived"]);

const glyphOf = (one: Citing): Citer["glyph"] => {
  if (one.kind === "task") return CLOSED.has(one.state ?? "") ? "done" : "todo";
  if (one.state === "archived") return "archive";
  return one.kind === "page" ? "alignleft" : "page";
};

const citerOf = (one: Citing, onPick: (one: Citing) => void): Citer => ({
  key: one.id,
  title: one.title.trim() || t("untitledDoc"),
  glyph: glyphOf(one),
  under: one.under?.length
    ? fill("citedUnder", one.under.map((head) => head.title).join(" · "))
    : undefined,
  away: CLOSED.has(one.state ?? ""),
  onPick: () => onPick(one),
});

// `undefined` while asked, `null` when the base has not been read yet, so neither says «nobody».
export const useCited = (paper: string): Citing[] | null | undefined => {
  const [held, setHeld] = useState<{ paper: string; citing: Citing[] | null } | null>(null);
  useEffect(() => {
    let gone = false;
    // A round moves the base on any write anywhere; the panel is drawn again only when its answer moved.
    let said: string | null = null;
    const ask = () =>
      docCitedBy(paper)
        .then((citing) => {
          const now = JSON.stringify(citing);
          if (gone || now === said) return;
          said = now;
          setHeld({ paper, citing });
        })
        .catch(() => {});
    void ask();
    const heard = listen("brain", () => void ask());
    return () => {
      gone = true;
      void heard.then((stop) => stop()).catch(() => {});
    };
  }, [paper]);
  return held?.paper === paper ? held.citing : undefined;
};

export const citedPanel = (
  citing: Citing[] | null | undefined,
  onPick: (one: Citing) => void,
): Panel[] =>
  citing === undefined
    ? []
    : [
        {
          kind: "cited",
          label: t("citedBy"),
          gist: citing?.length ? `${citing.length}` : "",
          none: t(citing ? "citedNobody" : "citedNotReady"),
          citers: (citing ?? []).map((one) => citerOf(one, onPick)),
          open: Boolean(citing?.length),
        },
      ];
