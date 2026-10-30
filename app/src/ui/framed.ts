import { dressed, shell } from "tisty-editor/read";
import bar from "./bar.css?raw";
import task from "./taskpage.css?raw";
import { darkNow } from "./theming";

const OWN_CSS = `${bar}${task}`;

export const framed = (html: string): string => shell(html, darkNow(), OWN_CSS);

export const dressedAsWindow = (page: Document) => dressed(page, darkNow());
