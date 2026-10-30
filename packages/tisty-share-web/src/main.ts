import "tisty-reader/tokens.css";
import "./web.css";
import init, { open, type Page } from "../wasm/tisty_share_wasm.js";
import wasm from "../wasm/tisty_share_wasm_bg.wasm?url";
import { drawn, type Opened, type Picture, plain, titleOf } from "./drawn";
import { fetched } from "./fetched";
import { linkOf } from "./link";
import { spanish, wordsFor } from "./words";

type About = { title?: string | null };
// What the link opened, with the title its front matter gives, which wins over the first line as in Tisty.
type Found = Opened & { readonly title: string };

const night = matchMedia("(prefers-color-scheme: dark)");
const inSpanish = spanish(navigator.languages);
const say = wordsFor(inSpanish);
const told = document.querySelector<HTMLElement>(".said") as HTMLElement;
const frame = document.querySelector<HTMLIFrameElement>("iframe.sheet") as HTMLIFrameElement;

const themed = () => {
  document.documentElement.dataset.theme = night.matches ? "dark" : "light";
};

const tell = (word: string) => {
  told.textContent = say(word);
  told.hidden = false;
  frame.hidden = true;
};

const wordOf = (refused: unknown): string =>
  refused instanceof Error ? refused.message : "failed";

// The page holds the words and pictures in the clear, so it is let go on every way out.
const taken = (page: Page): Found => {
  try {
    const about = JSON.parse(page.about) as About;
    const pictures: Picture[] = Array.from({ length: page.pictures }, (_, at) => ({
      // wasm-bindgen hands each picture over in a buffer of its own, never a shared one.
      bytes: (page.picture(at) ?? new Uint8Array()) as Uint8Array<ArrayBuffer>,
      media: page.media(at) ?? "",
    }));
    return { html: page.html, pictures, title: plain(about.title ?? "") };
  } finally {
    page.free();
  }
};

const opened = async (): Promise<Found | string> => {
  const link = linkOf(location.pathname, location.hash);
  if (!link) return "nolink";
  if (!link.key) return "keyless";
  const [answer] = await Promise.all([fetched(link.id), init({ module_or_path: wasm })]);
  if ("refused" in answer) return answer.refused;
  try {
    return taken(open(answer.sealed, link.key, answer.limits));
  } catch (refused) {
    return wordOf(refused);
  }
};

const shown = async () => {
  document.documentElement.lang = inSpanish ? "es" : "en";
  themed();
  tell("opening");
  const found = await opened();
  if (typeof found === "string") return tell(found);
  told.hidden = true;
  frame.hidden = false;
  const draw = () => drawn(frame, found, night.matches, say);
  const page = await draw();
  const title = found.title || titleOf(page);
  if (title) {
    document.title = `${title} · Tisty`;
    frame.title = title;
  }
  night.addEventListener("change", () => {
    themed();
    draw().catch(() => tell("failed"));
  });
};

// Another key pasted over this one is another link, so it is read anew.
addEventListener("hashchange", () => location.reload());

// A body that would not draw says so, rather than leaving half a sheet without a word.
await shown().catch(() => tell("failed"));
