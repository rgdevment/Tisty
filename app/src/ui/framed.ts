import { type Design, dressed, shell } from "tisty-editor/read";
import { LIVELY_CSS } from "tisty-sheet/lively";
import task from "./taskpage.css?raw";
import { darkNow } from "./theming";

export const framed = (html: string, bar = ""): string =>
  shell(html, darkNow(), `${bar}${task}${LIVELY_CSS}`);

export const dressedAsWindow = (page: Document, design?: Design) =>
  dressed(page, darkNow(), design);
