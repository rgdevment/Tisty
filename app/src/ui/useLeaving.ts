import { save as intoFile } from "@tauri-apps/plugin-dialog";
import { useEffect, useRef } from "react";
import { docWord, keepFile } from "../core";
import { saidPlainly } from "../refusal";
import { webPage } from "./webPage";

type Leaving = {
  readonly book: () => Promise<string[] | null>;
  readonly called: string;
  readonly onError: (said: string) => void;
};

const html = async ({ book, called }: Leaving) => {
  const bodies = await book();
  if (!bodies) return;
  const made = await webPage(bodies, called);
  const where = await intoFile({
    defaultPath: `${called}.html`,
    filters: [{ name: "HTML", extensions: ["html"] }],
  });
  if (where) await keepFile(where, Array.from(new TextEncoder().encode(made)));
};

const word = async ({ book, called }: Leaving) => {
  const bodies = await book();
  if (!bodies) return;
  const where = await intoFile({
    defaultPath: `${called}.docx`,
    filters: [{ name: "Word", extensions: ["docx"] }],
  });
  if (where) await docWord(bodies, where);
};

// The menu asks from outside the document, so the book is read as it stands when the event lands.
export const useLeaving = (leaving: Leaving) => {
  const now = useRef(leaving);
  now.current = leaving;
  useEffect(() => {
    const run = (made: (one: Leaving) => Promise<void>) => () =>
      made(now.current).catch((e) => now.current.onError(saidPlainly(e)));
    const toHtml = run(html);
    const toWord = run(word);
    window.addEventListener("tisty:to-html", toHtml);
    window.addEventListener("tisty:to-word", toWord);
    return () => {
      window.removeEventListener("tisty:to-html", toHtml);
      window.removeEventListener("tisty:to-word", toWord);
    };
  }, []);
};
