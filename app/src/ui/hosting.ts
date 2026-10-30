import { edited as editing, type Hosted } from "tisty-editor/edit";
import { docFromHtml, docFromText } from "../core";
import { t } from "../locales";
import { refused, tableEditNear } from "../tables";

// The window holds the editor: its words are Tisty's, and its tables and pasted text go to the core.
export const HOST: Hosted = {
  say: t,
  tableEditNear: (body, at, ...edits) =>
    tableEditNear(body, at, ...edits).catch((why) => {
      throw refused(why);
    }),
  fromHtml: docFromHtml,
  fromText: docFromText,
};

type Rest<T> = T extends (host: Hosted, ...rest: infer R) => unknown ? R : never;

export const edited = (...rest: Rest<typeof editing>) => editing(HOST, ...rest);
