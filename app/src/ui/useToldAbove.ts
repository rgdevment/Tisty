import { useEffect, useState } from "react";
import { docRead } from "../core";
import { namesIn } from "../paging";

export const useToldAbove = (book: string | undefined, fresh: number): string[] | undefined => {
  const [told, setTold] = useState<string[]>();
  useEffect(() => {
    if (!book) return setTold(undefined);
    let gone = false;
    docRead(book)
      .then((text) => {
        if (!gone) setTold(namesIn(text));
      })
      .catch(() => {});
    return () => {
      gone = true;
    };
  }, [book, fresh]);
  return told;
};
