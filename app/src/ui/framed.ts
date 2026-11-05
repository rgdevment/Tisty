import { dressed, shell } from "tisty-editor/read";
import task from "./taskpage.css?raw";
import { darkNow } from "./theming";

export const framed = (html: string, bar = ""): string => shell(html, darkNow(), `${bar}${task}`);

export const dressedAsWindow = (page: Document) => dressed(page, darkNow());
