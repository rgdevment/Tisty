const SITE = "https://markdown.tisty.app/";

export const LANGUAGES = [
  {
    code: "en",
    file: "en.md",
    dir: "",
    start: "Overview",
    topics: "Guide",
    search: "Search",
    near: "Next and previous",
    home: "https://www.tisty.app/",
    meet: "Meet Tisty",
    docs: "Docs",
    docsAt: "https://www.tisty.app/docs/",
    before: "Previous",
    after: "Next",
    written: "You write",
    drawn: "Tisty shows",
    name: "EN",
    reading: "Read in English",
    dark: "Light or dark",
    locale: "en_US",
    image: "https://www.tisty.app/tisty-social.png",
    title: "Tisty Markdown guide: syntax and examples",
    site: "Tisty Markdown",
    said: {
      index: "The Tisty Markdown guide: every mark a Tisty document can hold, from headings and tables to pens, icons and diagrams, with examples as they are drawn.",
      properties: "Properties at the top of a Tisty document: title, tags, paper, language and a task's dates, written as front matter between two --- lines.",
      "the-title": "How a Tisty document gets its title: from the first line with words, unless its properties name one with title:.",
      headings: "Headings in Tisty Markdown: six levels from # to ######, always followed by a space, and how to start a line with a # that is neither heading nor tag.",
      "paragraphs-and-line-breaks": "How Tisty Markdown treats line breaks and paragraphs: a single break stays a break, unlike GitHub, and a blank line starts a new paragraph.",
      "bold-italic-struck-through-and-code": "Bold, italic, strikethrough and inline code in Tisty Markdown, and the rules that decide when ** or _ counts as a mark and when it stays as written.",
      underline: "Underline in Tisty Markdown: two plus signs on each side, ++like this++, and why C++ and i++ never turn into an underline.",
      pens: "Highlight text in colour with Tisty's pens: ==yellow== by default, or any of twelve colours named in braces, such as ==words=={green}.",
      links: "Links in Tisty Markdown: inline, by reference and bare addresses, link cards for a link alone in its paragraph, and which schemes are followed.",
      lists: "Bulleted, numbered and nested lists in Tisty Markdown, plus stages: boxes you tick with - [ ] and - [x] that are not tasks.",
      "quotes-and-callouts": "Quotes and the five callouts in Tisty Markdown: NOTE, TIP, IMPORTANT, WARNING and CAUTION, written with > [!NOTE] or between ::: fences.",
      "collapsible-sections": "Collapsible sections in Tisty Markdown: a title you click to fold what is under it, written with :::details, how they nest and how they export.",
      tags: "Tags in a Tisty document: a #word in the sentence files it under that tag, and the rules on accents, numbers, code and what does not count as one.",
      icons: "Mark a line with an icon in Tisty Markdown: ::dog orange:: between double colons, with an optional colour, picked and searched from the / menu.",
      emoji: "Emoji in Tisty Markdown: write them as themselves or with a GitHub shortcode such as :tada:, and when a shortcode stays as plain words.",
      "other-documents-and-pages": "Link another Tisty document or page with tisty:doc/: as a card when alone with !, as a word in a sentence, and how cards place pages in a book.",
      attachments: "Attachments in Tisty Markdown: pictures, sound, video, HTML pages and other files dragged in, written like an image and named by a stamp of their bytes.",
      references: "References in Tisty Markdown: [[name]] names something without linking it, and [[#2]] shows a task's step or a document's stage by its number.",
      footnotes: "Footnotes in Tisty Markdown: a short label such as [^1] or [^vet] calls a note said once on its own line, the same way GitHub reads them.",
      "code-blocks": "Code blocks in Tisty Markdown: fences with a language and a title=\"name\", colours per language, inline code, and showing a fence inside a block.",
      "diagrams-formulas-and-widgets": "Mermaid diagrams, math formulas and widgets in Tisty Markdown: blocks drawn instead of shown as code, the diagram window, and what each one can do.",
      tables: "Tables in Tisty Markdown: alignment, column widths and typed columns for money, dates, ratings and more, with sums at the foot of the table.",
      "page-breaks": "Page breaks in Tisty Markdown: a --- rule on its own line starts a new sheet in print and PDF, and how sheets turn on their own.",
      "comments-and-entities": "Comments and entities in Tisty Markdown: notes to yourself that the page does not show, and names for characters that are awkward to type.",
      "what-is-outside-the-language": "What happens to HTML and other content from outside Tisty Markdown: what is converted on the way in, and what stays exactly as written.",
    },
  },
  {
    code: "es",
    file: "es.md",
    dir: "es/",
    start: "Vista general",
    topics: "Guía",
    search: "Buscar",
    near: "Anterior y siguiente",
    home: "https://www.tisty.app/es/",
    meet: "Conoce Tisty",
    docs: "Ayuda",
    docsAt: "https://www.tisty.app/es/ayuda/",
    before: "Anterior",
    after: "Siguiente",
    written: "Escribes",
    drawn: "Tisty muestra",
    name: "ES",
    reading: "Leer en español",
    dark: "Claro u oscuro",
    locale: "es_ES",
    image: "https://www.tisty.app/tisty-social-es.png",
    title: "Guía de Markdown de Tisty: sintaxis y ejemplos",
    site: "Markdown de Tisty",
    said: {
      index: "La guía de Markdown de Tisty: cada marca que puede llevar un documento, de títulos y tablas a rotuladores, iconos y diagramas, con ejemplos dibujados.",
      propiedades: "Propiedades al principio de un documento de Tisty: título, etiquetas, papel, idioma y fechas de una tarea, escritas como front matter entre dos líneas ---.",
      "el-titulo": "Cómo recibe su título un documento de Tisty: de la primera línea con palabras, salvo que sus propiedades digan otro con title:.",
      titulos: "Títulos en Markdown de Tisty: seis niveles de # a ######, siempre con un espacio detrás, y cómo empezar una línea con un # que no sea título ni etiqueta.",
      "parrafos-y-saltos-de-linea": "Saltos de línea y párrafos en Markdown de Tisty: un salto simple se queda como salto, a diferencia de GitHub, y una línea en blanco abre un párrafo.",
      "negrita-cursiva-tachado-y-codigo": "Negrita, cursiva, tachado y código en Markdown de Tisty, y las reglas que deciden cuándo ** o _ cuentan como marca y cuándo quedan como se escriben.",
      subrayado: "Subrayado en Markdown de Tisty: dos signos más a cada lado, ++así++, y por qué C++ e i++ nunca se convierten en subrayado.",
      rotuladores: "Destaca texto en color con los rotuladores de Tisty: ==amarillo== por defecto, o cualquiera de doce colores entre llaves, como ==palabras=={green}.",
      enlaces: "Enlaces en Markdown de Tisty: en línea, por referencia y direcciones sueltas, tarjetas para un enlace solo en su párrafo y qué esquemas se siguen.",
      listas: "Listas con viñetas, numeradas y anidadas en Markdown de Tisty, más las etapas: casillas que marcas con - [ ] y - [x] y que no son tareas.",
      "citas-y-avisos": "Citas y los cinco avisos de Markdown de Tisty: NOTE, TIP, IMPORTANT, WARNING y CAUTION, escritos con > [!NOTE] o entre vallas :::.",
      "secciones-plegables": "Secciones plegables en Markdown de Tisty: un título en el que haces clic para plegar lo de debajo, escrito con :::details, cómo se anidan y se exportan.",
      etiquetas: "Etiquetas en un documento de Tisty: una #palabra en la frase lo archiva bajo esa etiqueta, y las reglas sobre acentos, números, código y lo que no cuenta.",
      iconos: "Marca una línea con un icono en Markdown de Tisty: ::dog orange:: entre dos pares de dos puntos, con color opcional, buscado desde el menú /.",
      emojis: "Emojis en Markdown de Tisty: escríbelos tal cual o con un código de GitHub como :tada:, y cuándo un código se queda como palabras.",
      "otros-documentos-y-paginas": "Enlaza otro documento o página de Tisty con tisty:doc/: como tarjeta si va solo con !, como palabra en una frase, y cómo las tarjetas ordenan las páginas.",
      adjuntos: "Adjuntos en Markdown de Tisty: imágenes, sonido, video, páginas HTML y otros archivos que arrastras, escritos como una imagen y nombrados por un sello.",
      referencias: "Referencias en Markdown de Tisty: [[nombre]] nombra algo sin enlazarlo, y [[#2]] muestra un paso de una tarea o una etapa de un documento por su número.",
      "notas-al-pie": "Notas al pie en Markdown de Tisty: una etiqueta corta como [^1] o [^vet] llama a una nota escrita una vez en su propia línea, igual que en GitHub.",
      "bloques-de-codigo": "Bloques de código en Markdown de Tisty: vallas con lenguaje y title=\"nombre\", colores por lenguaje, código en línea y una valla dentro de otra.",
      "diagramas-formulas-y-widgets": "Diagramas Mermaid, fórmulas y widgets en Markdown de Tisty: bloques que se dibujan en vez de mostrarse como código, y la ventana de diagramas.",
      tablas: "Tablas en Markdown de Tisty: alineación, ancho de columnas y columnas con tipo para dinero, fechas, valoraciones y más, con sumas al pie de la tabla.",
      "saltos-de-pagina": "Saltos de página en Markdown de Tisty: una raya --- sola en su línea empieza una hoja nueva al imprimir o en PDF, y cómo las hojas pasan solas.",
      "comentarios-y-entidades": "Comentarios y entidades en Markdown de Tisty: notas para ti que la página no muestra, y nombres para caracteres difíciles de escribir.",
      "lo-que-queda-fuera-del-lenguaje": "Qué pasa con el HTML y lo que llega de fuera de Markdown de Tisty: lo que se convierte al entrar y lo que se queda tal como está escrito.",
    },
  },
];

