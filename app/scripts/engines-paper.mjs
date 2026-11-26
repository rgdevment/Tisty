import { readFileSync } from "node:fs";
import { chromium, webkit } from "playwright";
import { served } from "./engines/served.mjs";

const GUIDE = readFileSync(new URL("../src-tauri/resources/guide/es/guia.md", import.meta.url), "utf8");

const ENGINES = [
  { name: "WebKit", type: webkit },
  { name: "Chromium", type: chromium },
];

const KEPT = [
  { name: "a sheet sized to A4", asked: { mode: "sheeted", width: 794 }, wears: /\bsheeted\b/ },
  { name: "a sheet sized to A4, in the dark", asked: { mode: "sheeted", width: 794, dark: true }, wears: /\bsheeted\b/ },
  { name: "a task's page", asked: { mode: "tall", width: 760 }, wears: /\btall\b/ },
  { name: "a page inside a card", asked: { mode: "untitled", width: 640, dark: true }, wears: /\buntitled\b/ },
  { name: "a help page", asked: { mode: "plain", width: 900 }, wears: /^(?!.*\b(lively|tall)\b)/ },
  { name: "the document printed", asked: { mode: "lively", width: 1100 }, media: "print", wears: /\blively\b/ },
  { name: "the PDF's page", asked: { mode: "pdf", width: 643 }, media: "print", wears: /^(?!.*\blively\b)/ },
];

const MEASURED = [
  {
    name: "the PDF weighs the document's tables at the paper's size, as before",
    asked: { mode: "lively", width: 1100, measured: true },
  },
];

const lit = (said) => {
  const [r, g, b] = said.match(/[\d.]+/g).slice(0, 3).map(Number);
  const linear = (c) => {
    const s = c / 255;
    return s <= 0.04045 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4;
  };
  return 0.2126 * linear(r) + 0.7152 * linear(g) + 0.0722 * linear(b);
};

const contrast = (ink, paper) => {
  const [high, low] = [lit(ink), lit(paper)].sort((a, b) => b - a);
  return (high + 0.05) / (low + 0.05);
};

const near = (seen, wanted, by = 0.5) => Math.abs(seen - wanted) <= by;
const inside = ({ page }) => page.box[1] - Number.parseFloat(page["padding-left"]) - Number.parseFloat(page["padding-right"]);
const ratioOf = (one, root) => `${one.color} on ${root["background-color"]}, ${contrast(one.color, root["background-color"]).toFixed(2)}:1`;

const READS = [
  // The window holds the card to the document's measure; the page fills whatever card it gets.
  {
    name: "keeps 760px of words on a normal card",
    asked: { mode: "lively", width: 872 },
    holds: (seen) =>
      near(inside(seen), 760, 1) && near(seen.page.box[0], 0) ? null : `${inside(seen)}px of words starting at ${seen.page.box[0]}px`,
  },
  {
    name: "fills a full card, margins kept",
    asked: { mode: "lively", width: 1100 },
    holds: (seen) =>
      near(seen.page.box[1], 1100) && near(seen.page.box[0], 0) && seen.page["padding-left"] === "56px"
        ? null
        : `the page is ${seen.page.box[1]}px wide, starting at ${seen.page.box[0]}px`,
  },
  {
    name: "fills a narrow card, margins kept",
    asked: { mode: "lively", width: 600 },
    holds: (seen) =>
      near(seen.page.box[1], 600) && seen.page["padding-left"] === "56px" ? null : `the page is ${seen.page.box[1]}px wide`,
  },
  {
    name: "lets a width the person chose win over the measure",
    asked: { mode: "lively", width: 1100, wide: 900 },
    holds: (seen) => (near(seen.page.box[1], 900) ? null : `the page is ${seen.page.box[1]}px wide`),
  },
  {
    name: "opens with its title 40px from the top",
    asked: { mode: "lively", width: 1100 },
    holds: ({ title }) => (near(title.box[1], 40) ? null : `the title starts at ${title.box[1]}px`),
  },
  {
    name: "sets its title at 1.75em, 650, 1.2",
    asked: { mode: "lively", width: 1100 },
    holds: ({ title }) =>
      title["font-size"] === "28px" && title["font-weight"] === "650" && near(Number.parseFloat(title["line-height"]), 33.6, 0.05)
        ? null
        : `${title["font-size"]} ${title["font-weight"]} / ${title["line-height"]}`,
  },
  {
    name: "reads at 16px with 1.7 between lines",
    asked: { mode: "lively", width: 1100 },
    holds: ({ kinds: { p } }) =>
      p["font-size"] === "16px" && near(Number.parseFloat(p["line-height"]), 27.2, 0.05)
        ? null
        : `${p["font-size"]} / ${p["line-height"]}`,
  },
  {
    name: "puts its menu at the end of the title's line, where the words end",
    asked: { mode: "lively", width: 1100, row: true },
    holds: ({ row, rowOnTop, page, title }) => {
      const end = page.box[0] + page.box[1] - Number.parseFloat(page["padding-right"]);
      const middle = (box) => box[1] + box[3] / 2;
      const line = [title.box[0], title.box[1], title.box[2], Number.parseFloat(title["line-height"])];
      if (!rowOnTop) return "the title lies over the menu, so a click never reaches it";
      return row && near(row.box[0] + row.box[2], end, 1) && near(middle(row.box), middle(line), 1)
        ? null
        : `the menu stands at ${row?.box} for a line at ${line} ending at ${end}`;
    },
  },
  {
    name: "writes in a softer ink in the dark, well past AA",
    asked: { mode: "lively", width: 1100, dark: true },
    holds: ({ kinds: { p }, root }) =>
      p.color === "rgb(212, 212, 218)" && contrast(p.color, root["background-color"]) >= 7 ? null : ratioOf(p, root),
    says: ({ kinds: { p }, root }) => ratioOf(p, root),
  },
  {
    name: "keeps its headings bright in the dark",
    asked: { mode: "lively", width: 1100, dark: true },
    holds: ({ title, kinds: { h2, h3 } }) =>
      [title.color, h2.color, h3.color].join() === "rgb(242, 242, 247),rgb(242, 242, 247),rgb(236, 236, 241)"
        ? null
        : [title.color, h2.color, h3.color].join(" / "),
    says: ({ kinds: { h2, h3 }, root }) => `${ratioOf(h2, root)}; ${ratioOf(h3, root)}`,
  },
  {
    name: "keeps its ink in the light",
    asked: { mode: "lively", width: 1100 },
    holds: ({ title, kinds: { p } }) =>
      p.color === "rgb(29, 29, 31)" && title.color === p.color ? null : `${p.color} / ${title.color}`,
  },
];

