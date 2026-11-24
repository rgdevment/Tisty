import { save as intoFile } from "@tauri-apps/plugin-dialog";
import { useEffect, useRef } from "react";
import type { Design } from "tisty-editor/read";
import { docWord, keepFile } from "../core";
import { saidPlainly } from "../refusal";
import { webPage } from "./webPage";

type Leaving = {
  readonly book: () => Promise<string[] | null>;
  readonly called: string;
  readonly design?: Design;
  readonly onError: (said: string) => void;
};

const leaveAsHtml = async (bodies: string[], called: string, design?: Design) => {
  const made = await webPage(bodies, called, undefined, design);
  const where = await intoFile({
    defaultPath: `${called}.html`,
    filters: [{ name: "HTML", extensions: ["html"] }],
  });
  if (where) await keepFile(where, Array.from(new TextEncoder().encode(made)));
};

const leaveAsWord = async (bodies: string[], called: string) => {
  const where = await intoFile({
    defaultPath: `${called}.docx`,
    filters: [{ name: "Word", extensions: ["docx"] }],
  });
  if (where) await docWord(bodies, where);
};

const read =
  (made: typeof leaveAsHtml) =>
  async ({ book, called, design }: Leaving) => {
    const bodies = await book();
    if (bodies) await made(bodies, called, design);
  };

// The menu asks from outside the document, so the book is read as it stands when the event lands.
export const useLeaving = (leaving: Leaving) => {
  const now = useRef(leaving);
  now.current = leaving;
  useEffect(() => {
    const run = (made: (one: Leaving) => Promise<void>) => () =>
      made(now.current).catch((e) => now.current.onError(saidPlainly(e)));
    const toHtml = run(read(leaveAsHtml));
    const toWord = run(read(leaveAsWord));
    window.addEventListener("tisty:to-html", toHtml);
    window.addEventListener("tisty:to-word", toWord);
    return () => {
      window.removeEventListener("tisty:to-html", toHtml);
      window.removeEventListener("tisty:to-word", toWord);
    };
  }, []);
};