export const escaped = (said) =>
  said.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");

// Pages answers a bare path, and a link ending in .html would cost every click a redirect.
export const href = (slug) => (slug === "index" ? "./" : slug);

const told = (said) => escaped(said).replace(/"/g, "&quot;");

const address = (language, page) =>
  `${SITE}${language.dir}${page.slug === "index" ? "" : page.slug}`;

// It only remembers light or dark; every page reads without it.
export const THEME = `<script>(()=>{const r=document.documentElement;r.classList.add("js");let s=null;try{s=localStorage.getItem("tisty-guide-theme")}catch{}r.classList.toggle("dark",s?s==="dark":matchMedia("(prefers-color-scheme: dark)").matches)})()</script>`;
const SWITCH = `(()=>{const d=document.documentElement.classList.toggle('dark');try{localStorage.setItem('tisty-guide-theme',d?'dark':'light')}catch{}})()`;
const MOON = `<svg class="moon" viewBox="0 0 24 24" aria-hidden="true"><path d="M20.985 12.486a9 9 0 1 1-9.473-9.472c.405-.022.617.46.402.803a6 6 0 0 0 8.268 8.268c.344-.215.825-.004.803.401"/></svg>`;
const SUN = `<svg class="sun" viewBox="0 0 24 24" aria-hidden="true"><circle cx="12" cy="12" r="4"/><path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M6.34 17.66l-1.41 1.41M19.07 4.93l-1.41 1.41"/></svg>`;

const twinOf = (built, language, page) => {
  const other = LANGUAGES.find((one) => one !== language);
  return { other, twin: built.get(other.code)[page.at] ?? built.get(other.code)[0] };
};

const jsonLd = (said) => `<script type="application/ld+json">${JSON.stringify(said).replace(/</g, "\\u003c")}</script>`;

const TISTY = { "@type": "Organization", name: "Tisty", url: "https://www.tisty.app/" };

const described = (language, page) =>
  jsonLd({
    "@context": "https://schema.org",
    "@type": page.slug === "index" ? "WebSite" : "TechArticle",
    ...(page.slug === "index"
      ? { name: language.site }
      : {
          headline: page.title,
          dateModified: page.changed,
          author: TISTY,
          image: language.image,
        }),
    description: page.said,
    inLanguage: language.code,
    url: address(language, page),
    isPartOf: { "@type": "WebSite", name: language.site, url: `${SITE}${language.dir}` },
    publisher: TISTY,
  });

const crumbs = (language, page) =>
  page.slug === "index"
    ? ""
    : jsonLd({
        "@context": "https://schema.org",
        "@type": "BreadcrumbList",
        itemListElement: [
          { "@type": "ListItem", position: 1, name: language.site, item: address(language, { slug: "index" }) },
          { "@type": "ListItem", position: 2, name: page.title, item: address(language, page) },
        ],
      });

export const describedAs = (language, slug) => {
  const said = language.said[slug];
  if (!said) throw new Error(`the ${language.code} guide has no description for "${slug}": add one to LANGUAGES in guide-site.mjs`);
  return said;
};

// An example shows how a heading looks; it is no section of the guide to outline, link to or find in a search.
export const muted = (document) => {
  for (const shown of document.querySelectorAll(".drawn h1, .drawn h2, .drawn h3, .drawn h4, .drawn h5, .drawn h6")) {
    shown.setAttribute("role", "presentation");
    shown.removeAttribute("id");
  }
};

// A topic opens on a `##` of the whole reference, so on a page of its own every heading moves up a level.
export const raised = (document) => {
  const article = document.querySelector("article");
  const heads = article.querySelectorAll(":scope > h2, :scope > h3, :scope > h4, :scope > h5, :scope > h6");
  for (const head of heads) {
    const above = document.createElement(`h${Number(head.localName[1]) - 1}`);
    for (const { name, value } of head.attributes) above.setAttribute(name, value);
    above.append(...head.childNodes);
    head.replaceWith(above);
  }
};

export const commitsIn = (log) =>
  log
    .trim()
    .split("\n")
    .filter(Boolean)
    .map((line) => {
      const [hash, date] = line.split(" ");
      return { hash, date };
    });

// A topic changed on the oldest revision in the unbroken run that still says the same, so a release that leaves it alone does not pass it off as new.
export const changedOn = (revisions) => {
  const newest = new Map(revisions[0].topics.map((one) => [one.slug, one.body]));
  const dates = new Map();
  const open = new Set(newest.keys());
  for (const { date, topics } of revisions) {
    const said = new Map(topics.map((one) => [one.slug, one.body]));
    for (const slug of [...open]) {
      if (said.get(slug) === newest.get(slug)) dates.set(slug, date);
      else open.delete(slug);
    }
  }
  return dates;
};

const heads = (language, page, title, other, twin) => {
  const here = address(language, page);
  const first = LANGUAGES[0];
  return `<meta name="description" content="${told(page.said)}">
<link rel="canonical" href="${here}">
<link rel="alternate" hreflang="${language.code}" href="${here}">
<link rel="alternate" hreflang="${other.code}" href="${address(other, twin)}">
<link rel="alternate" hreflang="x-default" href="${address(first, language === first ? page : twin)}">
<meta property="og:type" content="${page.slug === "index" ? "website" : "article"}">
<meta property="og:site_name" content="${language.site}">
<meta property="og:title" content="${told(title)}">
<meta property="og:description" content="${told(page.said)}">
<meta property="og:url" content="${here}">
<meta property="og:image" content="${language.image}">
<meta property="og:locale" content="${language.locale}">
<meta property="og:locale:alternate" content="${other.locale}">
<meta name="twitter:card" content="summary_large_image">
${described(language, page)}${crumbs(language, page)}`;
};

// Plain HTML with its styles inside and relative links, so the same folder serves the web and the app.
export const sheetOf = (built, language, page, { style, logo, font }) => {
  const { other, twin } = twinOf(built, language, page);
  const pages = built.get(language.code);
  const up = language.dir ? "../" : "";
  const listed = pages
    .map(
      (one) =>
        `<li><a href="${href(one.slug)}"${one.slug === page.slug ? ' aria-current="page"' : ""}>${escaped(one.label)}</a>${one.slug === page.slug && one.subs.length ? `<ol class="subs">${one.subs.map((sub) => `<li><a href="#${sub.id}">${escaped(sub.title)}</a></li>`).join("")}</ol>` : ""}</li>`,
    )
    .join("");
  const before = pages[page.at - 1];
  const after = pages[page.at + 1];
  const near = [
    before &&
      `<a rel="prev" href="${href(before.slug)}" title="${language.before}"><span aria-hidden="true">←</span>${escaped(before.label)}</a>`,
    after &&
      `<a rel="next" href="${href(after.slug)}" title="${language.after}">${escaped(after.label)}<span aria-hidden="true">→</span></a>`,
  ]
    .filter(Boolean)
    .join("");
  const title = page.slug === "index" ? language.title : `${page.title} · ${language.site}`;
  return `<!doctype html>
<html lang="${language.code}">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<meta name="color-scheme" content="light dark">
<meta name="theme-color" media="(prefers-color-scheme: light)" content="#faf8f4">
<meta name="theme-color" media="(prefers-color-scheme: dark)" content="#0e0d0b">
<title>${escaped(title)}</title>
${heads(language, page, title, other, twin)}
<link rel="icon" href="${up}${logo}" type="image/svg+xml">
<link rel="preload" href="${font}" as="font" type="font/woff2" crossorigin>
${THEME}
<style>${style}</style>
${AHEAD}
</head>
<body>
<header class="top">
<a class="brand" href="./"><img src="${up}${logo}" alt="" width="30" height="30">Tisty Markdown</a>
<a class="docs" href="${language.docsAt}">${language.docs}</a>
<a class="out" href="${language.home}">${language.meet}</a>
<a class="lang" lang="${other.code}" hreflang="${other.code}" title="${other.reading}" href="${language.dir ? "../" : other.dir}${twin.slug === "index" ? "" : twin.slug}">${other.name}</a>
<button class="theme" type="button" aria-label="${language.dark}" title="${language.dark}" onclick="${SWITCH}">${MOON}${SUN}</button>
</header>
<div class="wrap">
<nav class="topics" aria-label="${language.topics}"><div class="search" data-search><input class="search-waiting" type="search" placeholder="${language.search}" aria-label="${language.search}"></div><ol>${listed}</ol></nav>
<main data-pagefind-body>${page.html}<nav class="near" aria-label="${language.near}" data-pagefind-ignore>${near}</nav></main>
</div>${SEARCH}${page.html.includes('class="widget"') ? HOSTS : ""}
</body>
</html>
`;
};

// Without its own page, Pages answers a wrong address with the start page and a 200.
export const missing = ({ style, logo }) => `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Not here · Tisty Markdown</title>
<meta name="robots" content="noindex">
<link rel="icon" href="/${logo}" type="image/svg+xml">
${THEME}
<style>${style}</style>
</head>
<body>
<header class="top"><a class="brand" href="/"><img src="/${logo}" alt="" width="36" height="36">Tisty Markdown</a><a class="out" href="https://www.tisty.app/">Meet Tisty</a></header>
<div class="wrap"><main class="page"><h1>Nothing here</h1><p>This page is not part of the guide. <a href="/">Start from the beginning</a> · <a href="/es/" lang="es">Empieza desde el inicio</a></p></main></div>
</body>
</html>
`;

export const mapped = (built) => {
  const urls = LANGUAGES.flatMap((language) =>
    built.get(language.code).map((page) => {
      const { other, twin } = twinOf(built, language, page);
      return `<url><loc>${address(language, page)}</loc>${page.changed ? `<lastmod>${page.changed}</lastmod>` : ""}<xhtml:link rel="alternate" hreflang="${language.code}" href="${address(language, page)}"/><xhtml:link rel="alternate" hreflang="${other.code}" href="${address(other, twin)}"/><xhtml:link rel="alternate" hreflang="x-default" href="${address(LANGUAGES[0], language === LANGUAGES[0] ? page : twin)}"/></url>`;
    }),
  );
  return `<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9" xmlns:xhtml="http://www.w3.org/1999/xhtml">
${urls.join("\n")}
</urlset>
`;
};

// Only where a widget lives: it hears how tall each one is and tells them light or dark.
const HOSTS = `<script>(()=>{const r=document.documentElement;const frames=()=>document.querySelectorAll("iframe.widget");const tell=(w)=>w&&w.postMessage({tisty:"theme",dark:r.classList.contains("dark")},"*");addEventListener("message",(e)=>{if(!e.data||e.data.tisty!=="height")return;for(const f of frames())if(f.contentWindow===e.source){f.style.height=Math.min(Math.max(e.data.h,40),2000)+"px";tell(e.source)}});new MutationObserver(()=>{for(const f of frames())tell(f.contentWindow)}).observe(r,{attributes:true,attributeFilter:["class"]})})()</script>`;

// Search weighs more than a page, so it comes when a reader reaches for the box, and keeps what they already typed.
export const searchOnReach = () => {
  const box = document.querySelector("[data-search]");
  if (!box) return;
  const waiting = box.querySelector("input");
  const landed = (one) =>
    new Promise((done, failed) => {
      one.onload = done;
      one.onerror = failed;
    });
  const load = async () => {
    box.removeEventListener("pointerenter", load);
    box.removeEventListener("focusin", load);
    const sheet = document.createElement("link");
    sheet.rel = "stylesheet";
    sheet.href = "/pagefind/pagefind-ui.css";
    const code = document.createElement("script");
    code.src = "/pagefind/pagefind-ui.js";
    const both = Promise.all([landed(sheet), landed(code)]);
    document.head.append(sheet, code);
    try {
      await both;
      new PagefindUI({ element: box, showSubResults: true, showImages: false, resetStyles: false });
      const field = box.querySelector("input:not(.search-waiting)");
      if (!field) throw new Error("Pagefind drew no box");
      const typed = waiting.value;
      const held = document.activeElement === waiting;
      waiting.remove();
      if (typed) {
        field.value = typed;
        field.dispatchEvent(new Event("input", { bubbles: true }));
      }
      if (held) field.focus();
    } catch {
      box.remove();
    }
  };
  box.addEventListener("pointerenter", load);
  box.addEventListener("focusin", load);
};

const SEARCH = `<script>(${searchOnReach})()</script>`;

// Declarative, not a script: the browser readies a page while the pointer rests on its link.
const AHEAD = `<script type="speculationrules">{"prerender":[{"where":{"selector_matches":".topics a, .near a, .page a[href]:not([href^='http'])"},"eagerness":"moderate"}]}</script>`;

const YEAR = "public, max-age=31536000, immutable";
const DAY = "public, max-age=86400";
const SHORT = "public, max-age=300, s-maxage=86400, stale-while-revalidate=3600";
const HTTPS_ONLY = "/*\n  Strict-Transport-Security: max-age=15552000";
// Pages stops at this many rules without a word, and the pages past it would lose their cache.
const MOST_RULES = 100;

const rule = (path, cache) => `${path}\n  Cache-Control: ${cache}`;

// Each page is named because Pages joins the values of every rule that matches, so `/*` carries only a header no other rule sets.
export const headers = (logo, built) => {
  const rules = [
    HTTPS_ONLY,
    rule(`/${logo}`, YEAR),
    rule("/fonts/*", YEAR),
    rule("/tisty.svg", DAY),
    rule("/rina.jpg", DAY),
    rule("/es/tisty.svg", DAY),
    rule("/es/rina.jpg", DAY),
    ...LANGUAGES.flatMap((language) =>
      built.get(language.code).map((page) => rule(`/${language.dir}${page.slug === "index" ? "" : page.slug}`, SHORT)),
    ),
  ];
  if (rules.length > MOST_RULES)
    throw new Error(`the guide needs ${rules.length} header rules, and Pages applies ${MOST_RULES}`);
  return `${rules.join("\n")}\n`;
};

export const robots = `User-agent: *\nAllow: /\n\nSitemap: ${SITE}sitemap.xml\n`;