const differs = (one, other, path = "") => {
  if (JSON.stringify(one) === JSON.stringify(other)) return null;
  if (typeof one !== "object" || one === null || typeof other !== "object" || other === null)
    return `${path || "the page"}: ${JSON.stringify(other)} became ${JSON.stringify(one)}`;
  for (const key of new Set([...Object.keys(one), ...Object.keys(other)])) {
    const found = differs(one[key], other[key], `${path}.${key}`);
    if (found) return found;
  }
  return null;
};

const ranIn = async (engine, url) => {
  const browser = await engine.type.launch({ headless: true });
  const page = await browser.newPage({ viewport: { width: 1200, height: 1000 } });
  const failed = [];
  try {
    await page.goto(url);
    await page.waitForFunction(() => window.paperReady === true, null, { timeout: 120_000 });
    const laid = new Map();
    const lay = async (asked, media = "screen") => {
      const key = JSON.stringify([asked, media]);
      if (!laid.has(key)) {
        await page.emulateMedia({ media });
        laid.set(key, await page.evaluate(([body, one]) => window.paper.lay(body, one), [GUIDE, asked]));
      }
      return laid.get(key);
    };
    for (const one of KEPT) {
      const worn = await lay(one.asked, one.media);
      const bare = await lay({ ...one.asked, bare: true }, one.media);
      const printing = one.media === "print";
      const why =
        worn.printed !== printing
          ? `the frame was ${worn.printed ? "" : "not "}printing`
          : !one.wears.test(worn.classes)
            ? `the frame wore "${worn.classes}"`
            : differs(worn, bare);
      console.log(`${why ? "✗" : "✓"} ${engine.name}: ${one.name} looks as it did without lively.css${why ? `\n    ${why}` : ""}`);
      if (why) failed.push(one.name);
    }
    for (const one of MEASURED) {
      const worn = await lay(one.asked);
      const bare = await lay({ ...one.asked, bare: true });
      const why = differs(worn, bare);
      console.log(`${why ? "✗" : "✓"} ${engine.name}: ${one.name}${why ? `\n    ${why}` : ""}`);
      if (why) failed.push(one.name);
    }
    for (const one of READS) {
      const seen = await lay(one.asked);
      const why = one.holds(seen);
      const said = one.says && !why ? ` (${one.says(seen)})` : "";
      console.log(`${why ? "✗" : "✓"} ${engine.name}: the document ${one.name}${said}${why ? `\n    ${why}` : ""}`);
      if (why) failed.push(one.name);
    }
  } finally {
    await browser.close();
  }
  return failed;
};

const server = await served("paper.ts");
let failures = 0;
try {
  for (const engine of ENGINES) failures += (await ranIn(engine, server.url)).length;
} finally {
  await server.close();
}
console.log(failures ? `\n${failures} checks failed` : "\nevery check holds");
process.exit(failures ? 1 : 0);
