import { checkHtml } from "@tisty/contracts";
import { finished } from "tisty-editor/read";

const parts = (drawing: Element) => drawing.querySelectorAll("svg, text").length;

const figure = (source: string): string => {
  const held = document.createElement("code");
  held.className = "language-mermaid";
  held.textContent = source;
  return `<figure class="diagram"><pre>${held.outerHTML}</pre></figure>`;
};

const answered = async <Asked, Answer>(each: (one: Asked) => Promise<Answer>) => {
  const asked: Asked[] = await (await fetch("/sources")).json();
  const answers = [];
  for (const one of asked) answers.push(await each(one));
  await fetch("/drawn", { method: "POST", body: JSON.stringify(answers) });
};

/** Finishes each diagram in this browser and answers what the contract says of it. Only `scripts/finished.mjs` loads this. */
export const finishAll = (): Promise<void> =>
  answered(async (source: string) => {
    const { html, notDrawn } = await finished(figure(source));
    const page = new DOMParser().parseFromString(html, "text/html");
    const [light, dark, ...more] = page.querySelectorAll("figure.diagram > svg");
    // One source says the same in both looks, so another count means one is not the whole drawing. Its box can differ: gitGraph makes up its ids.
    const both =
      !more.length &&
      light?.getAttribute("data-theme") === "light" &&
      dark?.getAttribute("data-theme") === "dark" &&
      parts(light) === parts(dark);
    return { source, drawn: notDrawn === 0 && both, wrong: checkHtml(html) };
  });

/** Finishes each document on its way. Only `scripts/journey.mjs` loads this, and Rust looks at what comes back before sealing it. */
export const finishWhole = (): Promise<void> =>
  answered(async ({ name, html }: { name: string; html: string }) => ({
    name,
    ...(await finished(html)),
  }));
