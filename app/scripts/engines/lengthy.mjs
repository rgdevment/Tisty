const SENTENCES = [
  "El taller abría temprano y el olor a madera recién cortada llenaba la calle entera.",
  "Nadie recordaba quién había pintado la puerta de verde, pero todos la usaban de señal.",
  "Las cartas llegaban los martes, atadas con un cordel rojo que la cartera guardaba después.",
  "En la cocina se discutía de todo menos del tiempo, que cada uno juzgaba a su manera.",
  "El río bajaba crecido aquel otoño y los puentes viejos crujían bajo los carros.",
  "Una lista de tareas colgaba junto al reloj, tachada a medias y con letra apurada.",
];

const MIDDLE = "marcamedia";
const END = "marcafinal";

const sentence = (at) => SENTENCES[at % SENTENCES.length];
const leadOf = (marker) => `la ${marker}.`;

const paragraph = (chapter, at, marker) => {
  const said = [
    sentence(chapter + at),
    `Ese día **${chapter}.${at}** quedó anotado en el [registro ${chapter}](https://example.com/${chapter}/${at}) con la etiqueta #tema${chapter % 7}.`,
    sentence(chapter * 3 + at + 1),
    marker ? `Aquí escribe ${leadOf(marker)}` : sentence(chapter + at * 5 + 2),
  ];
  return said.join(" ");
};

const pieces = (chapter) => [
  `Antes del código va una nota al pie[^${chapter}] y un \`detalle\` en línea.`,
  "",
  "```js",
  "const total = items.reduce((sum, one) => sum + one.price, 0);",
  `console.log("capítulo ${chapter}", total);`,
  "```",
  "",
  "| Mes | Ventas | Notas |",
  "|---|---|---|",
  `| enero | ${100 + chapter} | buen inicio |`,
  "| febrero | 98 | baja |",
  "",
  "```mermaid",
  "flowchart LR",
  `  A${chapter}[Idea] --> B${chapter}[Borrador]`,
  `  B${chapter} --> C${chapter}[Libro]`,
  "```",
  "",
  `![Portada ${chapter}](portada.png)`,
  "",
];

const LINES_PER_CHAPTER = 6;
const PIECES_EVERY = 10;

const markerOf = (chapter, at, chapters) => {
  if (chapter === Math.floor(chapters / 2) && at === 2) return MIDDLE;
  if (chapter === chapters - 1 && at === LINES_PER_CHAPTER - 2) return END;
  return null;
};

const chapterOf = (chapter, chapters) => [
  `## Capítulo ${chapter}`,
  "",
  ...(chapter % PIECES_EVERY === 0 ? pieces(chapter) : []),
  ...Array.from({ length: LINES_PER_CHAPTER }, (_, at) => [
    paragraph(chapter, at, markerOf(chapter, at, chapters)),
    "",
  ]).flat(),
  ...(chapter % 2 === 0 ? [`- primero del ${chapter}`, "- segundo con #lista", "- tercero", ""] : []),
];

export const pageOf = (chapters) => {
  const notes = Array.from({ length: chapters }, (_, chapter) => chapter)
    .filter((chapter) => chapter % PIECES_EVERY === 0)
    .flatMap((chapter) => [`[^${chapter}]: La nota del capítulo ${chapter}.`, ""]);
  const body = Array.from({ length: chapters }, (_, chapter) => chapterOf(chapter, chapters)).flat();
  return ["# Un documento largo", "", ...body, ...notes].join("\n");
};

const spotOf = (marker) => ({ words: `${marker}.`, lead: leadOf(marker) });

export const SPOTS = { middle: spotOf(MIDDLE), end: spotOf(END) };
