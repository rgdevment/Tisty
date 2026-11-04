// What Mermaid draws is seen only in a real browser, so one finishes every kind of diagram here and the contract looks at the result.
import { mkdtempSync, readdirSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { drawnInChrome } from "./chrome.mjs";
import { bundledFinisher, finisherPage } from "./finisher.mjs";

const app = resolve(dirname(fileURLToPath(import.meta.url)), "..");

const EVERY_KIND = [
  'flowchart TD\n A[Inicio] --> B{¿Sí?}\n B -->|sí| C((Fin))\n B -.->|no| D[(Base)]\n subgraph S\n C --> E>x]\n end\n click A href "https://a.example"',
  "graph LR\n a --> b\n b ==> c\n c --o d\n d --x e\n classDef k fill:#f00,stroke:#333;\n class a k",
  "sequenceDiagram\n participant A\n actor B\n A->>B: hola\n B-->>A: chao\n Note over A,B: nota\n loop x\n A->>A: y\n end\n alt a\n A->>B: b\n else c\n B->>A: d\n end",
  "classDiagram\n class A { +int x\n +f() }\n A <|-- B\n A *-- C : has",
  "stateDiagram-v2\n [*] --> A\n A --> B : x\n state B { [*] --> C }\n B --> [*]\n note right of A: n",
  "erDiagram\n A ||--o{ B : has\n A { string id PK }",
  "gantt\n title T\n dateFormat YYYY-MM-DD\n section S\n a :a1, 2026-01-01, 3d\n b :after a1, 2d",
  'pie title P\n "a" : 1\n "b" : 2',
  "journey\n title J\n section S\n a: 5: Me",
  "mindmap\n root((R))\n  a\n  b\n   c",
  "timeline\n title T\n 2026 : a : b\n 2027 : c",
  "gitGraph\n commit\n branch b\n commit\n checkout main\n merge b",
  "quadrantChart\n title Q\n x-axis a --> b\n y-axis c --> d\n p: [0.3, 0.6]",
  "requirementDiagram\n\n requirement r {\n  id: 1\n  text: t\n  risk: high\n  verifymethod: test\n }\n element e {\n  type: s\n }\n e - satisfies -> r",
  'C4Context\n Person(a, "A")\n System(b, "B")\n Rel(a, b, "x")',
  "sankey-beta\na,b,1\nb,c,1",
  "xychart-beta\n x-axis [a, b]\n y-axis 0 --> 3\n bar [1, 2]\n line [1, 2]",
  "block-beta\n columns 2\n a b\n a --> b",
  "architecture-beta\n group g(cloud)[G]\n service a(database)[A] in g\n service b(server)[B] in g\n a:R --> L:b",
  "kanban\n todo[Hacer]\n  a[Uno]",
  'packet-beta\n0-15: "a"\n16-31: "b"',
  "radar-beta\n axis a, b, c\n curve x{1,2,3}",
  'treemap-beta\n"a"\n  "b": 1\n  "c": 2',
];

const fenced = (text) => [...text.matchAll(/^```(?:mermaid|mmd)[^\n]*\n([\s\S]*?)^```/gm)].map(([, source]) => source);

const inTheGuide = ["en", "es"].flatMap((language) => {
  const folder = join(app, "src-tauri", "resources", "guide", language);
  return readdirSync(folder)
    .filter((one) => one.endsWith(".md"))
    .flatMap((one) => fenced(readFileSync(join(folder, one), "utf8")));
});

const sources = [...new Set([...EVERY_KIND, ...inTheGuide])];
const files = mkdtempSync(join(tmpdir(), "tisty-finished-"));
try {
  await bundledFinisher(files);
  const said = await drawnInChrome(sources, files, finisherPage("finishAll"), "what a document travels with");
  const wrong = said.filter((one) => one.wrong.length || !one.drawn);
  for (const one of wrong) console.error(`x  ${one.drawn ? one.wrong.join("; ") : "not drawn"}:\n${one.source}`);
  console.log(`${said.length - wrong.length} of ${said.length} diagrams travel as the contract says`);
  process.exitCode = wrong.length ? 1 : 0;
} finally {
  rmSync(files, { recursive: true, force: true });
}
