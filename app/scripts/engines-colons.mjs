import { webkit } from "playwright";
import { CASES } from "./engines/cases.mjs";
import { launched, sheetOf } from "./engines/hands.mjs";
import { served } from "./engines/served.mjs";

const DOCS = [
  "# Uno\n\n:::details A\n\nB\n\n:::\n",
  "# Dos\n\n:::details Fuera\n\n:::details Dentro\n\nx\n\n:::\n\nMás\n\n:::\n",
  "# Tres\n\n> :::details Citado\n>\n> Dentro\n>\n> :::\n",
  "# Cuatro\n\n:::tip\n\nUn consejo\n\n:::\n\nDespués\n",
  "# Cinco\n\nTexto\n\n---\n:::details Bajo la regla\n\nDentro\n\n:::\n",
  "# Seis\n\nTítulo\n===\n:::details Bajo el título\n\nDentro\n\n:::\n",
  "# Siete\n\n```\n:::details En código\n\n:::\n```\n",
  "# Ocho\n\n:::details A\n\n::: nota\n\nx\n\n:::\n\nluego\n\n:::\n",
];

const docs = [...new Set([...CASES.map((one) => one.doc).filter((doc) => doc.includes(":::")), ...DOCS])];
const server = await served();
let amiss = 0;
try {
  const { browser, page } = await launched(webkit, server.url);
  for (const doc of docs) {
    await sheetOf(page, doc);
    await page.waitForFunction(() => window.engines.drawn());
    const said = await page.evaluate(() => window.engines.colonsDrawn());
    if (said === "ok") continue;
    amiss += 1;
    console.error(`x  ${JSON.stringify(doc.slice(0, 60))}: ${said}`);
  }
  await browser.close();
} finally {
  await server.close();
}
if (amiss) {
  console.error(`${amiss} of ${docs.length} pages pair their ::: lines differently from the core`);
  process.exitCode = 1;
} else console.log(`ok all ${docs.length} pages pair their ::: lines as the core draws them`);
