import "tisty-reader/tokens.css";
import "./web.css";
import init, { open } from "../wasm/tisty_share_wasm.js";
import wasm from "../wasm/tisty_share_wasm_bg.wasm?url";
import { drawn, type Opened, titleOf } from "./drawn";
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

const opened = async (): Promise<Found | string> => {
  const link = linkOf(location.pathname, location.hash);
  if (!link) return "nolink";
  if (!link.key) return "keyless";
  const [answer] = await Promise.all([fetched(link.id), init({ module_or_path: wasm })]);
  if ("refused" in answer) return answer.refused;
  try {
    const page = open(answer.sealed, link.key, answer.limits);
    const about = JSON.parse(page.about) as About;
    const pictures = Array.from({ length: page.pictures }, (_, at) => ({
      bytes: new Uint8Array(page.picture(at) ?? []),
      media: page.media(at) ?? "",
    }));
    const shown = { html: page.html, pictures, title: about.title ?? "" };
    page.free();
    return shown;
  } catch (error_) {
    return error_ instanceof Error ? error_.message : "failed";
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
    void draw();
  });
};

// Another key pasted over this one is another link, so it is read anew.
addEventListener("hashchange", () => location.reload());

await shown();
