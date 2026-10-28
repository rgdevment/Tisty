import type { Chosen } from "../views";
import type { HelpEntry, HelpPictures } from "./calls";
import type { Topic } from "./reference";

export const LINKED = "help/";
export const MARKS = "markdown:";

export const marksOf = (topics: Topic[], section: string): HelpEntry[] =>
  topics.map((one, at) => ({
    slug: `${MARKS}${one.slug}`,
    title: one.heading ?? section,
    section,
    order: 1000 + at,
    twin: "",
  }));

export interface Suggested {
  slug: string;
  heading?: string;
}

type Twice = { en: Suggested; es: Suggested };

const twice = (en: string, es: string, enHead?: string, esHead?: string): Twice => ({
  en: { slug: en, heading: enHead },
  es: { slug: es, heading: esHead },
});

const SYNC = ["sync-and-backup", "sincronizar-y-respaldar"] as const;

const SUGGESTED: Record<string, Twice[]> = {
  "keeping:sync": [
    twice(...SYNC, "A place that already holds a Tisty", "Un lugar que ya guarda un Tisty"),
    twice(...SYNC, "Confirming your machines", "Confirmar tus equipos"),
    twice(...SYNC),
  ],
  "keeping:security": [
    twice(...SYNC, "Encrypting your cloud", "Cifrar tu nube"),
    twice(...SYNC, "Your other computers", "Tus otros equipos"),
    twice(...SYNC, "Keep the phrase safe", "Guarda bien la frase"),
  ],
  "keeping:data": [twice(...SYNC, "Backups", "Respaldos"), twice("privacy", "privacidad")],
  "keeping:agents": [
    twice("assistants-and-mcp", "asistentes-y-mcp"),
    twice("mcp-permissions", "que-puede-hacer-un-asistente"),
    twice("mcp-tools", "herramientas-mcp"),
  ],
  "keeping:general": [
    twice("keyboard-shortcuts", "atajos-de-teclado"),
    twice("faq", "preguntas-frecuentes"),
  ],
  docs: [
    twice("documents", "documentos"),
    twice("markdown-editor", "editor-markdown"),
    twice("pages-and-pdf", "paginas-y-pdf"),
    twice("diagrams", "diagramas"),
  ],
  tasks: [
    twice("tasks", "tareas"),
    twice("natural-language", "lenguaje-natural"),
    twice("spread-and-reminders", "reparto-y-recordatorios"),
  ],
  archive: [twice("archive", "el-archivo")],
};

export const cameFrom = (chosen: Chosen): string =>
  chosen.named === "help"
    ? (chosen.from ?? "tasks")
    : chosen.named === "keeping"
      ? `keeping:${chosen.tab ?? "general"}`
      : (chosen.named ?? "tasks");

export const suggestedFor = (from: string | undefined, spanish: boolean): Suggested[] =>
  (SUGGESTED[from ?? ""] ?? SUGGESTED.tasks).map((one) => (spanish ? one.es : one.en));

export const outlined = <T extends { level: number }>(heads: T[]): T[] => {
  const rest = heads.filter((one) => one.level > 0).slice(1);
  const top = Math.min(...rest.map((one) => one.level));
  return rest.filter((one) => one.level === top);
};

export const sectionsOf = (entries: HelpEntry[]): [string, HelpEntry[]][] =>
  entries
    .filter((one) => one.order > 0)
    .reduce<[string, HelpEntry[]][]>((all, one) => {
      const last = all[all.length - 1];
      if (last && last[0] === one.section) last[1].push(one);
      else all.push([one.section, [one]]);
      return all;
    }, []);

const plain = (text: string): string => text.normalize("NFD").replace(/\p{M}/gu, "").toLowerCase();

const SAID = 140;

export const describedBy = (body: string): string => {
  const first = body
    .split(/\n\s*\n/)
    .map((one) => one.trim())
    .find((one) => one && !/^[#>|`<!\-*\d]/.test(one));
  const words = (first ?? "")
    .replace(/\[([^\]]+)\]\([^)]*\)/g, "$1")
    .replace(/[*_`]/g, "")
    .replace(/\s+/g, " ");
  return words.length > SAID ? `${words.slice(0, SAID - 3).replace(/\s\S*$/, "")}…` : words;
};

export interface Found {
  entry: HelpEntry;
  near: string;
}

const NEAR = 70;

export const searched = (
  entries: HelpEntry[],
  bodies: Record<string, string>,
  asked: string,
): Found[] => {
  const words = plain(asked).split(/\s+/).filter(Boolean);
  if (words.length === 0) return [];
  return entries
    .map((entry) => {
      const body = bodies[entry.slug] ?? "";
      const title = plain(entry.title);
      const read = plain(body);
      if (!words.every((one) => title.includes(one) || read.includes(one))) return null;
      const score = words.reduce((sum, one) => sum + (title.includes(one) ? 3 : 1), 0);
      const at = read.indexOf(words[0]);
      const near =
        at < 0 ? "" : body.slice(Math.max(0, at - NEAR / 2), at + NEAR).replace(/\s+/g, " ");
      return { entry, near: near.replace(/[#*`[\]()>|_]/g, "").trim(), score };
    })
    .filter((one): one is Found & { score: number } => one !== null)
    .sort((a, b) => b.score - a.score)
    .slice(0, 12)
    .map(({ entry, near }) => ({ entry, near }));
};

