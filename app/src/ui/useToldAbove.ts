import { useEffect, useState } from "react";
import { type Design, laidIn } from "tisty-editor/read";
import { docRead } from "../core";
import { namesIn } from "../paging";

type Above = { readonly of?: string; readonly told?: string[]; readonly design?: Design };

const NONE: Above = {};

// What a page takes from the document it belongs to: its place among its sisters, and the design.
export const useToldAbove = (book: string | undefined, fresh: number): Above => {
  const [above, setAbove] = useState(NONE);
  useEffect(() => {
    if (!book) return setAbove(NONE);
    let gone = false;
    docRead(book)
      .then((text) => {
        if (!gone) setAbove({ of: book, told: namesIn(text), design: laidIn(text).design });
      })
      .catch(() => {});
    return () => {
      gone = true;
    };
  }, [book, fresh]);
  return above.of === book ? above : NONE;
};
