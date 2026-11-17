import { useMemo, useRef } from "react";
import type { Linkable } from "tisty-editor/edit";
import { folded, matched } from "tisty-editor/read";
import { docHeads, type Heading } from "../citing";
import type { Filed } from "../core";
import { docsAsked, named, underOf } from "../jumping";
import { latest } from "../lately";
import { docLink } from "../markdown";
import { shown } from "../tongues";

const LISTED = 8;
const HEADS_LAST = 30_000;

type Held = { wrote?: string | null; at: number; heads: Heading[] | null };

const linkOf = (doc: Filed, docs: Filed[]): Linkable => ({
  key: doc.file,
  label: named(doc),
  detail: underOf(doc, docs),
  link: docLink(doc.file, named(doc)),
});

const headOf = (doc: Filed, head: Heading): Linkable => ({
  key: `${doc.file}#${head.id}`,
  label: head.title,
  detail: named(doc),
  link: docLink(`${doc.file}#${head.id}`, `${named(doc)} › ${head.title}`),
});

// The editor asks on every letter, so the latest list is read through a ref and the answers stay one pair.
export const useLinkable = (known: Filed[], open: string | undefined) => {
  const now = useRef({ known, open });
  now.current = { known, open };
  const held = useRef(new Map<string, Held>());
  return useMemo(() => {
    const others = () =>
      shown(now.current.known).filter(
        (one) => !one.away && !one.gone && one.file !== now.current.open,
      );
    const linkable = (asked: string): Linkable[] => {
      const docs = others();
      const found = asked.trim()
        ? docsAsked(docs, asked, LISTED)
        : latest({ folders: [], docs }, LISTED);
      return found.map((doc) => linkOf(doc, docs));
    };
    const headsOf = (asked: string, head: string, ready: () => void): Linkable[] => {
      const docs = others();
      const doc =
        docs.find((one) => folded(named(one)) === folded(asked.trim())) ??
        docsAsked(docs, asked, 1)[0];
      if (!doc) return [];
      const had = held.current.get(doc.file);
      const fresh =
        had && had.wrote === doc.wrote && (!had.heads || Date.now() - had.at < HEADS_LAST);
      if (had && fresh) {
        return (had.heads ?? [])
          .filter((one) => matched(one.title, head))
          .map((one) => headOf(doc, one));
      }
      const keep = (heads: Heading[] | null) =>
        held.current.set(doc.file, { wrote: doc.wrote, at: Date.now(), heads });
      keep(null);
      docHeads(doc.file)
        .then((heads) => {
          keep(heads);
          ready();
        })
        .catch(() => keep([]));
      return [];
    };
    return { linkable, headsOf };
  }, []);
};
