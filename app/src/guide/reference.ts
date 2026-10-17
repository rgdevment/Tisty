export type Topic = {
  readonly at: number;
  readonly heading: string | null;
  readonly body: string;
  readonly slug: string;
};

const opening = (line: string): string | undefined => /^(`{3,}|~{3,})/.exec(line)?.[1];

export const plain = (said: string): string =>
  said
    .normalize("NFD")
    .replace(/\p{M}/gu, "")
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-|-$/g, "");

// Each `##` heading starts a topic; the start page is what comes before them and the first one, the glance.
export const topicsOf = (text: string): Topic[] => {
  const found: { heading: string | null; lines: string[] }[] = [{ heading: null, lines: [] }];
  let fence: string | null = null;
  let glanced = false;
  for (const line of text.replace(/\r\n/g, "\n").split("\n")) {
    if (fence) {
      if (line.trim() === fence) fence = null;
    } else if (opening(line)) {
      fence = opening(line) ?? null;
    } else if (line.startsWith("## ") && glanced) {
      found.push({ heading: line.slice(3).trim(), lines: [] });
    } else if (line.startsWith("## ")) {
      glanced = true;
    }
    found[found.length - 1].lines.push(line);
  }
  return found.map((one, at) => ({
    at,
    heading: one.heading,
    body: one.lines.join("\n"),
    slug: at === 0 ? "index" : plain(one.heading ?? "") || `topic-${at}`,
  }));
};

const escaped = (said: string): string =>
  said.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");

export const examplesIn = (page: Document | Element): Element[] =>
  Array.from(page.querySelectorAll("figure.code > pre > code.language-markdown"));

export type Words = { readonly written: string; readonly drawn: string };

export const shownBeside = (code: Element, drawn: string, words: Words) => {
  const page = code.ownerDocument;
  const figure = page.createElement("figure");
  figure.className = "example";
  const source = (code.textContent ?? "").replace(/\n$/, "");
  figure.innerHTML = `<pre class="written" data-said="${words.written}"><code>${escaped(source)}</code></pre><div class="drawn" data-said="${words.drawn}">${drawn}</div>`;
  code.closest("figure")?.replaceWith(figure);
};
