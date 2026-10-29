import { dressed, shell } from "tisty-reader/page";
import bar from "./bar.css?raw";
import task from "./taskpage.css?raw";
import { darkNow } from "./theming";

const OWN_CSS = `${bar}${task}`;

export const framed = (html: string, dark = darkNow()): string => shell(html, dark, OWN_CSS);

export const dressedAsWindow = (page: Document) => dressed(page, darkNow());
