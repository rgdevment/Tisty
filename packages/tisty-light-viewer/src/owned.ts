import {
  type Attaching,
  attachedAs,
  ending,
  type Lender,
  type Lending,
  named,
  SEEABLE,
  widgetsDrawn,
} from "tisty-sheet";
import { fileUrl, keptHere, served, weighs } from "./host";
import type { Words } from "./words";

const attached = (src: string): boolean =>
  src !== "" && !/^[a-z][a-z0-9+.-]*:/i.test(src) && !SEEABLE.includes(ending(src));

export const drawnOwn = (
  page: Document,
  words: Words,
  theme: Pick<Lender, "dark" | "onTheme">,
): Lending[] => {
  const host: Attaching = {
    serve: served,
    url: fileUrl,
    keptHere,
    weighs,
    again: () => {},
    showing: () => () => {},
    plainly: () => words.say("failed"),
    say: (word) => words.sayAny(word),
    named,
  };
  for (const img of page.querySelectorAll<HTMLImageElement>(".page img[src]")) {
    const src = img.getAttribute("src") ?? "";
    if (attached(src)) attachedAs(page, img, src, () => true, [], host);
  }
  return widgetsDrawn(page, { say: (word, name) => words.sayAny(word, name), ...theme });
};
