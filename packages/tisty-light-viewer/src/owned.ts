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
import { bringNearPictures } from "./pictures";
import { filled, type Words } from "./words";

const EN: Record<string, string> = {
  mediaBring: "Bring it from your cloud",
  mediaBringing: "Bringing it from your cloud…",
  widgetFrame: "Widget drawn from this block",
  widgetTooBig: "A widget can hold up to {name}; this one is bigger, so it is shown as code.",
  widgetNot:
    "This block is not a widget, so it is shown as code: {name}. A widget is only HTML and CSS, with nothing that runs and nothing from elsewhere.",
  kindPdf: "PDF",
  kindWord: "Word",
  kindExcel: "Spreadsheet",
  kindSheet: "Data sheet",
  kindSlides: "Slides",
  kindArchive: "Compressed archive",
  kindData: "Data",
  kindImage: "Image",
  kindPage: "Web page",
  kindVideo: "Video",
  kindAudio: "Audio",
  kindMarkdown: "Markdown",
  kindText: "Text",
};

const ES: Record<string, string> = {
  mediaBring: "Traer de tu nube",
  mediaBringing: "Trayendo de tu nube…",
  widgetFrame: "Widget dibujado a partir de este bloque",
  widgetTooBig:
    "Un widget puede tener hasta {name}; este pasa de ahí, así que se muestra como código.",
  widgetNot:
    "Este bloque no es un widget y por eso se muestra como código: {name}. Un widget es solo HTML y CSS, sin nada que se ejecute ni venga de afuera.",
  kindPdf: "PDF",
  kindWord: "Word",
  kindExcel: "Hoja de cálculo",
  kindSheet: "Hoja de datos",
  kindSlides: "Diapositivas",
  kindArchive: "Archivo comprimido",
  kindData: "Datos",
  kindImage: "Imagen",
  kindPage: "Página web",
  kindVideo: "Vídeo",
  kindAudio: "Audio",
  kindMarkdown: "Markdown",
  kindText: "Texto",
};

const pictured = (src: string): boolean => SEEABLE.includes(ending(src));

const attached = (src: string): boolean =>
  src !== "" && !/^[a-z][a-z0-9+.-]*:/i.test(src) && !pictured(src);

export const drawnOwn = (
  page: Document,
  words: Words,
  theme: Pick<Lender, "dark" | "onTheme">,
): Lending[] => {
  const table = words.locale === "es" ? ES : EN;
  const say = (word: string, name?: string): string =>
    word === "tryAgain" ? words.say("retry") : filled(table[word] ?? word, name);
  const host: Attaching = {
    serve: served,
    url: fileUrl,
    keptHere,
    weighs,
    again: () => {},
    showing: () => () => {},
    plainly: () => words.say("failed"),
    say,
    named,
  };
  const sheet = page.querySelector<HTMLElement>(".page");
  if (sheet) bringNearPictures(sheet, pictured);
  for (const img of page.querySelectorAll<HTMLImageElement>(".page img[src]")) {
    const src = img.getAttribute("src") ?? "";
    if (attached(src)) attachedAs(page, img, src, () => true, [], host);
  }
  return widgetsDrawn(page, { say, ...theme });
};
