import { BAR_CSS as bar } from "tisty-editor/edit";
import { dressed, shell } from "tisty-editor/read";
import task from "./taskpage.css?raw";
import { darkNow } from "./theming";

const OWN_CSS = `${bar}${task}`;

export const framed = (html: string): string => shell(html, darkNow(), OWN_CSS);

export const dressedAsWindow = (page: Document) => dressed(page, darkNow());
