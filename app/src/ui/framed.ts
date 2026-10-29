import { shell } from "tisty-reader/page";
import bar from "./bar.css?raw";
import task from "./taskpage.css?raw";
import { darkNow } from "./theming";

export const framed = (html: string): string => shell(html, darkNow(), `${bar}${task}`);