const here = (slug: string, anchor?: string) => `#${LINKED}${slug}${anchor ? `/${anchor}` : ""}`;

// Some editors write an anchor's accents as %C3%B3; the page's ids carry the letters themselves.
const unescaped = (anchor: string) => {
  try {
    return decodeURIComponent(anchor);
  } catch {
    return anchor;
  }
};

/** Where a page of the published guide or help sits in the app, in the app's language; null for any other address. */
export const carried = (url: string, spanish: boolean): string | null => {
  const found = /^https:\/\/(markdown|www)\.tisty\.app\/([^?#\s]*)(?:\?[^#\s]*)?(?:#(\S*))?$/.exec(
    url,
  );
  if (!found) return null;
  const [, site, path, anchor] = found;
  const parts = path.split("/").filter(Boolean);
  const anchored = anchor ? unescaped(anchor) : undefined;
  if (site === "markdown") {
    // A page of the other language's guide is not in the app, so it stays on the site.
    const rest = parts[0] === "es" ? parts.slice(1) : parts;
    if ((parts[0] === "es") !== spanish || rest.length > 1) return null;
    return here(`${MARKS}${rest[0] ?? "index"}`, anchored);
  }
  const [first, second, third, ...more] = parts;
  const slug = spanish
    ? first === "es" && second === "ayuda" && more.length === 0 && third
    : first === "docs" && third === undefined && second;
  return slug ? here(slug, anchored) : null;
};

/** The guide's address for one of its pages, for a link the app has no copy of. */
export const guideAt = (slug: string, anchor: string | null | undefined, spanish: boolean) =>
  `https://markdown.tisty.app/${spanish ? "es/" : ""}${slug === "index" ? "" : slug}${anchor ? `#${anchor}` : ""}`;

/** Links to other help pages and to the published guide and help, turned into pages the window carries. */
export const linkedHere = (markdown: string, spanish: boolean): string => {
  const kept = (whole: string, url: string, put: (here: string) => string) => {
    const at = carried(url, spanish);
    return at ? put(at) : whole;
  };
  return markdown
    .replace(
      /\]\(help:([^)#\s]*)(?:#([^)\s]*))?\)/g,
      (_, slug: string, anchor?: string) => `](${here(slug, anchor)})`,
    )
    .replace(/\]\((https:\/\/[^)\s]+)\)/g, (whole, url: string) =>
      kept(whole, url, (at) => `](${at})`),
    )
    .replace(/<(https:\/\/[^>\s]+)>/g, (whole, url: string) =>
      kept(whole, url, (at) => `[${url}](${at})`),
    )
    .replace(/^( {0,3}\[[^\]]+\]:[ \t]*)(https:\/\/\S+)/gm, (whole, lead: string, url: string) =>
      kept(whole, url, (at) => `${lead}${at}`),
    );
};

export const aimedAt = (id: string): { slug: string; anchor: string | null } | null => {
  if (!id.startsWith(LINKED)) return null;
  const [slug, ...anchor] = id.slice(LINKED.length).split("/");
  return { slug: slug ?? "", anchor: anchor.join("/") || null };
};

const ONLY_ONLINE = new Set(["installation", "instalacion"]);

export const forTheWindow = (entries: HelpEntry[]): HelpEntry[] =>
  entries.filter((one) => !ONLY_ONLINE.has(one.slug));

export const dressed = (
  html: string,
  pictures: HelpPictures | null,
  shown: (path: string) => string,
  away: (alt: string) => string,
): string => {
  const page = new DOMParser().parseFromString(`<article>${html}</article>`, "text/html");
  for (const img of page.querySelectorAll<HTMLImageElement>('img[src^="https://"]')) {
    const src = img.getAttribute("src") ?? "";
    const kept = pictures?.kept[src];
    if (kept) {
      img.setAttribute("src", shown(kept));
      continue;
    }
    const note = page.createElement("span");
    note.className = "picture-away";
    note.textContent = away(img.getAttribute("alt") ?? "");
    img.replaceWith(note);
  }
  return page.querySelector("article")?.innerHTML ?? "";
};
